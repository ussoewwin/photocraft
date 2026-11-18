"""Reference SAM 3.1 interactive segmenter (`SAM3Model.forward_segment`).

Written against `comfy/ldm/sam3/tracker.py`: `forward_sam_heads` (268-329), `_upscale_masks`
(425-433), `SAMMaskDecoder` (436-515), `SAMPromptEncoder` (518-577), and `comfy/ldm/sam3/sam.py`:
`SAMAttention` (28-43), `TwoWayAttentionBlock` (46-70), `TwoWayTransformer` (73-90),
`PositionEmbeddingRandom` (93-117). The driving code is `SAM3Model.forward_segment`
(`detector.py:524-565`).

It reuses `vitdet_reference.py` for the trunk and the FPN necks so the comparison isolates the
prompt encoder + decoder.

Usage:
    python tracker_reference.py <pack> <image> <out.bin> --box x1,y1,x2,y2
    python tracker_reference.py <pack> <image> <out.bin> --point x,y
    python tracker_reference.py <pack> <image> <out.bin> --mask <mask.f32>
"""

import json
import math
import os
import sys

import numpy as np
import torch
import torch.nn.functional as F

sys.path.insert(0, "data")
from vitdet_reference import Pack, DT, DEV, block, conv, fpn, layernorm, lin, rope_table  # noqa: E402

D = 256
HEADS = 8
EPS = 1e-5
IMAGE = 1008
GRID = 72
PROMPT_GRID = GRID * 4
# `forward_sam_heads` resizes the mask prompt with `antialias=True` (`tracker.py:293`). The switch
# exists so the downscale kernel can be isolated in the comparison against the Rust build.
ANTIALIAS = os.environ.get("SAM3_NO_ANTIALIAS", "0") != "1"
PE = "tracker.model.interactive_sam_prompt_encoder"
MD = "tracker.model.interactive_sam_mask_decoder"


def ln2d(x, prefix, pack):
    """`LayerNorm2d`: LayerNorm over channels (`cascade/common.py:64-69`)."""
    w = pack.get(prefix + ".weight")
    b = pack.get(prefix + ".bias")
    return F.layer_norm(x.permute(0, 2, 3, 1), (x.shape[1],), w, b, EPS).permute(0, 3, 1, 2)


def sam_attention(q, k, v, prefix, pack):
    """`SAMAttention` (`sam.py:39-43`)."""
    q = lin(q, *pack.linear(prefix + ".q_proj"))
    k = lin(k, *pack.linear(prefix + ".k_proj"))
    v = lin(v, *pack.linear(prefix + ".v_proj"))
    b, n, c = q.shape
    nk = k.shape[1]
    hd = c // HEADS
    qh = q.reshape(b, n, HEADS, hd).permute(0, 2, 1, 3)
    kh = k.reshape(b, nk, HEADS, hd).permute(0, 2, 1, 3)
    vh = v.reshape(b, nk, HEADS, hd).permute(0, 2, 1, 3)
    scores = (qh.float() @ kh.float().transpose(-1, -2)) / math.sqrt(hd)
    att = scores.softmax(-1).to(vh.dtype) @ vh
    return lin(att.permute(0, 2, 1, 3).reshape(b, n, c), *pack.linear(prefix + ".out_proj"))


def two_way_block(q, k, qpe, kpe, prefix, pack, skip_first_pe):
    """`TwoWayAttentionBlock.forward` (`sam.py:59-70`)."""
    if skip_first_pe:
        q = layernorm(sam_attention(q, q, q, prefix + ".self_attn", pack), prefix + ".norm1", pack)
    else:
        qq = q + qpe
        q = layernorm(q + sam_attention(qq, qq, q, prefix + ".self_attn", pack), prefix + ".norm1", pack)
    qq, kk = q + qpe, k + kpe
    q = layernorm(q + sam_attention(qq, kk, k, prefix + ".cross_attn_token_to_image", pack), prefix + ".norm2", pack)
    mlp = lin(F.relu(lin(q, *pack.linear(prefix + ".mlp.lin1"))), *pack.linear(prefix + ".mlp.lin2"))
    q = layernorm(q + mlp, prefix + ".norm3", pack)
    qq, kk = q + qpe, k + kpe
    k = layernorm(k + sam_attention(kk, qq, q, prefix + ".cross_attn_image_to_token", pack), prefix + ".norm4", pack)
    return q, k


