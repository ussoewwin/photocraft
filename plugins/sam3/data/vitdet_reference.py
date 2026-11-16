"""Reference ViTDet + FPN forward for the SAM 3.1 Multiplex pack.

Written against `comfy/ldm/sam3/sam.py` (ViTDet 223-298, Block 185-211, Attention 165-183,
window_partition 122-131, window_unpartition 134-142, rope_2d 145-150, FPNScaleConv 301-330) and the
Flux `rope` / `apply_rope1` helpers the checkpoint's RoPE comes from. It reads the HSWQ pack directly
and dequantises the ConvRot linears the same way the Rust loader does, so a difference here means a
difference in the model math, not in the weights.
"""

import json
import math
import sys

import numpy as np
import torch
import torch.nn.functional as F
from PIL import Image
from safetensors import safe_open

DEV = "cuda" if torch.cuda.is_available() else "cpu"
DT = torch.float16


def build_hadamard(size):
    h4 = torch.tensor([[1, 1, 1, -1], [1, 1, -1, 1], [1, -1, 1, 1], [-1, 1, 1, 1]], dtype=torch.float32)
    h, cur = h4, 4
    while cur < size:
        h = torch.kron(h, h4)
        cur *= 4
    return h / (size**0.5)


H256 = build_hadamard(256)


class Pack:
    def __init__(self, path):
        self.f = safe_open(path, framework="pt", device="cpu")
        self.keys = set(self.f.keys())

    def has(self, name):
        return name in self.keys

    def get(self, name):
        sl = self.f.get_slice(name)
        if sl.get_dtype() == "I8":
            q = self.f.get_tensor(name).float()
            s = self.f.get_tensor(name[: -len(".weight")] + ".weight_scale").float()
            w = q * s
            o, i = w.shape
            g = i // 256
            w = (w.view(o * g, 256) @ H256).reshape(o, i)
            return w.to(DT).to(DEV)
        t = self.f.get_tensor(name)
        return t.to(DEV)

    def linear(self, prefix):
        # q/k/v projections live inside a fused `in_proj_{weight,bias}` in the pack; the Rust loader
        # splits them on load, so the reference splits them here too.
        for suffix, index in ((".q_proj", 0), (".k_proj", 1), (".v_proj", 2)):
            if prefix.endswith(suffix) and not self.has(prefix + ".weight"):
                stem = prefix[: -len(suffix)]
                if self.has(stem + ".in_proj_weight"):
                    w = self.get(stem + ".in_proj_weight")
                    third = w.shape[0] // 3
                    w = w[index * third : (index + 1) * third]
                    b = None
                    if self.has(stem + ".in_proj_bias"):
                        full = self.get(stem + ".in_proj_bias")
                        b = full[index * third : (index + 1) * third]
                    return w, b
        w = self.get(prefix + ".weight")
        b = self.get(prefix + ".bias") if self.has(prefix + ".bias") else None
        return w, b


def lin(x, w, b):
    out = x @ w.t()
    return out if b is None else out + b


def layernorm(x, prefix, pack, eps=1e-5):
    # The reference runs LayerNorm through ComfyUI's ops in the model dtype; f32 statistics are what
    # torch's own LayerNorm does internally for half inputs, so keep the same accumulation.
    m = x.float().mean(-1, keepdim=True)
    v = x.float().var(-1, unbiased=False, keepdim=True)
    out = (x.float() - m) / torch.sqrt(v + eps)
    if pack.has(prefix + ".weight"):
        out = out * pack.get(prefix + ".weight").float()
    if pack.has(prefix + ".bias"):
        out = out + pack.get(prefix + ".bias").float()
    return out.to(x.dtype)


