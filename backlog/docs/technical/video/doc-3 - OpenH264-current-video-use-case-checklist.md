---
id: doc-3
title: OpenH264 current video use-case checklist
type: technical
created_date: '2026-11-04 20:57'
audience: technical
---

# OpenH264 current video use-case checklist

## Answer and scope

OpenH264 appears capable of the H.264 encoding profile currently produced by OT-2: 1920×1080 JPEG input converted to 8-bit I420, nominally 5 encoded frames/s, baseline, no B frames, and an IDR for every stored sample. Source/API review supports that conclusion; it does not prove end-to-end replacement compatibility.

OpenH264's encoder accepts I420 rather than JPEG. The Rust wrapper includes RGB-to-I420 helpers, but replacing FFmpeg still requires JPEG decoding and correctly configured pixel/color conversion. Camera capture, DVR, CMAF/HLS packaging, MP4 export, authentication, and playback remain application responsibilities. Those use cases require the surrounding application and a conversion adapter. MJPEG and H.265 paths must remain supported by their existing components.

The highest-risk checks are all-IDR output without frame skipping, correct ownership and timestamps of complete access units, playback of old x264 recordings after the switch, and fully static musl release builds. Do not call the migration compatible until the relevant checks below have observed evidence.

This desk review was performed November 4, 2026 against OT-2 HEAD b5d29c6, dev-kit HEAD 1ed5b93, and TCK HEAD 559b9da. Both sibling worktrees were clean when inspected. This is a current source inventory, not an assertion that every existing path works. Existing tests were inspected, not rerun. No dependencies were installed and no prototype, capture/streaming, browser, or performance experiment was run for this checklist. The later read-only camera format query is recorded below.

This supplements [doc-2: Video streaming package research plan](</Users/dylangustaveson/code/a2a-lab-tck/backlog/docs/technical/video/doc-2 - Video-streaming-package-research-plan.md>). All boxes intentionally remain unchecked: API availability and existing test names do not establish that OpenH264 passes an application use case.

## Assessment labels

- **Codec**: OpenH264 exposes the relevant capability; the application output still needs verification.
- **Adapter**: integration/conversion code must preserve or supply this behavior.
- **App**: existing application behavior outside the encoder; regression check.
- **Verify**: desk research cannot establish the required runtime result.
- **Limit**: an existing limitation or mismatch, not a feature proven to work today.

The list has 124 checks. BUILD-10 is the previously requested additional Windows/Intel Mac portability scope, not a current OT-2 release target. MIG checks exercise a transition between implementations rather than introducing new product features.

## Source and test map

| Area | Authoritative current implementation and inspected tests |
| --- | --- |
| Source capture and still reuse | OT-2 [catalog.rs](</Users/dylangustaveson/code/a2a-lab-ot2/src/videos/catalog.rs:255>), [external.rs](</Users/dylangustaveson/code/a2a-lab-ot2/src/videos/external.rs>), [camera.rs](</Users/dylangustaveson/code/a2a-lab-ot2/src/camera.rs:28>); [camera_warm_sessions.rs](</Users/dylangustaveson/code/a2a-lab-ot2/tests/camera_warm_sessions.rs>), [external_camera.rs](</Users/dylangustaveson/code/a2a-lab-ot2/tests/external_camera.rs>), [video_serve.rs](</Users/dylangustaveson/code/a2a-lab-ot2/tests/video_serve.rs>) |
| Encoding | OT-2 [h264.rs](</Users/dylangustaveson/code/a2a-lab-ot2/src/videos/h264.rs>), including its two synthetic NAL-grouping tests |
| DVR and catalog | OT-2 [store.rs](</Users/dylangustaveson/code/a2a-lab-ot2/src/videos/dvr/store.rs>), [provider.rs](</Users/dylangustaveson/code/a2a-lab-ot2/src/videos/dvr/provider.rs>), [video_dvr.rs](</Users/dylangustaveson/code/a2a-lab-ot2/tests/video_dvr.rs>); dev-kit [dvr_store_contract.rs](</Users/dylangustaveson/code/a2a-lab-dev-kit-rs/tests/support/dvr_store_contract.rs>) |
| CMAF, HLS, export | Dev-kit [cmaf.rs](</Users/dylangustaveson/code/a2a-lab-dev-kit-rs/src/videos/cmaf.rs>), [packager.rs](</Users/dylangustaveson/code/a2a-lab-dev-kit-rs/src/videos/packager.rs>), [backend_seam_matrix.rs](</Users/dylangustaveson/code/a2a-lab-dev-kit-rs/tests/support/backend_seam_matrix.rs>) |
| Playback speeds | Dev-kit [speed.rs](</Users/dylangustaveson/code/a2a-lab-dev-kit-rs/web-ui/src/videos/speed.rs>), [video_trick_play_8x.rs](</Users/dylangustaveson/code/a2a-lab-dev-kit-rs/tests/video_trick_play_8x.rs>), [web_ui_video_speed.rs](</Users/dylangustaveson/code/a2a-lab-dev-kit-rs/tests/web_ui_video_speed.rs>) |
| Browser playback and UI | Dev-kit [hls_playback.rs](</Users/dylangustaveson/code/a2a-lab-dev-kit-rs/web-ui/src/shell/hls_playback.rs>), [hls.rs](</Users/dylangustaveson/code/a2a-lab-dev-kit-rs/web-ui/src/videos/hls.rs>); web_ui_hls_mse, web_ui_video_playback, web_ui_video_transport, web_ui_videos, web_ui_videos_closeout, web_ui_videos_e2e, web_ui_videos_interactions, and web_ui_videos_visual tests |
| Protocol and gateway contracts | Dev-kit [provider.rs](</Users/dylangustaveson/code/a2a-lab-dev-kit-rs/src/videos/provider.rs>), [http.rs](</Users/dylangustaveson/code/a2a-lab-dev-kit-rs/src/videos/http.rs>), [media_asset.rs](</Users/dylangustaveson/code/a2a-lab-dev-kit-rs/src/videos/media_asset.rs>); a2a_videos, mcp_videos, sila_videos, video_gateway, video_contracts, video_tests, video_timeline_parity, video_outcome_parity, video_legacy_parity, video_parity, and tck_malformed_videos tests |
| Release and deployment | OT-2 [release.yml](</Users/dylangustaveson/code/a2a-lab-ot2/.github/workflows/release.yml:21>), [Dockerfile](</Users/dylangustaveson/code/a2a-lab-ot2/agent/Dockerfile>), [serve.rs](</Users/dylangustaveson/code/a2a-lab-ot2/src/videos/serve.rs>), [agent-media-smoke.sh](</Users/dylangustaveson/code/a2a-lab-ot2/.mise/scripts/agent-media-smoke.sh>) |
| TCK | [doc-1](</Users/dylangustaveson/code/a2a-lab-tck/backlog/docs/reference/compliance/doc-1 - A2A-LAB-TCK-profile.md:17>); optional video basic/full cases and four producer-to-consumer transport combinations |

