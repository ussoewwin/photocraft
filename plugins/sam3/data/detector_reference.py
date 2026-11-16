"""Reference SAM 3.1 detector forward (text prompt only).

Written against `comfy/ldm/sam3/detector.py`: SplitMHA (36-58), MLPWithNorm (61-81), EncoderLayer
(84-104), DecoderLayer (122-142), TransformerDecoder (145-214), PixelDecoder (314-325),
MaskPredictor (328-335), SegmentationHead (338-364), DotProductScoring (367-385) and
`SAM3Detector._detect` (422-470) for multiplex=True (scalp=0, three FPN levels).

It reuses `vitdet_reference.py` for the backbone and the CLIP tower so the comparison isolates the
detector.
"""

import json
import math
import sys

import numpy as np
import torch
import torch.nn.functional as F

sys.path.insert(0, "data")
from vitdet_reference import Pack, DT, DEV, block, conv, fpn, layernorm, lin, rope_table, H256  # noqa: E402

D = 256
HEADS = 8
EPS = 1e-5


def split_mha(x, prefix, pack, k_in=None, v_in=None, mask=None):
    q = lin(x, *pack.linear(prefix + ".q_proj"))
    if k_in is None:
        k_in = x
    k = lin(k_in, *pack.linear(prefix + ".k_proj"))
    v = lin(v_in if v_in is not None else k_in, *pack.linear(prefix + ".v_proj"))
    B, N, C = q.shape
    NK = k.shape[1]
    qh = q.reshape(B, N, HEADS, C // HEADS).permute(0, 2, 1, 3)
    kh = k.reshape(B, NK, HEADS, C // HEADS).permute(0, 2, 1, 3)
    vh = v.reshape(B, NK, HEADS, C // HEADS).permute(0, 2, 1, 3)
    scores = (qh.float() @ kh.float().transpose(-1, -2)) / math.sqrt(C // HEADS)
    if mask is not None:
        scores = scores + mask.float()
    att = scores.softmax(-1).to(DT) @ vh
    out = att.permute(0, 2, 1, 3).reshape(B, N, C)
    return lin(out, *pack.linear(prefix + ".out_proj"))


def mlp(x, prefix, pack, layers):
    for i in range(layers):
        x = lin(x, *pack.linear(f"{prefix}.layers.{i}"))
        if i < layers - 1:
            x = F.relu(x)
    return x


def sincos(vals):
    half = 128
    freqs = 10000.0 ** (2 * (torch.arange(half, device=vals.device) // 2) / half)
    raw = vals[..., None] * (2 * math.pi) / freqs
    return torch.stack((raw[..., 0::2].sin(), raw[..., 1::2].cos()), dim=-1).flatten(-2)


def position_encoding(feat):
    B, C, H, W = feat.shape
    gy = torch.arange(H, device=feat.device, dtype=torch.float32) / (H - 1 + 1e-6)
    gx = torch.arange(W, device=feat.device, dtype=torch.float32) / (W - 1 + 1e-6)
    yy, xx = torch.meshgrid(gy, gx, indexing="ij")
    enc = torch.cat((sincos(yy), sincos(xx)), dim=-1).permute(2, 0, 1).unsqueeze(0)
    return enc.expand(B, -1, -1, -1).to(feat.dtype)


def gen_sineembed(pos, num_feats=256):
    hdim = num_feats // 2
    freqs = 10000.0 ** (2 * (torch.arange(hdim, device=pos.device) // 2) / hdim)
    embeds = []
    for c in range(pos.shape[-1]):
        raw = (pos[..., c].float() * 2 * math.pi).unsqueeze(-1) / freqs
        embeds.append(torch.stack([raw[..., 0::2].sin(), raw[..., 1::2].cos()], dim=-1).flatten(-2))
    return torch.cat(embeds, dim=-1).to(pos.dtype)


def encoder_layer(x, pos, prefix, pack, text, text_mask):
    normed = layernorm(x, prefix + ".norm1", pack)
    qk = normed + pos
    x = x + split_mha(qk, prefix + ".self_attn", pack, qk, normed)
    if text is not None:
        normed = layernorm(x, prefix + ".norm2", pack)
        x = x + split_mha(normed, prefix + ".cross_attn_image", pack, text, text, mask=text_mask)
    normed = layernorm(x, prefix + ".norm3", pack)
    x = x + lin(F.relu(lin(normed, *pack.linear(prefix + ".linear1"))), *pack.linear(prefix + ".linear2"))
    return x


def decoder_layer(x, memory, x_pos, memory_pos, prefix, pack, text, text_mask, cross_bias):
    qk = x + x_pos
    x = layernorm(x + split_mha(qk, prefix + ".self_attn", pack, qk, x), prefix + ".norm2", pack)
    if text is not None:
        x = layernorm(x + split_mha(x + x_pos, prefix + ".ca_text", pack, text, text, mask=text_mask), prefix + ".catext_norm", pack)
    x = layernorm(x + split_mha(x + x_pos, prefix + ".cross_attn", pack, memory + memory_pos, memory, mask=cross_bias), prefix + ".norm1", pack)
    x = layernorm(x + lin(F.relu(lin(x, *pack.linear(prefix + ".linear1"))), *pack.linear(prefix + ".linear2")), prefix + ".norm3", pack)
    return x


def box_cxcywh_to_xyxy(x):
    cx, cy, w, h = x.unbind(-1)
    return torch.stack([cx - 0.5 * w, cy - 0.5 * h, cx + 0.5 * w, cy + 0.5 * h], dim=-1)


def inverse_sigmoid(x):
    return torch.log(x / (1 - x + 1e-6) + 1e-6)


def compute_box_rpb(ref_points, H, W, prefix, pack):
    xyxy = box_cxcywh_to_xyxy(ref_points)
    B, Q, _ = xyxy.shape
    coords_h = torch.arange(H, device=ref_points.device, dtype=torch.float32) / H
    coords_w = torch.arange(W, device=ref_points.device, dtype=torch.float32) / W
    deltas_x = coords_w.view(1, 1, -1, 1) - xyxy[:, :, None, 0:3:2]
    deltas_y = coords_h.view(1, 1, -1, 1) - xyxy[:, :, None, 1:4:2]
    log2_8 = float(math.log2(8))

    def log_scale(d):
        return torch.sign(d * 8) * torch.log2(torch.abs(d * 8) + 1.0) / log2_8

    rpb_x = mlp(log_scale(deltas_x).to(ref_points.dtype), prefix + ".boxRPB_embed_x", pack, 2)
    rpb_y = mlp(log_scale(deltas_y).to(ref_points.dtype), prefix + ".boxRPB_embed_y", pack, 2)
    bias = (rpb_y.unsqueeze(3) + rpb_x.unsqueeze(2)).flatten(2, 3).permute(0, 3, 1, 2)
    pres = torch.zeros(B, bias.shape[1], 1, bias.shape[3], device=bias.device, dtype=bias.dtype)
    return torch.cat([pres, bias], dim=2)


def main():
    pack_path, image_path, prompt_ids_path, out_path = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
    pack = Pack(pack_path)
    meta = json.load(open(prompt_ids_path))
    ids = meta["ids"]
    am = meta["attention_mask"]

    # Backbone (reuse the ViTDet reference).
    from PIL import Image
    img = Image.open(image_path).convert("RGB").resize((1008, 1008), Image.BILINEAR)
    arr = torch.from_numpy(np.asarray(img).copy()).float() / 255.0
    x = arr.permute(2, 0, 1).unsqueeze(0).to(DEV).to(DT)

    x = conv(x, "detector.backbone.vision_backbone.trunk.patch_embed.proj", pack, stride=14)
    B, C, Hp, Wp = x.shape
    x = x.permute(0, 2, 3, 1).reshape(B, Hp * Wp, C)
    pos = pack.get("detector.backbone.vision_backbone.trunk.pos_embed")
    old = int(math.sqrt(pos.shape[1] - 1))
    spatial = pos[:, 1:].reshape(1, old, old, -1).permute(0, 3, 1, 2)
    tiles = Hp // old + 1
    tiled = spatial.tile([1, 1, tiles, tiles])[:, :, :Hp, :Wp].permute(0, 2, 3, 1).reshape(1, Hp * Wp, -1)
    x = x + tiled
    x = layernorm(x.view(B, Hp, Wp, C), "detector.backbone.vision_backbone.trunk.ln_pre", pack)
    cos_g, sin_g = rope_table(72, 72, 64, scale_pos=24.0 / 72.0)
    cos_w, sin_w = rope_table(24, 24, 64, scale_pos=1.0)
    for i in range(32):
        x = block(x, i, pack, cos_w, sin_w, cos_g, sin_g)
    trunk = x.permute(0, 3, 1, 2)
    features = [fpn(trunk, f"detector.backbone.vision_backbone.convs.{i}", s, pack) for i, s in enumerate([4.0, 2.0, 1.0])]
    positions = [position_encoding(f) for f in features]

    # CLIP text tower.
    base = "detector.backbone.language_backbone.encoder"
    tok = pack.get(f"{base}.token_embedding.weight")
    pos_e = pack.get(f"{base}.positional_embedding")
    ids_t = torch.tensor([ids], dtype=torch.long, device=DEV)
    am_t = torch.tensor([am], dtype=torch.long, device=DEV)
    h = tok[ids_t[0]] + pos_e
    h = h.unsqueeze(0).to(DT)
    L = h.shape[1]
    neg = torch.finfo(h.dtype).min
    mask = 1.0 - am_t.to(h.dtype).reshape((1, 1, -1, L)).expand(1, 1, L, L)
    mask = mask.masked_fill(mask.to(torch.bool), neg) + torch.full((L, L), neg, dtype=h.dtype, device=DEV).triu_(1)
    for i in range(24):
        p = f"{base}.transformer.resblocks.{i}"
        n1 = layernorm(h, p + ".ln_1", pack)
        in_w = pack.get(f"{p}.attn.in_proj_weight")
        in_b = pack.get(f"{p}.attn.in_proj_bias")
        q = n1 @ in_w[0:1024].t() + in_b[0:1024]
        k = n1 @ in_w[1024:2048].t() + in_b[1024:2048]
        v = n1 @ in_w[2048:3072].t() + in_b[2048:3072]
        qh = q.reshape(1, L, 16, 64).permute(0, 2, 1, 3)
        kh = k.reshape(1, L, 16, 64).permute(0, 2, 1, 3)
        vh = v.reshape(1, L, 16, 64).permute(0, 2, 1, 3)
        att = (qh.float() @ kh.float().transpose(-1, -2) / math.sqrt(64) + mask.float()).softmax(-1).to(DT)
        o = (att @ vh).permute(0, 2, 1, 3).reshape(1, L, 1024)
        h = h + lin(o, *pack.linear(p + ".attn.out_proj"))
        n2 = layernorm(h, p + ".ln_2", pack)
        h = h + lin(F.gelu(lin(n2, *pack.linear(p + ".mlp.c_fc"))), *pack.linear(p + ".mlp.c_proj"))
    h = layernorm(h, base + ".ln_final", pack)
    text = lin(h.to(DT), *pack.linear("detector.backbone.language_backbone.resizer"))
    text_mask = (am_t == 0).to(torch.bool)

    # Detector.
    enc_feat, enc_pos = features[-1], positions[-1]
    _, _, Hh, Ww = enc_feat.shape
    img_flat = enc_feat.flatten(2).permute(0, 2, 1)
    pos_flat = enc_pos.flatten(2).permute(0, 2, 1)

    geo = "detector.geometry_encoder"
    img_normed = layernorm(img_flat, geo + ".img_pre_norm", pack)
    cls = pack.get(geo + ".cls_embed.weight").view(1, 1, -1).expand(B, -1, -1)
    geo_cls = layernorm(lin(cls, *pack.linear(geo + ".final_proj")), geo + ".norm", pack)

    def run_geo_layer(x, p):
        x = x + split_mha(layernorm(x, p + ".norm1", pack), p + ".self_attn", pack)
        x = x + split_mha(layernorm(x, p + ".norm2", pack), p + ".cross_attn_image", pack, img_flat + pos_flat, img_flat)
        return x + lin(F.relu(lin(layernorm(x, p + ".norm3", pack), *pack.linear(p + ".linear1"))), *pack.linear(p + ".linear2"))

    for i in range(3):
        geo_cls = run_geo_layer(geo_cls, f"{geo}.encode.{i}")
    geo_cls = layernorm(geo_cls, geo + ".encode_norm", pack)

    prompt = torch.cat([text, geo_cls], dim=1)
    prompt_mask = torch.cat([text_mask, torch.ones(B, 1, dtype=torch.bool, device=DEV)], dim=1)
    add_mask = torch.zeros(B, 1, 1, prompt.shape[1], device=DEV)
    add_mask = add_mask.masked_fill(prompt_mask[:, None, None, :], -1.0e9)

    import os
    limit = int(os.environ.get("SAM3_ENC_LAYERS", "6"))
    memory = img_flat
    for i in range(limit):
        memory = encoder_layer(memory, pos_flat, f"detector.transformer.encoder.layers.{i}", pack, prompt, add_mask)

    dec = "detector.transformer.decoder"
    tgt = pack.get(dec + ".query_embed.weight").unsqueeze(0).expand(B, -1, -1)
    presence = pack.get(dec + ".presence_token.weight")[None].expand(B, -1, -1)
    ref_points = pack.get(dec + ".reference_points.weight").unsqueeze(0).expand(B, -1, -1).sigmoid()

    for i in range(6):
        query_pos = mlp(gen_sineembed(ref_points, D), dec + ".ref_point_head", pack, 2)
        tgt_p = torch.cat([presence, tgt], dim=1)
        pos_p = torch.cat([torch.zeros_like(presence), query_pos], dim=1)
        tgt_p = decoder_layer(tgt_p, memory, pos_p, pos_flat, f"{dec}.layers.{i}", pack, prompt, add_mask,
                              compute_box_rpb(ref_points, Hh, Ww, dec, pack))
        presence, tgt = tgt_p[:, :1], tgt_p[:, 1:]
        if i < 5:
            ref_points = (inverse_sigmoid(ref_points) + mlp(layernorm(tgt, dec + ".norm", pack), dec + ".bbox_embed", pack, 3)).sigmoid().detach()

    query_out = layernorm(tgt, dec + ".norm", pack)
    boxes = (inverse_sigmoid(ref_points) + mlp(query_out, dec + ".bbox_embed", pack, 3)).sigmoid()

    # Scoring.
    p = "detector.dot_prod_scoring"
    orig = prompt
    hidden = prompt
    for i in range(2):
        hidden = lin(hidden, *pack.linear(f"{p}.prompt_mlp.layers.{i}"))
        if i < 1:
            hidden = F.relu(hidden)
    pe = layernorm(hidden + orig, p + ".prompt_mlp.out_norm", pack)
    weight = prompt_mask.unsqueeze(-1).to(pe.dtype)
    pooled = (pe * weight).sum(1) / weight.sum(1).clamp(min=1)
    hs = lin(query_out, *pack.linear(p + ".hs_proj"))
    pp = lin(pooled, *pack.linear(p + ".prompt_proj")).unsqueeze(-1).to(hs.dtype)
    scores = ((hs @ pp) * (1.0 / math.sqrt(D))).clamp(-12.0, 12.0).squeeze(-1)

    # Segmentation head.
    p = "detector.segmentation_head"
    enc_normed = layernorm(memory, p + ".cross_attn_norm", pack)
    encoded = split_mha(enc_normed, p + ".cross_attend_prompt", pack, prompt, prompt, mask=add_mask) + memory
    visual = encoded[:, : Hh * Ww].permute(0, 2, 1).view(B, D, Hh, Ww)
    levels = list(features)
    levels[-1] = visual
    prev = levels[-1]
    for i, feat in enumerate(levels[:-1][::-1]):
        prev = F.relu(F.group_norm(F.conv2d(feat + F.interpolate(prev, size=feat.shape[-2:], mode="nearest"),
                                            pack.get(f"{p}.pixel_decoder.conv_layers.{i}.weight"),
                                            pack.get(f"{p}.pixel_decoder.conv_layers.{i}.bias"), padding=1),
                                   8, pack.get(f"{p}.pixel_decoder.norms.{i}.weight"),
                                   pack.get(f"{p}.pixel_decoder.norms.{i}.bias")))
    inst = F.conv2d(prev, pack.get(p + ".instance_seg_head.weight"), pack.get(p + ".instance_seg_head.bias"))
    mask_embed = mlp(query_out, p + ".mask_predictor.mask_embed", pack, 3)
    masks = torch.einsum("bqc,bchw->bqhw", mask_embed, inst)

    out = {
        "boxes_xyxy": box_cxcywh_to_xyxy(boxes).flatten().float().cpu().numpy(),
        "scores": scores.flatten().float().cpu().numpy(),
        "queries": query_out.flatten().float().cpu().numpy(),
        "masks": masks.flatten().float().cpu().numpy(),
        "memory": memory.flatten().float().cpu().numpy(),
        "prompt": prompt.flatten().float().cpu().numpy(),
    }
    np.concatenate([out["boxes_xyxy"], out["scores"], out["queries"], out["memory"], out["prompt"]]).astype(np.float32).tofile(out_path)
    out["masks"].astype(np.float32).tofile(out_path.replace(".bin", ".masks.f32"))
    print("boxes", tuple(boxes.shape), "scores", tuple(scores.shape), "masks", tuple(masks.shape))

    order = torch.argsort(scores.flatten(), descending=True)[:5]
    b = box_cxcywh_to_xyxy(boxes).flatten()
    for rank, idx in enumerate(order.tolist()):
        print(f"  #{rank} query {idx}: score {scores.flatten()[idx].item():.6f}  box [{b[idx*4]:.4f}, {b[idx*4+1]:.4f}, {b[idx*4+2]:.4f}, {b[idx*4+3]:.4f}]")


if __name__ == "__main__":
    main()
