# FSR 3.1 integration

The Vulkan renderer includes AMD FidelityFX Super Resolution 3.1.4 temporal upscaling through the vendored
FidelityFX SDK 1.1.4 and the FSR-Vulkan portable wrapper. FSR is off by default. When enabled, the 3D scene renders
below its output resolution and is reconstructed from jittered frames; the HUD is drawn at output resolution.

## Settings

The launcher exposes these settings in Graphics, and the runtime stores them in `mhgurecomp.ini`:

```ini
[Upscaler]
FSR = Off
FSRQuality = NativeAA
Sharpness = 0.8
ObjectMotion = On
Reactive = 0.9
AnimSurf = On
```

`FSRQuality` accepts `NativeAA`, `Quality`, `Balanced`, `Performance`, and `UltraPerformance`. The selected preset
sets the scene render-to-output ratio. Sharpness and reactive-mask strength range from 0 to 1. Object motion tracks
moving geometry in addition to camera movement; animated-surface detection helps reduce ghosting on texture-animated
surfaces. MSAA and FXAA are disabled while FSR is active.

If FSR's Vulkan prerequisites are unavailable, FSR remains off and the renderer scales the picture normally. The
requirements include subgroup operations, supported storage formats, and `shaderStorageImageWriteWithoutFormat`.
The output size is selected at startup; resizing the window later scales that output.

## Implementation

- `runtime/config.c` computes the scene and output resolutions, applies the FSR quality ratio, and disables MSAA/FXAA
  while FSR is enabled.
- `runtime/hle/camera_jitter.c` jitters the main camera projection and provides camera history for motion vectors.
- `runtime/nvn/vulkan/fsr.c` detects the scene/HUD boundary, prepares depth and motion inputs, records the FSR
  dispatch, and supplies output-size stand-in render targets for the HUD and present.
- `runtime/nvn/vulkan/exec.c` routes scene and HUD draws through the appropriate render sizes.
- `third_party/fsr-vulkan/` contains the portable wrapper and the required FidelityFX SDK source and generated
  shader headers; `third_party/ffx-api/` contains the FidelityFX API headers. Their upstream revisions, retained
  license files, and local changes are documented in
  `third_party/fsr-vulkan/UPSTREAM.md` and `third_party/fsr-vulkan/NOTICE.md`.

The runtime build does not invoke a shader compiler for FidelityFX shaders; their generated SPIR-V headers are
vendored. The renderer still uses its existing shader compiler for its own built-in preparation shaders.