## Observed chupacabra camera formats

Read directly from chupacabra through V4L2 QUERYCAP, ENUM_FMT, ENUM_FRAMESIZES, and ENUM_FRAMEINTERVALS on November 4, 2026. The query enumerated capabilities only: no format change, stream startup, capture, service restart, or dependency installation. These are the currently advertised driver modes, not proof of streaming behavior or every possible vendor-SDK mode. Device numbers are the observed assignments.

| Color camera | Device | Raw YUV advertised | Relevant MJPEG modes |
| --- | --- | --- | --- |
| EMEET SmartCam Nova 4K | /dev/video4 | YUYV 4:2:2, 640×480 or 640×360 at 30 fps | 1920×1080 at 30/60 fps; 3840×2160 at 30 fps |
| HP 5MP Camera | /dev/video0 | YUYV 4:2:2, 640×480 or 640×360 at 15/30 fps | 1280×720 and 1920×1080 at 15/30 fps; 2560×1920 at 15/30 fps |
| Orbbec Gemini 335L color | /dev/video12 | YUYV 4:2:2, 424×240 at 5/10/15/30/60 fps; 640×400 or 640×480 at 5/10/15/30 fps | 1280×720 at 5/10/15 fps; 1280×800 at 8 fps; matching lower-resolution modes |

Other enumerated capture nodes were HP IR GREY (/dev/video2), Orbbec Z16 depth (/dev/video6), GREY (/dev/video8), and BA81 Bayer (/dev/video10). None of the enumerated capture formats was I420, NV12, or H.264.

