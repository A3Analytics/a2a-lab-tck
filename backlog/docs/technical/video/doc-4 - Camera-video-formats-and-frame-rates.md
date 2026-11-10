---
id: doc-4
title: Camera video formats and frame rates
type: technical
created_date: '2026-11-04 21:16'
audience: technical
---

# Camera video formats and frame rates

## Scope and how to read the tables

Reference compiled November 4, 2026 for A2A-LAB camera ingest and H.264 encoding research. It covers ordinary USB webcams, classic ESP32-CAM/ESP32-S3/ESP32-P4 examples, Orbbec Gemini 335/335L, and RealSense D435/i and D455. It is a representative compatibility list, not an exhaustive list of every camera model.

A mode is a tuple of device, stream, transport, format, width, height, and frame interval. Never combine a camera's maximum resolution, maximum fps, and all listed formats into an assumed supported mode. Color, infrared, and depth have separate mode lists. SDK RGB output may be converted on the host rather than sent as RGB by the camera.

Evidence labels distinguish:

- **Observed**: driver-advertised modes queried on chupacabra earlier in this conversation; no capture/streaming test.
- **Specification**: manufacturer-documented modes, subject to firmware, USB speed, and simultaneous-stream limits.
- **Benchmark**: vendor test for a specified sensor or encoder; not a guaranteed end-to-end Wi-Fi or USB application rate.

Related research: [doc-2](</Users/dylangustaveson/code/a2a-lab-tck/backlog/docs/technical/video/doc-2 - Video-streaming-package-research-plan.md>) and the [doc-3 use-case checklist](</Users/dylangustaveson/code/a2a-lab-tck/backlog/docs/technical/video/doc-3 - OpenH264-current-video-use-case-checklist.md>).

## Common color formats and compressed encodings

Formats have no inherent fixed fps. The camera/driver/transport determines the supported resolution/rate combinations.

| Format or encoding | Kind | What it contains | Common integration consequence |
| --- | --- | --- | --- |
| JPEG / MJPEG / MJPG | Compressed images/video | One JPEG per frame; an MJPEG stream carries a sequence of these frames | Decode selected frames before raw-pixel H.264 encoding; preserve original JPEGs for stills where required |
| YUYV / YUY2 | Raw packed YUV 4:2:2 | 8-bit components, averaging 2 bytes/pixel | Convert packed 4:2:2 to I420 for OpenH264 |
| UYVY | Raw packed YUV 4:2:2 | Same subsampling as YUYV, different byte order | Use the correct byte order before conversion |
| NV12 | Raw semi-planar YUV 4:2:0 | Y plane plus interleaved UV; about 1.5 bytes/pixel without padding | Separate UV into planar U/V for I420 |
| I420 / YU12 | Raw planar YUV 4:2:0 | Separate Y/U/V planes; about 1.5 bytes/pixel without padding | Direct OpenH264 input when dimensions, strides, color range, and lifetime are valid |
| YV12 | Raw planar YUV 4:2:0 | Separate Y/V/U planes | Swap chroma planes for I420 |
| RGB8 / BGR8 / RGB565 | Raw processed color | RGB/BGR bytes or packed 16-bit color | Convert to the encoder's YUV layout; determine whether the SDK already performed conversion |
| Bayer RAW8/10/12 | Raw sensor mosaic | Color-filter samples, not a ready RGB picture | Requires sensor-specific demosaicing/color processing before normal color encoding |
| H.264 / AVC | Compressed video | Independently coded and/or predicted pictures plus codec configuration | Available from selected cameras/firmware; consider passthrough/remux if the bitstream meets seek, timing, and keyframe requirements |