def two_way_transformer(src, pos, tokens, prefix, pack, depth=2):
    """`TwoWayTransformer.forward` (`sam.py:84-90`)."""
    q, k = tokens, src
    for i in range(depth):
        q, k = two_way_block(q, k, tokens, pos, f"{prefix}.layers.{i}", pack, i == 0)
    qq, kk = q + tokens, k + pos
    q = layernorm(q + sam_attention(qq, kk, k, prefix + ".final_attn_token_to_image", pack),
                  prefix + ".norm_final_attn", pack)
    return q, k


def encode_coords(coords, matrix):
    """`PositionEmbeddingRandom._encode` (`sam.py:99-104`), multiplication before the matmul."""
    orig = coords.dtype
    projected = 2 * math.pi * (2 * coords.float() - 1) @ matrix.float()
    return torch.cat([projected.sin(), projected.cos()], dim=-1).to(orig)


def dense_pe(matrix, h, w):
    """`PositionEmbeddingRandom.forward((h, w))` (`sam.py:106-111`)."""
    ones = torch.ones((h, w), device=DEV, dtype=torch.float32)
    norm_xy = torch.stack([(ones.cumsum(1) - 0.5) / w, (ones.cumsum(0) - 0.5) / h], dim=-1)
    return encode_coords(norm_xy, matrix).permute(2, 0, 1).unsqueeze(0)


def with_coords(coords, matrix, image_size):
    """`PositionEmbeddingRandom.forward_with_coords` (`sam.py:113-117`)."""
    norm = coords.clone()
    norm[:, :, 0] /= image_size[1]
    norm[:, :, 1] /= image_size[0]
    return encode_coords(norm, matrix)


def prompt_encoder(pack, points=None, boxes=None, masks=None):
    """`SAMPromptEncoder.forward` (`tracker.py:544-577`)."""
    matrix = pack.get(PE + ".pe_layer.positional_encoding_gaussian_matrix")
    point_embed = [pack.get(f"{PE}.point_embeddings.{i}.weight") for i in range(4)]
    not_a_point = pack.get(PE + ".not_a_point_embed.weight")
    ref = points[0] if points is not None else boxes if boxes is not None else masks
    sparse = torch.empty((1, 0, D), device=ref.device, dtype=ref.dtype)
    batch = 1

    if points is not None:
        coords, labels = points
        batch = coords.shape[0]
        if boxes is None:
            coords = torch.cat([coords, torch.zeros(batch, 1, 2, device=coords.device, dtype=coords.dtype)], dim=1)
            labels = torch.cat([labels, -torch.ones(batch, 1, device=labels.device, dtype=labels.dtype)], dim=1)
        pe = with_coords(coords + 0.5, matrix, (IMAGE, IMAGE))
        for i in range(4):
            pe[labels == i] += point_embed[i]
        invalid = labels == -1
        pe[invalid] = 0.0
        pe[invalid] += not_a_point
        sparse = torch.cat([sparse.expand(batch, -1, -1), pe], dim=1)

    if boxes is not None:
        batch = boxes.shape[0]
        corners = with_coords(boxes.reshape(-1, 2, 2) + 0.5, matrix, (IMAGE, IMAGE))
        corners[:, 0] += point_embed[2]
        corners[:, 1] += point_embed[3]
        sparse = torch.cat([sparse.expand(batch, -1, -1), corners], dim=1)

    if masks is not None:
        x = conv(masks, PE + ".mask_downscaling.0", pack, stride=2)
        x = F.gelu(ln2d(x, PE + ".mask_downscaling.1", pack))
        x = conv(x, PE + ".mask_downscaling.3", pack, stride=2)
        x = F.gelu(ln2d(x, PE + ".mask_downscaling.4", pack))
        dense = conv(x, PE + ".mask_downscaling.6", pack)
    else:
        dense = pack.get(PE + ".no_mask_embed.weight").reshape(1, -1, 1, 1).expand(batch, -1, GRID, GRID)

    return sparse, dense