def rope_table(w, h, dim, theta=10000.0, scale_pos=1.0):
    """`rope_2d(w, h, dim, theta, scale_pos)` reduced to its cos/sin factors."""
    axis_dim = dim // 2
    axis_half = axis_dim // 2
    scale = torch.linspace(0, (axis_dim - 2) / axis_dim, steps=axis_half, dtype=torch.float64, device=DEV)
    omega = (1.0 / (theta**scale)).float()
    cos = torch.zeros(w * h, dim // 2, device=DEV)
    sin = torch.zeros(w * h, dim // 2, device=DEV)
    xs = torch.arange(w, device=DEV).float() * scale_pos
    ys = torch.arange(h, device=DEV).float() * scale_pos
    for y in range(h):
        for x in range(w):
            t = y * w + x
            ax = (xs[x] * omega)
            ay = (ys[y] * omega)
            cos[t, :axis_half] = ax.cos()
            sin[t, :axis_half] = ax.sin()
            cos[t, axis_half : axis_half * 2] = ay.cos()
            sin[t, axis_half : axis_half * 2] = ay.sin()
    return cos.unsqueeze(0).unsqueeze(0).to(DT), sin.unsqueeze(0).unsqueeze(0).to(DT)


def apply_rope(x, cos, sin):
    """`apply_rope1`: the trailing axis splits into (half, 2) rotation pairs."""
    b, h, n, d = x.shape
    half = d // 2
    pairs = x.reshape(b, h, n, half, 2)
    even = pairs[..., 0]
    odd = pairs[..., 1]
    c = cos[:, :, :n, :]
    s = sin[:, :, :n, :]
    out_even = c * even - s * odd
    out_odd = s * even + c * odd
    return torch.stack([out_even, out_odd], dim=-1).reshape(b, h, n, d)


def window_partition(x, ws):
    B, H, W, C = x.shape
    pad_h = (ws - H % ws) % ws
    pad_w = (ws - W % ws) % ws
    if pad_h or pad_w:
        x = F.pad(x, (0, 0, 0, pad_w, 0, pad_h))
    Hp, Wp = H + pad_h, W + pad_w
    x = x.view(B, Hp // ws, ws, Wp // ws, ws, C)
    return x.permute(0, 1, 3, 2, 4, 5).contiguous().view(-1, ws, ws, C), (Hp, Wp)


def window_unpartition(windows, ws, pad_hw, hw):
    Hp, Wp = pad_hw
    H, W = hw
    B = windows.shape[0] // (Hp * Wp // ws // ws)
    x = windows.view(B, Hp // ws, Wp // ws, ws, ws, -1)
    x = x.permute(0, 1, 3, 2, 4, 5).contiguous().view(B, Hp, Wp, -1)
    if Hp > H or Wp > W:
        x = x[:, :H, :W, :].contiguous()
    return x


def attention(x, prefix, pack, cos, sin):
    B, N, C = x.shape
    w, b = pack.linear(prefix + ".qkv")
    qkv = lin(x, w, b).reshape(B, N, 3, 16, 64)
    q, k, v = qkv.permute(2, 0, 3, 1, 4).unbind(0)
    q = apply_rope(q, cos, sin)
    k = apply_rope(k, cos, sin)
    att = (q.float() @ k.float().transpose(-1, -2) / math.sqrt(64)).softmax(-1).to(DT)
    o = att @ v
    o = o.permute(0, 2, 1, 3).reshape(B, N, C)
    w2, b2 = pack.linear(prefix + ".proj")
    return lin(o, w2, b2)


def block(x, index, pack, cos_win, sin_win, cos_glob, sin_glob, window=24, global_blocks=(7, 15, 23, 31)):
    p = f"detector.backbone.vision_backbone.trunk.blocks.{index}"
    shortcut = x
    h = layernorm(x, p + ".norm1", pack)
    if index in global_blocks:
        B, H, W, C = h.shape
        flat = h.view(B, H * W, C)
        h = attention(flat, p + ".attn", pack, cos_glob, sin_glob).view(B, H, W, C)
    else:
        H, W = h.shape[1], h.shape[2]
        win, pad_hw = window_partition(h, window)
        win = win.view(win.shape[0], window * window, -1)
        win = attention(win, p + ".attn", pack, cos_win, sin_win)
        win = win.view(-1, window, window, win.shape[-1])
        h = window_unpartition(win, window, pad_hw, (H, W))
    x = shortcut + h
    n2 = layernorm(x, p + ".norm2", pack)
    w1, b1 = pack.linear(p + ".mlp.fc1")
    w2, b2 = pack.linear(p + ".mlp.fc2")
    x = x + lin(F.gelu(lin(n2, w1, b1)), w2, b2)
    return x


def conv(x, prefix, pack, stride=1, padding=0):
    w = pack.get(prefix + ".weight")
    b = pack.get(prefix + ".bias") if pack.has(prefix + ".bias") else None
    out = F.conv2d(x, w, b, stride=stride, padding=padding)
    return out


def fpn(x, prefix, scale, pack):
    if scale == 4.0:
        x = F.gelu(F.conv_transpose2d(x, pack.get(prefix + ".dconv_2x2_0.weight"), pack.get(prefix + ".dconv_2x2_0.bias"), stride=2))
        x = F.conv_transpose2d(x, pack.get(prefix + ".dconv_2x2_1.weight"), pack.get(prefix + ".dconv_2x2_1.bias"), stride=2)
    elif scale == 2.0:
        x = F.conv_transpose2d(x, pack.get(prefix + ".dconv_2x2.weight"), pack.get(prefix + ".dconv_2x2.bias"), stride=2)
    x = conv(x, prefix + ".conv_1x1", pack)
    return conv(x, prefix + ".conv_3x3", pack, padding=1)


def main():
    pack_path, image_path, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
    pack = Pack(pack_path)

    img = Image.open(image_path).convert("RGB").resize((1008, 1008), Image.BILINEAR)
    arr = torch.from_numpy(np.asarray(img)).float() / 255.0
    x = arr.permute(2, 0, 1).unsqueeze(0).to(DEV).to(DT)
    print("image", tuple(x.shape), "on", DEV)

    # patch embed
    x = conv(x, "detector.backbone.vision_backbone.trunk.patch_embed.proj", pack, stride=14)
    B, C, Hp, Wp = x.shape
    x = x.permute(0, 2, 3, 1).reshape(B, Hp * Wp, C)

    pos = pack.get("detector.backbone.vision_backbone.trunk.pos_embed")
    old = int(math.sqrt(pos.shape[1] - 1))
    spatial = pos[:, 1:].reshape(1, old, old, -1).permute(0, 3, 1, 2)
    tiles = Hp // old + 1
    tiled = spatial.tile([1, 1, tiles, tiles])[:, :, :Hp, :Wp]
    tiled = tiled.permute(0, 2, 3, 1).reshape(1, Hp * Wp, -1)
    x = x + tiled

    x = x.view(B, Hp, Wp, C)
    x = layernorm(x, "detector.backbone.vision_backbone.trunk.ln_pre", pack)

    cos_g, sin_g = rope_table(72, 72, 64, scale_pos=24.0 / 72.0)
    cos_w, sin_w = rope_table(24, 24, 64, scale_pos=1.0)

    limit = int(sys.argv[4]) if len(sys.argv) > 4 else 32
    for i in range(limit):
        x = block(x, i, pack, cos_w, sin_w, cos_g, sin_g)
    if limit < 32:
        trunk_only = x.permute(0, 3, 1, 2)
        print("trunk", tuple(trunk_only.shape))
        trunk_only.flatten().float().cpu().numpy().astype("float32").tofile(out_path)
        print("wrote", out_path, trunk_only.numel())
        return

    trunk = x.permute(0, 3, 1, 2)
    print("trunk", tuple(trunk.shape))

    levels = {}
    for name, group in (("convs", "convs"), ("interactive_convs", "interactive_convs")):
        outs = []
        for idx, scale in enumerate([4.0, 2.0, 1.0]):
            outs.append(fpn(trunk, f"detector.backbone.vision_backbone.{group}.{idx}", scale, pack))
        levels[name] = outs
        print(name, [tuple(o.shape) for o in outs])

    buffer = []
    meta = []
    order = [("trunk", trunk)] + [(f"convs.{i}", t) for i, t in enumerate(levels["convs"])] + [
        (f"interactive_convs.{i}", t) for i, t in enumerate(levels["interactive_convs"])
    ]
    offset = 0
    for name, tensor in order:
        flat = tensor.flatten().float().cpu().numpy()
        buffer.append(flat)
        meta.append({"name": name, "shape": list(tensor.shape), "offset": offset, "count": int(flat.size)})
        offset += flat.size
    np.concatenate(buffer).astype(np.float32).tofile(out_path)
    json.dump(meta, open(out_path.rsplit(".", 1)[0] + ".json", "w"), indent=1)
    print("wrote", out_path, offset, "f32")


if __name__ == "__main__":
    main()