For the current 1920×1080 external-camera use case, raw YUV is not advertised by the EMEET driver. Keeping that resolution therefore retains the MJPEG-decode requirement through the current V4L2 interface. At the lower raw resolutions, YUYV avoids JPEG decoding but still needs packed 4:2:2 to planar I420 conversion for OpenH264. See the [kernel packed YUV format definition](https://docs.kernel.org/userspace-api/media/v4l/pixfmt-packed-yuv.html).

## Support summary

| Requirement | Desk-research result |
| --- | --- |
| Baseline H.264, no B frames, every picture independently decodable | Encoder profile/intra-period/force-IDR controls are present; verify actual all-IDR output and disable default skipping. |
| Current external-camera dimensions and nominal 5 fps | 1920×1080 is within the wrapper's declared size limits; target throughput and quality are unmeasured. |
| Complete pictures plus SPS/PPS | Per-frame bitstream, frame-type, layer, and NAL APIs exist; adapter must produce the current initialization/sample formats and own the bytes. |
| JPEG/MJPEG input | No JPEG decoder in this encoder. Wrapper conversion helpers do not eliminate the JPEG decoding requirement. |
| HLS, DVR, arbitrary seek, MP4 export, and 8x | Existing application components can consume matching encoded samples; compatibility depends on all-IDR and correct timing/init handling. No integration result yet. |
| Stills, MJPEG playback, H.265 contracts | Preserve existing paths; these are outside OpenH264's H.264 encoder role. |
| Old recordings and rollback | Unresolved application-level initialization-version risk; one current track-level SPS/PPS value is reused for historical samples. |
| Existing GNU Linux / Apple Silicon release targets | Upstream wrapper documents relevant targets; actual application encoding/linkage must be exercised. |
| Existing fully static musl releases | Build logic recognizes musl, but no published target-matrix or application-runtime evidence reviewed proves the static contract. |
| Additional Windows / Intel Mac releases | Wrapper documents platform support; these are additions to the current application release matrix. |

## 1. Sources, capture, and still images

- [ ] **CAP-01 — Video disabled.** With neither source configured, startup reports videos off, opens no device, and creates no encoder. **App**
- [ ] **CAP-02 — Native external camera.** Open exact MJPEG 1920×1080 at a 30 fps camera mode, discard 30 settling frames, then continuously capture. This is camera mode, not a requirement to encode 30 fps. **App**
- [ ] **CAP-03 — Configured sampling rate.** Preserve default 5 fps and positive configurable pump rates. Resolve the current mismatch: the FFmpeg command hardcodes 5 fps while disk sample duration uses configured fps. **Adapter, Verify, Limit**
- [ ] **CAP-04 — Second webcam.** Preserve source camera using the closest 1280×720 MJPEG mode at 5 fps. It currently stores MJPEG and does not invoke the H.264 encoder. **App**
- [ ] **CAP-05 — Injected test sources.** External and webcam LiveJpegSource injections remain usable without hardware. They currently bypass native H.264 encoding; add an explicit encoder seam before treating injection tests as encoder coverage. **App, Limit**
- [ ] **CAP-06 — Dual-source operation.** Run external-camera and camera together; one source's failure or slow encoder does not stop the other's capture, stills, or playback. **App, Verify**
- [ ] **CAP-07 — External still reuse.** A still request reuses the captured JPEG without opening the device again; returned/stored image bytes remain identical and independent of lossy H.264 output. **App**
- [ ] **CAP-08 — Webcam still reuse.** Preserve remembered webcam JPEG reuse, same-byte image retrieval, and independence from the external-camera still path. **App**
- [ ] **CAP-09 — Source configuration and discovery.** Preserve indices, names, descriptions, dimensions, fps, stable source IDs, and paginated listing; invalid configuration fails before robot contact or camera opening. **App**
- [ ] **CAP-10 — Missing or unavailable device.** Missing exact format, permission denial, backend failure, disabled camera, and disconnect produce actionable unavailable behavior and allow the current reconnect path. **App, Verify**
- [ ] **CAP-11 — Invalid input limits.** Empty, oversized, malformed, or dimensionless JPEG frames do not reach unsafe codec calls, enter DVR as valid media, or leave partial still images. **Adapter**
- [ ] **CAP-12 — OT-2 onboard camera boundary.** Preserve still capture separately. Robot-server GET /camera/stream returns metadata and is rejected as video; POST /camera/picture polling must not become a video source. **App**

## 2. JPEG conversion and input buffers

- [ ] **IN-01 — JPEG decoding.** Supply a JPEG decoder; OpenH264 accepts raw YUV, not JPEG/MJPEG input. Replacing the encoder alone does not replace this FFmpeg responsibility. **Adapter**
- [ ] **IN-02 — Preserve original JPEGs.** Encoding must not mutate, discard, or substitute reconstructed frames for JPEGs used by live MJPEG, warm still capture, or image history. **Adapter, App**
- [ ] **IN-03 — I420 plane layout.** Produce 8-bit planar Y, U, V with correct chroma subsampling, strides, lengths, and plane ordering. Current output is yuv420p. **Codec, Adapter**
- [ ] **IN-04 — Color interpretation.** Match conversion matrices, JPEG full-range handling, encoded range, and SPS VUI; inspect black/white levels, lab-label colors, and fine colored features. **Codec, Adapter, Verify**
- [ ] **IN-05 — 1080-line cropping.** Encode actual 1920×1080; verify macroblock padding/crop metadata yields 1080 displayed lines rather than 1088 or a fallback size. **Codec, Adapter, Verify**
- [ ] **IN-06 — Actual dimensions.** Feed decoded dimensions into the encoder and derive CMAF width/height and browser codec metadata from its real SPS. Avoid the packager's 640×480 fallback. **Codec, Adapter**
- [ ] **IN-07 — Defensive dimension handling.** Reject or explicitly handle odd, zero, excessive, changing, or inconsistent dimensions before constructors that can panic; do not silently stretch or crop input. **Adapter, Verify**
- [ ] **IN-08 — Buffer ownership.** YUV planes remain valid during encoding, and encoded NALs are copied/owned before the next encode call or DVR queue transfer invalidates borrowed buffers. **Adapter**

## 3. H.264 bitstream contract

- [ ] **ENC-01 — Baseline profile.** Explicitly select compatible baseline H.264, single spatial/temporal layer, and the required level. Do not rely on unspecified profile defaults. **Codec, Adapter**
- [ ] **ENC-02 — Every sample is an IDR.** Configure intra-frame period 1 and verify every emitted picture contains an IDR NAL, including first frame, reconnect, and encoder restart. The current -g 1 contract underpins independent seek and decimation. **Codec, Verify**
- [ ] **ENC-03 — No reordered frames.** Preserve no B frames and presentation/decode order parity. Current CMAF timestamps have no composition-offset handling. **Codec, Adapter**
- [ ] **ENC-04 — No silent frame skipping.** Disable codec frame skipping and explicitly handle empty/skipped output. The wrapper defaults to skip_frames=true. **Codec, Adapter, Verify**
- [ ] **ENC-05 — One picture per stored sample.** Collect all video-layer slice NALs belonging to a picture, including multi-slice output, into one access unit. Never store one sample per NAL. **Codec, Adapter**
- [ ] **ENC-06 — Sample byte format.** Convert Annex B start-coded NALs to the current 4-byte big-endian length-prefixed sample format, handling both start-code lengths and checking boundaries. **Adapter**
- [ ] **ENC-07 — Initialization bytes.** Extract valid SPS and PPS into Annex B codec initialization; ensure avcC contains parameter sets referenced by the stored samples. **Codec, Adapter**
- [ ] **ENC-08 — Metadata-only encoder output.** SPS/PPS/AUD/SEI-only output must not be mistaken for a stored picture; preserve the current separation of parameter sets and VCL payload. **Adapter**
- [ ] **ENC-09 — Threading and slices.** Start with the current one-thread behavior or justify a measured change. Preserve complete access units if native threading or slice settings differ. **Codec, Adapter, Verify**
- [ ] **ENC-10 — Quality and bitrate.** Choose rate control deliberately; there is no direct equivalence to the existing x264 veryfast/zerolatency configuration. Do not assume the wrapper's 120,000 bps default is suitable for 1080p lab footage. **Codec, Verify**
- [ ] **ENC-11 — Real profile/level signaling.** Browser MIME probing must use the generated avcC profile/compatibility/level bytes; successful decoding must not depend on an inaccurate hardcoded codec string. **Adapter, Verify**
- [ ] **ENC-12 — Valid output before publication.** Do not publish encoded readiness or write an encoded sample until initialization and nonempty picture data are available. Decode independent sampled output to confirm the declared keyframe flags. **Adapter, Verify**

## 4. Timing, failure, and long-running capture

- [ ] **RUN-01 — Frame-to-timestamp association.** Pair encoded output with its input JPEG's capture timestamp. A synchronous encoder must not inherit the current previous-picture queue delay or attach the next JPEG to the previous timestamp. **Adapter**
- [ ] **RUN-02 — Timestamp precision.** Retain microsecond UTC timestamps in DVR; OpenH264's millisecond timestamp API must not quantize the application's timeline or overflow during long runs. **Codec, Adapter**
- [ ] **RUN-03 — Missing and delayed frames.** Preserve capture gaps and sample durations rather than manufacturing continuous media across outages, discarded input, or dropped disk jobs. **App, Adapter**
- [ ] **RUN-04 — Default frame budget.** JPEG decode, conversion, encode, copies, and queueing together sustain 1920×1080 at nominal 5 fps, allowing the default 200 ms budget without starving other service work. **Verify**
- [ ] **RUN-05 — First-frame latency.** A first usable IDR and initialization reach the DVR/live path promptly; do not add the existing stream-boundary parser's extra-frame wait to complete frame output. **Codec, Adapter, Verify**
- [ ] **RUN-06 — Final-frame delivery.** Define and verify what happens to the final submitted picture on stop/error. The current subprocess is killed on drop and does not provide a successful drain guarantee; avoid claiming this is already covered. **Adapter, Verify, Limit**
- [ ] **RUN-07 — Camera reconnect.** After source loss/reopen, reset encoder/timestamp state as needed and resume with valid initialization and an independently decodable picture. **App, Adapter, Verify**
- [ ] **RUN-08 — Encoder failure recovery.** Translate initialization/encode errors into existing unavailable behavior, clear stale queued timestamps/parameter sets, and recreate state without mixing sessions. **Adapter, Verify**
- [ ] **RUN-09 — Disk backpressure.** Preserve the bounded 64-job disk queue and nonblocking capture behavior; a full queue may drop recording samples, but must not leak memory or silently hide resulting coverage gaps. **App, Verify**
- [ ] **RUN-10 — Shutdown and device ownership.** Drop/SIGTERM stops capture, releases cameras and native encoder allocations, and does not leak worker threads. Removing fork/pre_exec must preserve exclusive camera ownership. **Adapter, App, Verify**
- [ ] **RUN-11 — Source/viewer concurrency.** Encoder state belongs to the source; multiple viewers, clips, and protocol clients reuse recorded samples instead of starting duplicate encoders or corrupting each other's buffers. **App, Adapter, Verify**
- [ ] **RUN-12 — Continuous service and fault containment.** Check bounded memory/CPU over sustained capture, temporary idle periods, repeated restart, and malformed input. An in-process native failure no longer has the subprocess's crash isolation. **Verify**

## 5. Indexed disk DVR

- [ ] **DVR-01 — Encoded append/read.** H.264 samples, keyframe flags, source/track IDs, capture times, initialization, and byte lengths round-trip through SQLite metadata and minute chunk files. **App, Adapter**
- [ ] **DVR-02 — Retention policy.** Preserve configurable disk retention, three-day default, hourly cleanup, and the separate five-second default live-memory window. **App**
- [ ] **DVR-03 — Source/track isolation.** Recording, lookup, cleanup, and a failure on one source never delete or reinterpret another source's MJPEG/H.264 media. **App**
- [ ] **DVR-04 — Restart/crash recovery.** Recover finalized chunks, remove partial payloads, truncate unindexed tails, and keep accurate retained coverage after restart. **App**
- [ ] **DVR-05 — Integrity checks.** Reject corrupted/mismatched checksums, truncated byte ranges, missing payloads, and invalid sample lengths without treating them as playable frames. **App**
- [ ] **DVR-06 — Efficient window reads.** Preserve paginated large-range reads and indexed frame byte-range access; encoding changes must not require scanning/decoding the entire retention history for each request. **App**
- [ ] **DVR-07 — Concurrent read/write.** Playback and timeline reads remain responsive while capture appends and minute rotation/cleanup runs. **App, Verify**
- [ ] **DVR-08 — Legacy persistence.** Continue reading existing per-frame files and current chunk storage without a destructive migration or deleting retained footage. **App, Verify**
- [ ] **DVR-09 — Invalid storage.** Empty/unwritable storage roots and invalid retention fail through existing configuration/storage errors before camera startup where currently enforced. **App**
- [ ] **DVR-10 — Disk write failure.** Logging/recovery and source availability remain honest when a job fails after capture. Successful live publication must not be used as proof of successful disk persistence. **App, Verify**

## 6. Catalog, frame lookup, and playback sessions

- [ ] **CAT-01 — Catalog operations.** Preserve list_video_sources pagination, stable IDs, dimensions, descriptions, and source availability with video off, external only, webcam only, and both sources. **App**
- [ ] **CAT-02 — Timeline coverage.** Return ordered half-open logical intervals and genuine gaps, correct oldest/newest timestamps, and no storage-chunk boundaries in public metadata. **App**
- [ ] **CAT-03 — Live-edge follow.** After live capture and H.264 storage are ready, follow_video selects encoded HLS; sources without a ready encoded track preserve their current MJPEG behavior. **App, Adapter**
- [ ] **CAT-04 — Anchored follow/rebase.** Opening at a retained timestamp and rebasing an existing session preserve the requested source, anchor, relative session root, and timeline semantics. **App**
- [ ] **CAT-05 — Nearest JPEG frame lookup.** Preserve nearest/tie-to-earlier/tolerance behavior for live-memory and retained MJPEG frames. Current OT-2 historical frame lookup does not decode the H.264 track into JPEGs; adding OpenH264 encoding does not fill that gap. **App, Limit**
- [ ] **CAT-06 — Finite requested/effective range.** Clip metadata retains requested and available effective half-open ranges, tolerates retained gaps as designed, and rejects reversed/empty input. **App**
- [ ] **CAT-07 — Long retained clips.** Disk-backed H.264 clips may span long retained ranges. Preserve the separate legacy MemoryVideos MJPEG 60-second request limit instead of assuming all providers have the same limit. **App**
- [ ] **CAT-08 — Session expiration.** Preserve opaque clip/follow IDs, current TTL behavior, clip access renewal where implemented, expired-session errors, and cleanup without leaking retained buffers. **App**
- [ ] **CAT-09 — Capability accuracy.** Verify source media/capability metadata against selected playback sessions. Current OT-2 source construction advertises MJPEG while readiness can select an H.264 follow; do not infer capability parity from a successful encoder call. **App, Limit**
- [ ] **CAT-10 — Availability and validation.** Missing source, no recording, coverage gap, out-of-retention requests, unavailable live source, malformed IDs/timestamps, and empty retention preserve their existing typed outcomes. **App**

## 7. CMAF, HLS, and finite exports

- [ ] **PKG-01 — Initialization segment.** Emit a decodable init.mp4 with avc1/avcC, correct SPS/PPS, dimensions, and sample-length format from OpenH264 output. **Adapter, Verify**
- [ ] **PKG-02 — Ordinary fragments.** Existing length-prefixed samples produce complete moof/mdat media fragments and .m4s assets that independent decoders accept. **Adapter, Verify**
- [ ] **PKG-03 — Sync flags.** Every sample marked independently decodable by the current CMAF writer really is an IDR. Its writer currently sets sync/no-dependency flags for all samples; a default inter-frame GOP is unsafe here. **Codec, Adapter, Verify**
- [ ] **PKG-04 — Grouping.** Preserve five-second absolute segment boundaries, multiple frame fragments per segment, short first/last segments, and the existing treatment of incomplete live segments. **App**
- [ ] **PKG-05 — Timestamps and refresh stability.** Preserve microsecond timescale, decode times, durations, stable segment names, and session timeline origins across live refresh/rebase. **App, Adapter**
- [ ] **PKG-06 — Playback windows.** Preserve the default 120-second bounded packaging window and seeking across longer retained history through rebasing. A playback window is not the disk-retention limit. **App**
- [ ] **PKG-07 — Gaps and wall clock.** Preserve program date-time and discontinuity signaling, coverage gaps, and independently decodable restart points after outages. **App, Adapter, Verify**
- [ ] **PKG-08 — Finite versus live manifests.** Finite HLS ends with ENDLIST; live EVENT manifests remain refreshable without ENDLIST, including trick-play manifests. **App**
- [ ] **PKG-09 — Seek and clip boundaries.** Start playback/exports at arbitrary retained frames and half-open clip boundaries without requiring unseen previous reference frames. Exercise first/last frame and seek across gaps. **Codec, App, Verify**
- [ ] **PKG-10 — Finite MP4.** Export actual decodable video/mp4 without running an encoder again; verify duration, displayed dimensions, ordering, and the current export policy for concatenating samples across timestamp gaps. **App, Verify**
- [ ] **PKG-11 — Other codec/media paths.** Preserve existing MJPEG finite multipart export and H.265 storage/packaging contracts. OpenH264 does not encode MJPEG or H.265, and OT-2 does not currently have an H.265 capture encoder. **App**
- [ ] **PKG-12 — Asset identity and completeness.** Session-relative init, normal/trick manifests, and segment assets retain correct content types; unknown, expired, or vanished segments return the intended error rather than partial successful media. **App, Verify**

## 8. Playback rates and 8x trick play

- [ ] **RATE-01 — Ordinary rates.** Recorded HLS plays at 0.5x, 1x, 2x, and 4x through the existing playbackRate policy; decoded motion and elapsed wall-clock mapping remain correct. **App, Verify**
- [ ] **RATE-02 — All-IDR decimation.** The current 8x rendition can select every eighth picture, produce fewer samples, and decode each independently. Preserve the separate index-8x.m3u8 path and compressed media timing. **Codec, App, Verify**
- [ ] **RATE-03 — Keyframe-only alternative and fallback.** Retain existing sparse-keyframe handling, insufficient_keyframes/no_sample_reduction reasons, and recoverable fallback when no usable trick rendition exists. **App**
- [ ] **RATE-04 — 8x playback timing.** Dedicated HLS trick media uses browser playbackRate 1.0 while mapping its progress to 8x source time; it must not accidentally run at 64x or show normal-speed footage. **App, Verify**
- [ ] **RATE-05 — State preservation.** Switching speed preserves playing/paused state and wall-clock position; source change and retained seeking cannot reuse stale encoded sessions. **App, Verify**
- [ ] **RATE-06 — Live speed lock.** Follow-live and playheads within the existing two-second live-edge threshold lock to 1x; moving back into retained history restores eligible speed controls. **App**
- [ ] **RATE-07 — Gapped/refreshing trick media.** Normal and 8x live refresh, rebasing, program date-time, and discontinuities remain consistent across gaps and newly arriving recordings. **App, Verify**
- [ ] **RATE-08 — MJPEG speed parity.** Existing MJPEG 0.5x/1x/2x/4x/8x scheduling remains usable; speed metadata and paths must not bypass the media gateway or encode credentials. **App**

## 9. Browser player and reader

- [ ] **UI-01 — Actual picture decoding.** Display changing real OpenH264 pixels in the agreed browser matrix. Chromium test harness behavior alone does not establish Firefox/Safari or every desktop decoder combination. **Verify**
- [ ] **UI-02 — MSE startup.** Probe the actual avcC, choose a supported MIME, append initialization before fragments, and show correct video dimensions without false codec support. **App, Adapter, Verify**
- [ ] **UI-03 — Live viewing.** Start near the current live edge, append newly captured media during manifest refresh, and avoid skipping recordings that arrived during a slow fetch. **App, Verify**
- [ ] **UI-04 — Recorded navigation.** Source selection, timeline click/drag, date/time range input, anchored follow, rebasing, recorded playback, and jump-to-newest preserve intended wall-clock position. **App**
- [ ] **UI-05 — Pause/resume/end.** Pause stops visual movement at the selected frame; resume advances; finite playback ends at its boundary without continuing beyond the requested recording. **App, Verify**
- [ ] **UI-06 — Gap recovery.** Skip missing/unavailable segments and genuine coverage gaps according to current UI policy, while reporting the gap rather than freezing indefinitely. **App, Verify**
- [ ] **UI-07 — Useful failure and retry.** Unsupported codec, malformed init, decode/append/network errors, no recording, offline source, and expired handle produce recoverable UI state and retry controls. **App, Verify**
- [ ] **UI-08 — Cancellation and stale work.** Source/range changes, stopping follow, navigation, and disconnect cancel requests and reject late responses from older generations. **App**
- [ ] **UI-09 — Resource cleanup.** Release MSE/SourceBuffer state, object URLs, listeners, queues, and pending requests on teardown or repeated start/stop. **App, Verify**
- [ ] **UI-10 — Reader usability/privacy.** Preserve details/ranges, keyboard controls and focus, status announcements, reduced motion, theme/viewport layout, and current storage rules excluding bearer/media bytes. **App**

## 10. Protocols, authenticated gateway, and conformance

- [ ] **API-01 — A2A control plane.** List, frame-at, clip, timeline, and follow operations preserve typed requests/results, metadata-only sessions, and existing image-result behavior. **App**
- [ ] **API-02 — MCP parity.** The same five operations preserve A2A-equivalent outcomes, source IDs, ranges, codec declarations, and optional 8x metadata. **App**
- [ ] **API-03 — SiLA boundary.** Preserve source/frame/clip/timeline behavior and finite binary export, including encoded MP4 where the provider exposes it. SiLA's current absence of live follow remains explicit. **App**
- [ ] **API-04 — TCK handoffs.** Preserve optional video capability declaration, basic boundary/error cases, and A2A→A2A, MCP→MCP, A2A→MCP, MCP→A2A source-ID handoffs. TCK metadata success is not decoding evidence. **App**
- [ ] **API-05 — Authentication on all assets.** Secured clip/live roots, normal/trick manifests, init segments, and media segments require the configured bearer header, including redirects/errors where relevant. **App**
- [ ] **API-06 — Header and request policy.** Preserve same-origin gateway requests, omitted ambient credentials, cache=no-store, response no-store/nosniff, and no secrets in URLs or playlist text. **App**
- [ ] **API-07 — Path containment.** Reject absolute/upstream URLs, traversal, encoded traversal/separators, malformed relative asset names, and credential-bearing query parameters. **App**
- [ ] **API-08 — Typed failures.** Keep no_recording, recording_gap, clip_expired, live_unavailable, invalid, not-found, and unavailable distinctions stable across protocol adapters and gateway responses. **App**
- [ ] **API-09 — Session ownership.** Preserve source/session matching, unknown/expired ID errors, and the current MJPEG follow single-open/double-open rejection behavior. **App**
- [ ] **API-10 — Local and remote asset seams.** Preserve LocalMediaAssetProvider and remote-provider parity, sanitize upstream failures, and use the in-process service for local delivery rather than a loopback client. **App**
- [ ] **API-11 — Multipart fallback lifecycle.** MJPEG clips are finite, ordered, and closed; MJPEG live streams deliver post-open frames, remain open when idle, bound buffering, and stop on disconnect/unavailability. **App**
- [ ] **API-12 — Report privacy.** Keep clip bytes, live frame bytes, playlists, and credentials out of TCK compliance reports and metadata-only session serialization. **App**

## 11. Current release/deployment matrix and requested portability

- [ ] **BUILD-01 — Linux x86_64 GNU.** Build and run the real encoding path on x86_64-unknown-linux-gnu; --help/--version alone are insufficient. **Codec, Verify**
- [ ] **BUILD-02 — Linux ARM64 GNU.** Build and encode on aarch64-unknown-linux-gnu, including its native assembly path and real input conversion. **Codec, Verify**
- [ ] **BUILD-03 — Linux x86_64 musl.** Preserve the fully static x86_64-unknown-linux-musl release contract, including OpenH264 and the C++ runtime. The wrapper recognizes musl in its build logic but its published support table does not verify this target. **Verify**
- [ ] **BUILD-04 — Linux ARM64 musl.** Preserve fully static aarch64-unknown-linux-musl output and actual encoding; confirm C++ toolchain, native objects, linker, and deployed execution. **Verify**
- [ ] **BUILD-05 — Apple Silicon.** Build and encode on aarch64-apple-darwin with supported native frameworks and no runtime FFmpeg requirement. **Codec, Verify**
- [ ] **BUILD-06 — Minimal agent images.** Run both linux/amd64 and linux/arm64 images without compilers, codec downloads, or FFmpeg installed. The current image recipe only installs CA certificates, while the existing encoder starts ffmpeg from PATH: treat that as an unresolved current dependency mismatch. **Adapter, Verify, Limit**
- [ ] **BUILD-07 — Native camera access.** Preserve platform-library camera capability discovery, permissions, explicit container device access, warm sessions, and writable state volumes. Encoder portability does not prove capture-backend portability. **App, Verify**
- [ ] **BUILD-08 — Portable CPU/runtime requirements.** Build for distributable CPU baselines, validate optional NASM/ARM assembly and dispatch, inspect C/C++ runtime linkage, and require no end-user assembler/compiler. **Verify**
- [ ] **BUILD-09 — Bundling and maintenance.** Pin the actual source version/features, preserve required notices, identify native-code security/update ownership, and verify no FFmpeg subprocess/library remains necessary on the selected encoding path. **Adapter, Verify**
- [ ] **BUILD-10 — Additional desktop targets.** For the earlier Windows/macOS/Linux goal, separately validate Windows x86_64/ARM64 and Intel macOS if those are intended releases. They are absent from the current OT-2 release matrix; upstream wrapper platform support is not an application build/runtime result. **Codec, Verify**

## 12. Migration, rollback, and evidence quality

- [ ] **MIG-01 — Existing x264 history.** Retained x264 samples still decode after OpenH264 starts writing to the same source. Current append_h264_sample overwrites one track-level codec_init; list_tracks joins all historical samples to that current initialization. Different SPS/PPS can therefore break old recordings. Demonstrate configuration-version handling or an explicit safe transition; do not erase history to pass. **Adapter, Verify, Limit**
- [ ] **MIG-02 — Changed/restarted encoder configuration.** Encoder restart, resolution, profile, color configuration, and configured fps changes do not reinterpret older samples or silently leave stale metadata. **Adapter, Verify**
- [ ] **MIG-03 — Active sessions across transition.** Existing clip/live sessions either continue with compatible initialization or recover through an explicit new-init/discontinuity path. A fixed init.mp4 URL and refreshed media must not combine incompatible epochs. **Adapter, Verify**
- [ ] **MIG-04 — Rollback.** Returning to the old encoder preserves recordings from both implementations and does not reopen the same SPS/PPS-overwrite problem in reverse. **Adapter, Verify**
- [ ] **MIG-05 — Quality and storage equivalence.** Compare the same lab footage for readable labels, motion, color, bitrate, file growth over configured retention, and resulting disk-pressure behavior. Record accepted limits rather than assuming an x264/OpenH264 preset equivalence. **Verify**
- [ ] **MIG-06 — Whole-service cost.** Compare startup, capture-to-display delay, CPU, peak/steady memory, and dropped frames with both sources active and multiple viewers; include JPEG decoding/conversion and all-IDR overhead. **Verify**
- [ ] **MIG-07 — Existing test scope.** Retain the inspected source/DVR/protocol/browser tests, but add real encoder-produced media before claiming replacement compatibility. Injected capture tests bypass H.264; synthetic NAL, box, and append-order assertions do not prove valid decoded pictures. **Verify**
- [ ] **MIG-08 — Hardware smoke scope.** The current agent-media-smoke script assumes a live MJPEG multipart stream and advancing JPEGs; it does not validate HLS/H.264 decoding. Update the validation plan to distinguish MJPEG still reuse from encoded follow rather than treating a smoke pass as encoder proof. **Adapter, Verify, Limit**

## OpenH264 evidence and configuration cautions

Reviewed the already-cached published openh264 and openh264-sys2 0.9.9 source, alongside the official [encoder API](https://docs.rs/openh264/latest/openh264/encoder/struct.Encoder.html), [configuration API](https://docs.rs/openh264/latest/openh264/encoder/struct.EncoderConfig.html), [wrapper platform/features documentation](https://github.com/ralfbiedert/openh264-rs), and [Cisco upstream](https://github.com/cisco/openh264). No package was installed for this review.

The local published wrapper source exposes profile, level, intra-frame period, force-IDR, frame-skipping, rate-control, timestamps, VUI, thread count, frame-type reporting, and per-layer NAL iteration. It configures one spatial and one temporal layer. Its source-backed build compiles native C++ libraries. This supports investigating a matching encoder configuration; it does not establish every checklist result.

Do not use Encoder::new defaults as application parity. The reviewed defaults enable frame skipping, set a 120,000 bps target, leave maximum frame rate at 0, do not select an explicit profile/level, and set intra-frame period 0. A compatible adapter needs deliberate baseline/all-IDR/no-skip/fps/quality/color choices. The current CMAF writer assumes all samples are sync samples; changing GOP structure is a separate packaging/seek/trick-play change.

Resolution and timestamp API availability are also narrower than application support. The wrapper checks for at most 3840×2160 landscape or 2160×3840 portrait, accepts I420 through the YUV interface, and reinitializes when dimensions change. Its timestamps are milliseconds; the DVR timeline remains microseconds. Input adapters must validate dimensions/strides before constructors that can panic.

## Decision rule

Desk research supports OpenH264 as a candidate for the current H.264 codec role, with no identified fundamental mismatch for 1080p baseline all-IDR encoding. It does not establish support for every current use case or deployment artifact.

Before adopting it, obtain observed results for ENC-01 through ENC-12, IN-01 through IN-08, PKG-01/02/03/09/10, RATE-02/04/07, BUILD-01 through BUILD-09, and MIG-01 through MIG-08; run the existing surrounding regressions for the affected paths. Resolve the current limitations explicitly instead of checking them as already-supported functionality. Keep the remaining experimental work outside this desk-research task.
