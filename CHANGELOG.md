# Changelog

## v1.3.1

LaMa Cleaner runs on the GPU. The inpainting helper selects CUDA when the CUDA 13 and cuDNN 9 runtimes are present, falls back to DirectML and then CPU, preloads every CUDA/cuDNN library from the discovered folders so no PATH setup is needed, and keeps the model loaded in a resident worker, so a stroke pays only the inference: 87-95 ms on CUDA (measured), down from a 6.7-8.6 s model load per stroke.

Release notes: [v1.3.1](https://github.com/ussoewwin/photocraft/releases/tag/v1.3.1)