Packed and planar layouts are defined in the [kernel packed-YUV reference](https://docs.kernel.org/userspace-api/media/v4l/pixfmt-packed-yuv.html) and [planar-YUV reference](https://docs.kernel.org/userspace-api/media/v4l/pixfmt-yuv-planar.html). H.264 is also a defined [V4L2 compressed format](https://docs.kernel.org/userspace-api/media/v4l/pixfmt-compressed.html). Format availability must still be enumerated on the device.

HTTP multipart, RTSP/RTP, HLS, and MP4 describe delivery/container arrangements; they do not by themselves identify the camera's codec or frame rate. An ESP32 HTTP MJPEG feed and a USB webcam's MJPG capture can both contain JPEG frames.

## Frame rates to expect

| Rate group | Examples in this reference | Interpretation |
| --- | --- | --- |
| Low-rate monitoring | 5, 6, 8, 10, 15 fps | Supported native rates vary; 5 fps application output does not require a 5 fps camera mode |
| Regular viewing | 25, 30 fps | 30 appears widely across the USB examples; the ESP32-S3 vendor capture example below is 25 |
| Faster color video | 60 fps | Usually depends strongly on resolution, compression, and USB generation |
| Faster depth/IR or reduced-resolution color | 90 fps | Documented for selected Orbbec and RealSense modes, not every stream |
| Specialized capture | 100 or 125 fps | Orbbec depth/IR reduced-height modes and an ESP monochrome benchmark; not ordinary full-resolution color video |

Other devices expose fractional intervals or additional video rates. Preserve the actual negotiated frame interval rather than assuming all cameras use integer multiples of 30.

## Ordinary USB webcams: observed and representative examples

| Camera / evidence | Camera output | Example supported resolution and fps |
| --- | --- | --- |
| EMEET SmartCam Nova 4K on chupacabra — Observed | MJPEG | 3840×2160 at 30; 1920×1080 at 30/60; 1280×720 at 30/60 |
| Same EMEET — Observed | YUYV | 640×480 or 640×360 at 30 |
| HP 5MP on chupacabra — Observed | MJPEG | 2560×1920, 1920×1080, or 1280×720 at 15/30 |
| Same HP — Observed | YUYV | 640×480 or 640×360 at 15/30 |
| Logitech BRIO — Specification | Manufacturer page gives output resolution/rate; enumerate exact format combinations on the device | 4K at 30; 1920×1080 at up to 60; 1280×720 at up to 90 |

The EMEET/HP results are recorded with device nodes and query method in [doc-3](</Users/dylangustaveson/code/a2a-lab-tck/backlog/docs/technical/video/doc-3 - OpenH264-current-video-use-case-checklist.md>). Both currently negotiate 480 Mb/s USB links on chupacabra. BRIO rates are from [Logitech's specifications](https://www.logitech.com/en-ca/products/webcams/brio-4k-hdr-webcam.960-001105.html?crid=34); they are not a claim that every rate is available in raw YUV.

## ESP32 camera examples

| Board/chip and sensor | Common output / processing | Resolution and fps evidence |
| --- | --- | --- |
| Classic ESP32-CAM + OV2640 | Usually JPEG frames / HTTP MJPEG; sensor also supports YUV and RGB formats | OV2640 sensor specification: 1600×1200 up to 15; SVGA up to 30. These are sensor ceilings, not a guarantee for the ESP32 board, PSRAM, JPEG settings, or Wi-Fi feed. |
| ESP32-S3 + OV5640 | JPEG capture | Vendor Benchmark: 1600×1200 at 25 fps |
| ESP32-S3 + SC030IOT | YUV422 or RGB565 capture | Vendor Benchmark: 640×480 at 30 fps |
| ESP32-S3 + SC031GS | Monochrome capture | Vendor Benchmark: 240×240 at 125 fps; not color webcam throughput |
| ESP32-S3R8 + software H.264 | OpenH264-derived encoder processing raw input | Vendor codec Benchmark: 320×240 YUYV input at 11.23 fps; no camera/Wi-Fi guarantee |
| ESP32-P4 + hardware H.264 | On-chip H.264 baseline encoding | Vendor codec Benchmark: 1920×1080 at 30 fps; sensor/network pipeline must also sustain it |

The [Espressif camera driver](https://github.com/espressif/esp32-camera) documents sensor formats and warns about raw YUV/RGB pressure on PSRAM, particularly with Wi-Fi. Its example recommends avoiding above-QVGA non-JPEG capture on the original ESP32. Do not generalize that limitation to every ESP32-S3/P4 board.

Sensor ceilings come from the [OmniVision OV2640 datasheet](https://dl.sipeed.com/MAIX/HDK/Chip_DS/OV2640-DATASHEET.pdf), page 1. Capture measurements are [Espressif's sensor/chip benchmarks](https://docs.espressif.com/projects/esp-techpedia/en/latest/esp-friends/solution-introduction/camera/dvp-mipi-csi-camera-solution.html). H.264 figures and hardware/software distinctions are from [ESP_H264](https://github.com/espressif/esp-h264-component/blob/master/esp_h264/README.md). An ordinary ESP32-CAM board should not be assumed to have the P4 hardware encoder.

## Orbbec Gemini: color versus depth and transport

| Model / stream | Transport / evidence | Format | Representative resolution and fps |
| --- | --- | --- | --- |
| Gemini 335/336 color | USB 3 — Specification | MJPEG / YUYV | 1920×1080: 6/10/15/30; 1280×720: 6/10/15/30/60 |
| Gemini 335L/336L color | USB 3 — Specification | MJPEG / YUYV | 1280×800 or 1280×720: 5/10/15/30/60; 640×480 adds 90 |
| Gemini 335L color on chupacabra | USB 2 — Observed | MJPEG | 1280×800: 8; 1280×720: 5/10/15 |
| Same current device | USB 2 — Observed | YUYV | 640×480 or 640×400: 5/10/15/30; 424×240 adds 60 |
| Gemini 335L/336L depth / IR | USB 3 — Specification | Y16 depth / Y8 IR | 1280×800: 5/10/15/30; 848×480 adds 60; 640×480 adds 60/90 |

Source: [Orbbec Gemini 330 datasheet v1.6](https://www.orbbec.com/wp-content/uploads/2025/06/Gemini-330-series-Datasheet-V1.6.pdf), printed pages 25–28. The 335L's color maximum differs from the 335's; do not assume the L model means 1080p color. USB 3 specifications are not the modes currently available on chupacabra's USB 2 link. SDK RGB/BGR representations may require host conversion.

## RealSense D400 examples

| Model / stream | Native format | USB 3 specification examples |
| --- | --- | --- |
| D435 / D435i color | YUY2 | 1920×1080 or 1280×720 at 6/15/30; 640×480 adds 60 |
| D435 / D435i depth / IR | 16-bit depth / Y8 | 1280×720 at 6/15/30; 848×480 adds 60/90 |
| D455 color | YUY2 | 1280×800 at 5/15/30; 640×480 adds 60 |
| D455 depth / IR | 16-bit depth / Y8 | 1280×720 at 5/15/30; 848×480 adds 60/90 |

Source: [RealSense D400 datasheet, March 2026 revision 023](https://www.realsenseai.com/wp-content/uploads/2026/03/RealSense-D400-Series-Datasheet-Mar-2026.pdf), printed pages 73–75. D435/i and D455 have different low-rate modes: 6 versus 5. Their depth fps is not their full-resolution color fps. USB 2, simultaneous streams, SDK conversion, exposure, and firmware can constrain results; enumerate the actual camera's profiles.

## Depth and infrared are separate inputs

Y8/GREY is monochrome intensity, Bayer is a color-filter mosaic, and Z16/Y16 depth represents numeric measurements with camera-specific interpretation. A colorized depth preview can be encoded as ordinary video, but that does not preserve the original depth measurements. Do not route depth payloads into an H.264 color encoder as though they were ordinary YUV.

The ordinary-camera color table therefore describes a different ingest concern from retaining depth, calibration, units, timestamps, and stream synchronization.

## Consequences for a 5 fps A2A-LAB pipeline

1. Enumerate format/resolution/frame-interval profiles first. Prefer a native 5 fps mode when suitable; otherwise select a supported rate and sample fresh frames by timestamps.
2. For MJPEG input, select frames before JPEG decode so discarded pictures do not incur conversion/encoding cost.
3. For YUYV/UYVY/NV12/RGB input, convert the selected frames to validated I420. Avoid assuming SDK RGB conversion is free or camera-native.
4. For device-produced H.264, inspect profile, SPS/PPS, frame boundaries, timestamps, and IDR spacing before deciding whether passthrough meets the current all-IDR seek/8x contract.
5. Treat capture fps, encoder throughput, retained fps, and player speed as different values. A camera advertising 30 fps does not force storage of 30 pictures per second.

No devices were reconfigured or streams started while creating this reference. Manufacturer figures were checked against primary documentation; no additional camera hardware or Wi-Fi throughput was benchmarked.