def mask_decoder(pack, image_embedding, image_pe, sparse, dense, high_res, multimask_output):
    """`SAMMaskDecoder.forward` (`tracker.py:467-515`)."""
    b = sparse.shape[0]
    tokens = torch.cat([pack.get(MD + ".obj_score_token.weight"),
                        pack.get(MD + ".iou_token.weight"),
                        pack.get(MD + ".mask_tokens.weight")], dim=0)
    tokens = torch.cat([tokens.unsqueeze(0).expand(b, -1, -1), sparse], dim=1)

    src = image_embedding + dense
    pos = image_pe.expand(b, -1, -1, -1)
    bb, c, h, w = src.shape
    hs, src_out = two_way_transformer(src.flatten(2).permute(0, 2, 1), pos.flatten(2).permute(0, 2, 1),
                                      tokens, MD + ".transformer", pack)

    obj_score_token_out = hs[:, 0, :]
    iou_token_out = hs[:, 1, :]
    mask_tokens_out = hs[:, 2:6, :]

    src_out = src_out.permute(0, 2, 1).view(bb, c, h, w)
    dc1 = F.conv_transpose2d(src_out, pack.get(MD + ".output_upscaling.0.weight"),
                             pack.get(MD + ".output_upscaling.0.bias"), stride=2)
    upscaled = F.gelu(ln2d(dc1, MD + ".output_upscaling.1", pack)
                      + conv(high_res[1], MD + ".conv_s1", pack))
    dc2 = F.conv_transpose2d(upscaled, pack.get(MD + ".output_upscaling.3.weight"),
                             pack.get(MD + ".output_upscaling.3.bias"), stride=2)
    upscaled = F.gelu(dc2 + conv(high_res[0], MD + ".conv_s0", pack))

    hyper = []
    for i in range(4):
        x = mask_tokens_out[:, i, :]
        p = f"{MD}.output_hypernetworks_mlps.{i}"
        for layer in range(3):
            x = lin(x, *pack.linear(f"{p}.layers.{layer}"))
            if layer < 2:
                x = F.relu(x)
        hyper.append(x)
    hyper_in = torch.stack(hyper, dim=1)

    masks = (hyper_in @ upscaled.flatten(2)).view(b, 4, upscaled.shape[2], upscaled.shape[3])

    def mlp3(x, prefix):
        for layer in range(3):
            x = lin(x, *pack.linear(f"{prefix}.layers.{layer}"))
            if layer < 2:
                x = F.relu(x)
        return x

    iou_pred = mlp3(iou_token_out, MD + ".iou_prediction_head")
    object_score_logits = mlp3(obj_score_token_out, MD + ".pred_obj_score_head")

    if multimask_output:
        return masks[:, 1:], iou_pred[:, 1:], mask_tokens_out[:, 1:], object_score_logits
    return masks[:, 0:1], iou_pred[:, 0:1], mask_tokens_out[:, 0:1], object_score_logits


