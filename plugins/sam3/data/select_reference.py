"""Reference `SAM3_Detect` selection pipeline: detector dump -> `_refine_mask` -> masks.

Written against `comfy_extras/nodes_sam3.py`: `_refine_mask` (42-84) and the mask assembly in
`SAM3_Detect.execute` (193-255). The detector itself is already verified, so this script takes the
detections from a `detector_reference.py` dump and only exercises the refinement stage, which is
what `src/select.rs` implements.

Usage:
    python select_reference.py <pack> <image> <out.bin> --coarse <det.bin> [--threshold 0.5]
                               [--refine 2] [--individual]
"""

import json
import os
import sys

import numpy as np
import torch
import torch.nn.functional as F

sys.path.insert(0, "data")
from tracker_reference import DT, DEV, Pack, segment_mask_inputs  # noqa: E402


def read_dump(prefix):
    """Reads the `--detect-out` side-car plus the separate mask file."""
    meta = json.load(open(prefix.rsplit(".", 1)[0] + ".json"))
    raw = open(prefix, "rb").read()
    entries = {entry["name"]: entry for entry in meta}

    def read(name):
        entry = entries[name]
        start = entry["offset"] * 4
        count = entry["count"]
        return np.frombuffer(raw, dtype=np.float32, count=count, offset=start).copy()

    boxes = read("boxes").reshape(-1, 4)
    scores = read("scores_raw")
    mask_file = os.path.join(os.path.dirname(prefix), entries["masks"]["file"])
    shape = entries["masks"]["shape"]
    masks = np.fromfile(mask_file, dtype=np.float32).reshape(shape)
    return boxes, scores, torch.from_numpy(masks).to(device=DEV, dtype=DT)


def refine_mask(pack, orig_hwc, coarse, box, h, w, iterations):
    """`_refine_mask` (`nodes_sam3.py:42-84`)."""

    def coarse_fallback():
        up = F.interpolate(coarse.unsqueeze(0).unsqueeze(0), size=(h, w),
                           mode="bilinear", align_corners=False)[0]
        return (up > 0).float()

    if iterations <= 0:
        return coarse_fallback()

    pad_frac = 0.1
    x1, y1, x2, y2 = [float(v) for v in box]
    bw, bh = x2 - x1, y2 - y1
    cx1 = max(0, int(x1 - bw * pad_frac))
    cy1 = max(0, int(y1 - bh * pad_frac))
    cx2 = min(w, int(x2 + bw * pad_frac))
    cy2 = min(h, int(y2 + bh * pad_frac))
    if cx2 <= cx1 or cy2 <= cy1:
        return coarse_fallback()

    crop = orig_hwc[cy1:cy2, cx1:cx2, :3]
    crop_1008 = F.interpolate(crop.unsqueeze(0).movedim(-1, 1), size=(1008, 1008),
                              mode="bilinear", align_corners=False)
    crop_frame = crop_1008.to(device=DEV, dtype=DT)
    crop_h, crop_w = cy2 - cy1, cx2 - cx1

    mask_h, mask_w = coarse.shape[-2:]
    mx1, my1 = int(cx1 / w * mask_w), int(cy1 / h * mask_h)
    mx2, my2 = int(cx2 / w * mask_w), int(cy2 / h * mask_h)
    if mx2 <= mx1 or my2 <= my1:
        return coarse_fallback()
    mask_logit = coarse[..., my1:my2, mx1:mx2].unsqueeze(0).unsqueeze(0)

    for _ in range(iterations):
        coarse_input = F.interpolate(mask_logit, size=(1008, 1008), mode="bilinear", align_corners=False)
        mask_logit = segment_mask_inputs(pack, crop_frame, coarse_input)

    refined_crop = F.interpolate(mask_logit, size=(crop_h, crop_w), mode="bilinear", align_corners=False)
    full_mask = torch.zeros(1, 1, h, w, device=DEV, dtype=DT)
    full_mask[:, :, cy1:cy2, cx1:cx2] = refined_crop
    coarse_full = F.interpolate(coarse.unsqueeze(0).unsqueeze(0), size=(h, w),
                                mode="bilinear", align_corners=False)
    return ((full_mask[0] > 0) | (coarse_full[0] > 0)).float()


def main():
    pack_path, image_path, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
    coarse_prefix = None
    threshold = 0.5
    iterations = 2
    individual = False
    for index, arg in enumerate(sys.argv[4:], start=4):
        if arg == "--coarse":
            coarse_prefix = sys.argv[index + 1]
        elif arg == "--threshold":
            threshold = float(sys.argv[index + 1])
        elif arg == "--refine":
            iterations = int(sys.argv[index + 1])
        elif arg == "--individual":
            individual = True
    if coarse_prefix is None:
        raise SystemExit("--coarse <det.bin> is required")

    from PIL import Image

    pack = Pack(pack_path)
    img = Image.open(image_path).convert("RGB")
    native = torch.from_numpy(np.asarray(img)).float() / 255.0
    h, w = native.shape[0], native.shape[1]
    print("image", (w, h), "native")

    boxes_norm, scores_raw, masks = read_dump(coarse_prefix)
    probs = torch.from_numpy(scores_raw).sigmoid().numpy()
    order = np.argsort(-probs)
    keep = [int(i) for i in order if probs[i] > threshold]
    print("queries above", threshold, ":", len(keep))

    refined = []
    info = []
    for index in keep:
        box = [boxes_norm[index][0] * w, boxes_norm[index][1] * h,
               boxes_norm[index][2] * w, boxes_norm[index][3] * h]
        coarse = F.interpolate(masks[:, index:index + 1], size=(h, w), mode="bilinear",
                               align_corners=False)[0, 0]
        mask = refine_mask(pack, native, coarse, box, h, w, iterations)
        refined.append(mask)
        info.append({"score": float(probs[index]), "box": [float(v) for v in box]})
        print("  score %.6f box [%.1f, %.1f, %.1f, %.1f] coverage %.6f"
              % (probs[index], box[0], box[1], box[2], box[3], float(mask.mean())))

    if not refined:
        out = np.zeros((0, h, w), dtype=np.float32)
    elif individual:
        out = torch.stack(refined).cpu().numpy().astype(np.float32)
    else:
        union = refined[0]
        for mask in refined[1:]:
            union = torch.maximum(union, mask)
        out = union.unsqueeze(0).cpu().numpy().astype(np.float32)

    out.tofile(out_path)
    meta = {
        "masks": [list(m.shape) for m in (refined if individual else [out])],
        "coverage": [float(m.mean().item()) for m in refined] if individual else [float(out.mean())],
        "objects": info,
        "threshold": threshold,
        "refine_iterations": iterations,
        "individual": individual,
    }
    json.dump(meta, open(out_path.rsplit(".", 1)[0] + ".json", "w"), indent=1)
    print("wrote", out_path, out.size, "f32")


if __name__ == "__main__":
    main()
