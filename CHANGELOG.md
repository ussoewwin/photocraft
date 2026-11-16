# Changelog

## v1.3.3

The SAM Selection tool now fetches its model itself: when `select.sam3` runs and the 1.74 GB HSWQ ConvRot INT8 pack is missing, the engine downloads it from Hugging Face into `plugins/sam3/models`, which every later run searches first. The download streams through a `.part` file with a `Content-Length` check and progress every 10 s, and every path is derived from the preferences plug-ins folder or the helper's own location — no machine's absolute path is written anywhere.

Release notes: [v1.3.3](https://github.com/ussoewwin/photocraft/releases/tag/v1.3.3)

## v1.3.2

The SAM Selection tool: SAM 3.1 open-vocabulary detection and interactive segmentation in pure Rust on CUDA, loading a 1.74 GB HSWQ ConvRot INT8 pack directly (no ONNX, no Python, no ComfyUI runtime). A click, a box drag or a text prompt selects an object, and the resident worker keeps the pack in VRAM so consecutive selections run in sub-second GPU time. Also carries the SAM Japanese labels in the tree and stops a font scan from blocking a UI frame.

Release notes: [v1.3.2](https://github.com/ussoewwin/photocraft/releases/tag/v1.3.2)

## v1.3.1

LaMa Cleaner runs on the GPU. The inpainting helper selects CUDA when the CUDA 13 and cuDNN 9 runtimes are present, falls back to DirectML and then CPU, preloads every CUDA/cuDNN library from the discovered folders so no PATH setup is needed, and keeps the model loaded in a resident worker, so a stroke pays only the inference: 87-95 ms on CUDA (measured), down from a 6.7-8.6 s model load per stroke.

Release notes: [v1.3.1](https://github.com/ussoewwin/photocraft/releases/tag/v1.3.1)