def frame_features(pack, x):
    """Trunk plus the interactive FPN necks for a `[1, 3, 1008, 1008]` tensor (`sam.py:395-424`)."""
    x = conv(x, "detector.backbone.vision_backbone.trunk.patch_embed.proj", pack, stride=14)
    b, c, hp, wp = x.shape
    x = x.permute(0, 2, 3, 1).reshape(b, hp * wp, c)

    pos = pack.get("detector.backbone.vision_backbone.trunk.pos_embed")
    old = int(math.sqrt(pos.shape[1] - 1))
    spatial = pos[:, 1:].reshape(1, old, old, -1).permute(0, 3, 1, 2)
    tiles = hp // old + 1
    tiled = spatial.tile([1, 1, tiles, tiles])[:, :, :hp, :wp]
    x = x + tiled.permute(0, 2, 3, 1).reshape(1, hp * wp, -1)

    x = x.view(b, hp, wp, c)
    x = layernorm(x, "detector.backbone.vision_backbone.trunk.ln_pre", pack)

    cos_g, sin_g = rope_table(72, 72, 64, scale_pos=24.0 / 72.0)
    cos_w, sin_w = rope_table(24, 24, 64, scale_pos=1.0)
    for i in range(32):
        x = block(x, i, pack, cos_w, sin_w, cos_g, sin_g)

    trunk = x.permute(0, 3, 1, 2)
    return [fpn(trunk, f"detector.backbone.vision_backbone.interactive_convs.{i}", scale, pack)
            for i, scale in enumerate([4.0, 2.0, 1.0])]


def segment_mask_inputs(pack, frame, mask_logit):
    """`forward_segment(frame, mask_inputs=mask_logit)` -> `[1, 1, 1008, 1008]` high-res logits."""
    tracker_features = frame_features(pack, frame)
    high_res = tracker_features[:-1]
    backbone_feat = tracker_features[-1]

    no_mem = pack.get("tracker.model.interactivity_no_mem_embed")
    b, c, h, w = backbone_feat.shape
    flat = backbone_feat.flatten(2).permute(0, 2, 1) + no_mem
    backbone_feat = flat.view(b, h, w, c).permute(0, 3, 1, 2)

    image_pe = dense_pe(pack.get(PE + ".pe_layer.positional_encoding_gaussian_matrix"), GRID, GRID).to(DT)

    coords = torch.zeros(1, 1, 2, device=DEV, dtype=DT)
    labels = -torch.ones(1, 1, dtype=torch.int32, device=DEV)
    coarse = F.interpolate(mask_logit, size=(PROMPT_GRID, PROMPT_GRID), mode="bilinear",
                           align_corners=False, antialias=True)
    sparse, dense = prompt_encoder(pack, points=(coords, labels), masks=coarse)
    sparse = sparse.to(DT)
    dense = dense.to(DT)

    low_res, _, _, object_score = mask_decoder(pack, backbone_feat, image_pe, sparse, dense,
                                              high_res, False)
    is_obj = object_score > 0
    low_res = torch.where(is_obj[:, None, None], low_res,
                          torch.tensor(-1024.0, device=DEV, dtype=low_res.dtype))
    return F.interpolate(low_res, size=(IMAGE, IMAGE), mode="bilinear", align_corners=False)


def build_trunk(pack, image_path):
    """Trunk plus the interactive FPN necks (`sam.py:395-424`)."""
    from PIL import Image

    img = Image.open(image_path).convert("RGB").resize((1008, 1008), Image.BILINEAR)
    arr = torch.from_numpy(np.asarray(img)).float() / 255.0
    x = arr.permute(2, 0, 1).unsqueeze(0).to(DEV).to(DT)
    return frame_features(pack, x)


def main():
    pack_path, image_path, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
    pack = Pack(pack_path)

    box = point = mask_path = None
    for i, arg in enumerate(sys.argv[4:], start=4):
        if arg == "--box":
            box = [float(v) for v in sys.argv[i + 1].split(",")]
        elif arg == "--point":
            point = [float(v) for v in sys.argv[i + 1].split(",")]
        elif arg == "--mask":
            mask_path = sys.argv[i + 1]

    tracker_features = build_trunk(pack, image_path)
    high_res = tracker_features[:-1]
    backbone_feat = tracker_features[-1]

    # `interactivity_no_mem_embed` on the 72x72 level (`detector.py:548-554`).
    no_mem = pack.get("tracker.model.interactivity_no_mem_embed")
    b, c, h, w = backbone_feat.shape
    flat = backbone_feat.flatten(2).permute(0, 2, 1) + no_mem
    backbone_feat = flat.view(b, h, w, c).permute(0, 3, 1, 2)

    image_pe = dense_pe(pack.get(PE + ".pe_layer.positional_encoding_gaussian_matrix"), GRID, GRID).to(DT)

    num_pts = 0
    if mask_path is not None:
        raw = np.fromfile(mask_path, dtype=np.float32).reshape(1, 1, IMAGE, IMAGE)
        mask_logit = torch.from_numpy(raw).to(DEV).to(DT)
        coords = torch.zeros(1, 1, 2, device=DEV, dtype=DT)
        labels = -torch.ones(1, 1, dtype=torch.int32, device=DEV)
        coarse = F.interpolate(mask_logit, size=(PROMPT_GRID, PROMPT_GRID), mode="bilinear",
                               align_corners=False, antialias=ANTIALIAS)
        sparse, dense = prompt_encoder(pack, points=(coords, labels), masks=coarse)
    elif box is not None:
        corners = torch.tensor(box, device=DEV, dtype=DT).reshape(1, 2, 2)
        sparse, dense = prompt_encoder(pack, boxes=corners)
    elif point is not None:
        coords = torch.tensor(point, device=DEV, dtype=DT).reshape(1, 1, 2)
        labels = torch.ones(1, 1, dtype=torch.int32, device=DEV)
        num_pts = 1
        sparse, dense = prompt_encoder(pack, points=(coords, labels))
    else:
        raise SystemExit("one of --box / --point / --mask is required")

    multimask = 0 < num_pts <= 1
    # `forward_sam_heads` casts the prompt tensors to the backbone dtype (`tracker.py:300-302`).
    sparse = sparse.to(DT)
    dense = dense.to(DT)
    dump = os.environ.get("SAM3_DUMP_PROMPT")
    if dump:
        for suffix, tensor in (("dense", dense), ("sparse", sparse)):
            tensor.detach().float().cpu().numpy().astype("float32").tofile(f"{dump}_{suffix}.bin")
            print("dumped", f"{dump}_{suffix}.bin", tuple(tensor.shape))
    low_res, iou, _, object_score = mask_decoder(pack, backbone_feat, image_pe, sparse, dense, high_res, multimask)

    is_obj = object_score > 0
    low_res = torch.where(is_obj[:, None, None], low_res,
                          torch.tensor(-1024.0, device=DEV, dtype=low_res.dtype))
    high = F.interpolate(low_res, size=(IMAGE, IMAGE), mode="bilinear", align_corners=False)

    if multimask:
        best = torch.argmax(iou, dim=-1)
        idx = torch.arange(1, device=DEV)
        low_res = low_res[idx, best].unsqueeze(1)
        high = high[idx, best].unsqueeze(1)

    high.flatten().float().cpu().numpy().astype("float32").tofile(out_path)
    meta = {
        "iou": [float(v) for v in iou.flatten().tolist()],
        "object_score": float(object_score.flatten()[0]),
        "object_appearing": bool(is_obj.flatten()[0]),
        "mask_shape": list(high.shape),
        "mask_min": float(high.min()), "mask_max": float(high.max()),
        "mask_mean": float(high.mean()), "mask_positive_frac": float((high > 0).float().mean()),
    }
    json.dump(meta, open(out_path.rsplit(".", 1)[0] + ".json", "w"), indent=1)
    print("iou", meta["iou"], "object_score", f"{meta['object_score']:.6f}",
          "appearing", meta["object_appearing"])
    print("mask", tuple(high.shape), "min", f"{meta['mask_min']:.4f}",
          "max", f"{meta['mask_max']:.4f}", "positive", f"{meta['mask_positive_frac']:.4f}")
    print("wrote", out_path, high.numel())


if __name__ == "__main__":
    main()
