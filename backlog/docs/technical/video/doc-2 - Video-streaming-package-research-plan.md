---
id: doc-2
title: Video streaming package research plan
type: technical
audience: technical
created_date: '2026-11-04 18:01'
---

# Video streaming package research plan

## Objective and scope

Find the smallest maintainable package combination that replaces substantial custom video code while preserving A2A-LAB live viewing, DVR, seeking, clips, and 8x playback. Evaluate both Rust libraries that avoid FFmpeg and implementations written in Rust without a C/C++ codec dependency. These are separate requirements.

This document records the plan and desk research as of November 4, 2026. Candidates have not been integrated into the project, benchmarked on the target machine, or accepted; one isolated temporary package compile is recorded below. Implementation belongs in `a2a-lab-dev-kit-rs` and `a2a-lab-ot2`; this TCK repository owns contract validation. The OT-2 repository instructions require approval before installing dependencies.

The later [doc-3 use-case checklist](</Users/dylangustaveson/code/a2a-lab-tck/backlog/docs/technical/video/doc-3 - OpenH264-current-video-use-case-checklist.md>) audits the current source snapshot and contains 124 checks. It qualifies the OpenH264 recommendation: JPEG input conversion, all-IDR/no-skip configuration, old-recording SPS/PPS compatibility, and fully static musl builds remain adoption gates. The earlier source snapshot below is historical.

## Current implementation

Inspected sibling worktrees: dev-kit HEAD `1ed5b93`, OT-2 HEAD `40c6836`, and TCK HEAD `559b9da`. The OT-2 worktree contains uncommitted changes in `src/videos/catalog.rs` and `src/videos/dvr/chunk.rs`; those changes move DVR writes to a bounded asynchronous queue and remove a per-chunk `sync_all`. They were left untouched. The H.264 encoder file inspected is unchanged in that worktree.

| Layer | Existing code | Research question |
| --- | --- | --- |
| Camera capture | OT-2 `Cargo.toml` uses `nokhwa` with `input-native`; `src/videos/catalog.rs` captures JPEG frames. | Can the actual camera supply H.264 directly, avoiding transcoding? Keep `nokhwa` unless evidence favors a change. |
| Encoding | OT-2 `src/videos/h264.rs` starts a long-lived FFmpeg process using `libx264`, 5 fps, baseline YUV420, no B frames, and GOP 1. It manually splits Annex B NALs, groups access units, and extracts SPS/PPS. | Can an in-process codec return complete access units and initialization data with less lifecycle and parsing code? |
| Capture/DVR integration | OT-2 `src/videos/catalog.rs` enables encoding for native external capture, associates queued timestamps with encoder output, and queues disk writes; `src/videos/dvr/` stores/indexes samples. | What buffering, timestamp, retention, and recovery behavior must an adapter preserve? Injected sources and webcam capture follow different paths. |
| Packaging | Dev-kit `src/videos/cmaf.rs` writes MP4 boxes; `src/videos/packager.rs` groups samples, renders HLS playlists, handles gaps, live windows, and the 8x rendition. | Can a library own binary serialization and playlists while A2A-LAB retains timeline policy? |
| Delivery | Dev-kit `src/videos/http.rs`, `media_asset.rs`, and `local_media.rs` resolve and serve media assets. | Can packages work behind the existing authenticated gateway and session-relative URLs? |
| Browser playback | Dev-kit `web-ui/src/shell/hls_playback.rs` manages MediaSource/SourceBuffer; `web-ui/src/videos/hls.rs` parses playlists and probes codecs. | Can an established HLS player replace fetching, playlist interpretation, buffering, and recovery? |
| Conformance | TCK doc-1 and the video compliance cases validate typed metadata over A2A/MCP. | Which behaviors need separate real-media/browser verification? TCK success does not establish decodable media. |

The five encoder/packaging/player files above total about 3,482 lines, including tests. This is an investigation boundary, not an estimate of removable production code.

OT-2 `src/opentrons/stream.rs` explicitly rejects the robot-server stream metadata as a usable video source. Keep that constraint; do not substitute picture polling for video.

## Initial package shortlist

### Encoding without FFmpeg

| Candidate | Verified from primary sources | Initial assessment and unresolved work |
| --- | --- | --- |
| [`openh264-rs` / `openh264`](https://github.com/ralfbiedert/openh264-rs) | Release `0.9.9` (October 15, 2026); Rust encoder API over Cisco OpenH264. Bundled native source is the default and compiled through `cc`; no FFmpeg is required. BSD-2. | Best first H.264 compatibility spike. The codec is upstream-maintained by Cisco, while the Rust wrapper has a much smaller maintainer community. It is C/C++, not pure Rust, and does not make the codec’s parsing memory-safe. Verify baseline output, forced IDR/GOP=1, flush behavior, runtime on the target CPU, and JPEG-to-I420 conversion cost. |
| [`rusty_h264`](https://github.com/Remade-With-Rust/rusty_h264) | Release `0.17.0` (October 28, 2026); pure-Rust H.264 encoder/decoder; API accepts I420 and returns an Annex-B access unit. `EncoderConfig::baseline` selects Constrained Baseline/CAVLC and disables lookahead; `gop_size = 1` makes each frame an IDR with SPS/PPS. A scalar build can exclude its optional acceleration crate and unsafe intrinsics; do not use `--no-default-features` alone, because the no-std configuration requires `libm`. An isolated registry build with `default-features = false, features = ["std", "global-alloc"]` compiled without the assembly feature and without FFmpeg/C/C++. | Best direct native-Rust match to the current profile/keyframe behavior on paper. Its published corpus and bit-exact checks against FFmpeg/Cisco decoders, fuzzing claims, CI, and BSD-2 license are meaningful technical evidence. However, the project is very young and rapid release activity is concentrated in one maintainer-led ecosystem. Its README identifies one independent downstream app using the decoder and reports 123,983 active installs; that is useful decoder adoption evidence, but does not establish encoder use or broad production validation. docs.rs failed to build the newest release at review time. Its September all-intra CIF benchmark is maintainer-reported and not target evidence. It merits a serious isolated prototype, but that evidence does not yet establish an independently validated, reputable production dependency. |
| [OxideAV `oxideav-h264`](https://github.com/OxideAV/oxideav-h264) | Current manifest is `0.1.10`; README/changelog describe a pure-Rust H.264 encoder and decoder, including Baseline/Main/High output, rate control, and CAVLC/CABAC. The project also has an [`oxideav-mp4`](https://github.com/OxideAV/oxideav-mp4) crate with fragmented MP4/CMAF muxing. | A relevant second native-Rust candidate and a broader stack comparison. It may cover encoding plus fMP4 serialization, but does not appear to provide HLS playlist policy. The exact published `0.1.10` registry archive contains encoder and decoder modules and registers both factories; an isolated `cargo check` of the pinned crate succeeded. Its package description and introductory `lib.rs` comment are stale. This verifies the artifact builds, not codec quality, conformance, target behavior, or production maturity. The framework is recent and has little independent production adoption despite extensive maintainer-reported conformance gates. Treat as experimental until artifact, CI, API, and target behavior are verified. |
| [Chromium `openh264rs`](https://chromium.googlesource.com/chromium/third_party/openh264rs/) | Chromium-hosted Rust port of Cisco OpenH264, with recent reviewed changes. Its [manifest](https://chromium.googlesource.com/chromium/third_party/openh264rs/+/refs/heads/main/Cargo.toml) declares version 0.1.0 and enables C/C++ API bridges by default; [build.rs](https://chromium.googlesource.com/chromium/third_party/openh264rs/+/refs/heads/main/build.rs) builds C++ bridge code only with `cxx-api`. | Strong provenance makes this worth investigating for native Rust H.264. Verify the Rust-only build configuration, public encoder API, release/distribution status, independent users, unsafe code, conformance corpus, and throughput. Hosting and recent activity alone do not establish production readiness. Do not confuse it with `openh264-rs`. |
| [Xiph `rav1e`](https://github.com/xiph/rav1e) | Established Rust AV1 encoder with configurable speed and bitrate, plus assembly optimizations. No FFmpeg is needed to use its encoder API. | Credible Rust codec option if AV1 is acceptable. It changes the codec: current `VideoCodec` supports MJPEG/H.264/H.265, so assess model, CMAF, MIME probing, browser support, and TCK compatibility work. Measure live performance; reputation does not prove target-device throughput. |
| [Straw Lab `less-avc`](https://github.com/strawlab/less-avc) | Pure Rust H.264 encoder using lossless all-intra PCM; upstream round-trip tests also use external decoders. | Niche option, not an assumed replacement for compressed baseline H.264. Check browser profile support and bitrate/storage cost before spending time on an adapter. Separate production dependencies from upstream test dependencies. |

The distinction is important: FFmpeg is not required to encode H.264 from Rust, and a technically substantial pure-Rust H.264 encoder now exists. From the evidence reviewed, no pure-Rust H.264 encoder yet meets a conservative “independently established production dependency” bar for this application. `rusty_h264` does have one disclosed downstream decoder deployment, but no comparable evidence of its encoder being used in production. `rusty_h264` is the best native-Rust prototype candidate and has stronger technical evidence than a toy or research-only encoder; `openh264` is the lower-risk H.264 compatibility candidate but wraps C/C++. `rav1e` is the more established pure-Rust encoder when an AV1 migration is acceptable. Any H.264 choice needs a separate patent/licensing review; a Rust rewrite does not remove codec patent questions.

The OxideAV artifact check found that `oxideav-h264` 0.1.10 includes encoder and decoder implementations and passes an isolated dependency compile despite stale package and introductory source prose. docs.rs API pages still resolve to the 0.1.8 IDR-only encoder surface, so the published crate source is more current than its rendered API docs. This is useful buildability evidence only; public API ergonomics, bitstream behavior, test quality, and independent adoption remain open. The sibling `oxideav-mp4` crate is version `0.0.10` (2.5 MB source package) and its published API includes fragmented MP4/CMAF muxing with H.264 `avc1` sample entries. It does not supply A2A-LAB's HLS timeline or playlist policy. Its repository has very little independent adoption (2 GitHub stars at review time), so buildability and feature breadth should not be mistaken for an established production track record.

### Reducing hand-written streaming code

| Candidate | Replacement boundary | Limits to investigate |
| --- | --- | --- |
| [`hls.js`](https://github.com/video-dev/hls.js) | Apache-2.0 browser HLS player; current release is `1.7.4` (October 28, 2026). Supports live/VOD, fMP4/CMAF, accurate seeking, gaps/retries, and custom XHR/fetch loaders. Its parser recognizes the playlist tags used here, including `EXT-X-MAP`, discontinuities, program date-time, and gaps. | Strongest package fit. Its current release is signed and the project is actively maintained. The current UI embeds assets in the Rust binary, but its assembly step only copies an explicit asset list and fingerprints known sources; a pinned bundle needs deliberate inclusion in both. A small wasm-bindgen bridge can own startup and teardown. Preserve the existing same-origin URL construction, `Authorization` header, `credentials: omit`, and `cache: no-store` request policy; custom loaders must validate playlist asset URLs before attaching credentials. Retain A2A-LAB session, 8x selection, and UI state. Verify live refresh, gaps, callbacks, browser fallback, and the existing custom behavior before removing code. |
| [`m3u8-rs`](https://docs.rs/m3u8-rs/latest/m3u8_rs/) | Release `6.0.1` (July 30, 2026), 126 GitHub stars at review time; parser and writer support media playlists, `EXT-X-MAP`, discontinuities, and program-date-time. | Plausible replacement for the simple Rust playlist parse/render code. Source parses `EXTINF` into `f32`, so test duration precision and round trips. Preserve the existing relative-path validation wrapper; a standards parser is not an authorization/path-security layer. |
| [`muxfin`](https://docs.rs/crate/muxfin/latest) | Release `0.5.1` (September 14, 2026); pure-Rust MP4/fMP4 mux API accepts encoded H.264 and emits init/media fragments. | More directly relevant to live segments than a generic box model, but very new with no visible GitHub stars/adoption. Prototype fragment validity, exact timestamp handling, H.264 length-prefix conversion, and retained 8x behavior before considering it. |
| [`scuffle-mp4`](https://docs.rs/crate/scuffle-mp4/latest) | Pure-Rust MP4 parser/writer with typed fragment-box models. | Secondary comparison only: current crate warns it may not be stable; docs.rs shows sparse examples and no example coverage. Verify that the writer API actually reduces our fragment code rather than only exposing individual box types. |
| [GStreamer Rust bindings](https://github.com/GStreamer/gstreamer-rs) with [`hlscmafsink`](https://gstreamer.freedesktop.org/documentation/hlssink3/hlscmafsink.html) / [`hlsmultivariantsink`](https://gstreamer.freedesktop.org/documentation/hlsmultivariantsink/index.html) | Established media framework; the current stable series is [1.28.8](https://gstreamer.freedesktop.org/releases/1.28/), released October 8, 2026. GStreamer [1.26 release notes](https://gstreamer.freedesktop.org/releases/1.26/) introduced CMAF HLS and multivariant sinks. The multivariant sink has callbacks for init segments, fragments, media playlists, multivariant playlists, and fragment deletion. Version 1.28 also adds single-media-file and I-frame-only playlist support. | Stronger packaging candidate than the older `hlssink2` comparison: it can serialize CMAF and HLS outputs without an FFmpeg backend. Still requires the native GStreamer runtime/plugins, so it is not pure Rust. H.264 pads expect AVC/AVC3 access-unit input, meaning Annex B parsing/conversion and app-specific capture, DVR retention, gaps, clips, and 8x policy may remain. Verify exact plugin packages, OpenH264 encoder integration, output callback behavior, retention semantics, and gateway integration. |

A newly advertised full Rust media stack is a discovery lead, not evidence of reputation. `transmux` advertises an extensive CMAF/HLS mux surface, but the current release is recent, the crate spans many formats and features, and its published package is large. Evaluate it only if the narrower adapters cannot meet requirements and after a supply-chain/reputation review. GStreamer now has documented CMAF and multivariant HLS sinks, so compare that real packaging path against smaller Rust adapters. Its native runtime/plugin costs and callback integration are the main tradeoffs.

## Desk research results and recommendation

### What the current implementation actually needs

The current path receives JPEG from the camera. FFmpeg currently decodes that JPEG, converts it to I420, encodes constrained-baseline H.264 with an IDR for each frame, and emits Annex B. A replacement encoder taking I420 does not remove JPEG decode or color conversion. The adapter must still provide stable timestamps, one access unit per sample, SPS/PPS for initialization, and defined behavior on restart and flush. The current H.264 wrapper additionally owns a child process, pipe/thread/channel lifecycle, and a hand-written Annex B access-unit parser; a direct encoder API may eliminate these pieces. Upstream `rusty_h264` source exposes an explicit Baseline configuration with CAVLC and no lookahead; with `gop_size = 1` its documented streaming API emits an Annex-B access unit per input frame and prefixes IDRs with SPS/PPS. This closely matches the current output contract, but is source-level compatibility evidence only; the registry build, JPEG conversion, quality, throughput, and playback still need verification.

The dev-kit stores complete encoded samples and implements the DVR window, gap/discontinuity policy, HLS timeline metadata, and trick-play selection. `hls.js` can own more of the browser’s HLS parsing, fetch/retry, MSE buffering, live refresh, and recovery. It cannot move DVR retention, credentials, URL containment, source discovery, or the HLS-specific 8x product decision out of A2A-LAB.

The browser-side integration is feasible but not just a script tag: `web-ui/build/assemble.rs` hashes and packages an explicit set of UI assets into the Rust binary, while `web-ui/build/freshness.rs` skips `vendor/` paths from its source fingerprint. A vendored player bundle must be pinned, included in package freshness and the generated manifest, and served with the existing JavaScript MIME type. The current `Transport` also enforces same-origin requests, omits ambient credentials, disables caching, and sets the bearer token itself. `hls.js` request hooks must preserve those properties and reject external/unsafe playlist URLs before forwarding the token. A native-HLS fallback cannot use these JavaScript request hooks, so the supported browser matrix and its authentication behavior are gating evidence, not a detail to defer.

### Recommended order

1. Prototype `hls.js` first. Its current release and documented feature set support the strongest maturity and fit, and it can reduce code in the roughly 700-line MSE orchestration path plus the playlist parser. Confirm vendored bundle pinning, freshness/integrity and asset packaging in the Rust-only web build, same-origin URL and Authorization-header handling, credentials/cache parity, cancellation, live latency, gaps, supported-browser behavior, and switching between regular and 8x playlists.
2. Compare GStreamer's `hlsmultivariantsink` CMAF output callbacks with the current segment/playlist path and a narrow Rust muxer adapter. Check input conversion from Annex B, output-stream integration with the authenticated asset store, rolling-window behavior, and whether application-specific gaps, clips, and 8x policy remain straightforward. Do not assume the sink handles camera capture or DVR retention.
3. Compare `openh264`, `rusty_h264`, and `oxideav-h264` on the same recorded JPEG frames. OpenH264 is the compatibility baseline; the other two are pure-Rust candidates. Include JPEG decode and color conversion in results. Use the registry artifact/API check already recorded above; its behavioral and quality claims remain unverified. Do not promote either pure-Rust candidate on maintainer claims or benchmark numbers alone.
4. Prototype `oxideav-mp4`, `muxfin`, and (if its API proves viable) `scuffle-mp4` against existing DVR access units. Keep the current packager policy and serve path; replace only fragment construction if a library makes that boundary smaller and passes real browser playback. Evaluate the OxideAV H.264+MP4 combination as a broad pure-Rust stack, while keeping HLS playlists, DVR, and 8x rendition policy in A2A-LAB.
5. Consider `rav1e` only as a separately scoped AV1 migration. Do not treat the codec change as a dependency-only swap.

The clearest near-term package win is browser HLS playback with `hls.js`. For packaging, compare GStreamer 1.28's CMAF/multivariant HLS sinks against a small Rust muxer adapter: GStreamer is the more established framework and does not need FFmpeg, while carrying a native runtime/plugin footprint. The best low-risk FFmpeg-free H.264 encoder route is the Rust wrapper over Cisco OpenH264; the codec is mature C/C++, not native Rust. A native Rust H.264 encoder exists and merits a prototype, but current independent adoption and release evidence is too limited to call it the reputable production choice. OxideAV's MP4 crate does provide CMAF fragment muxing, but has low adoption and does not own A2A-LAB's playlist/DVR policy.

The source review cannot establish target-machine latency, image quality, memory use, runtime/package size, browser behavior, or successful decoding. Those are still required before an implementation decision.

## Research sequence

### 1. Establish requirements and a reproducible baseline

- Trace native external capture, webcam capture, and injected test sources through live and DVR playback. Record actual dimensions, configured fps, sample format, initialization data, and timestamps.
- Check camera/device capabilities for encoded H.264 passthrough. Record whether any supported source can avoid JPEG decode and re-encode entirely.
- Exercise the current pipeline with repeatable footage. Measure startup latency, capture-to-display latency, CPU, peak memory, bitrate, DVR growth, and dropped frames. Include two simultaneous sources and two viewers.
- Inventory existing tests, the supported browser/device matrix, deployment/build constraints, and current user-visible failures. Mark missing hardware evidence explicitly.

Deliverable: pipeline map, requirement matrix, baseline measurements, and a list of custom code each replacement could remove. Agree on performance targets from this baseline before comparing candidates.

### 2. Check reputation, scope, and dependency claims

For each shortlisted package, record an exact release or commit, owner, release history, recent issue handling, CI targets, independent downstream usage, conformance/fuzz tests, known limitations, and source license notices. Record evidence rather than assigning reputation from stars alone.

Inspect manifests, feature defaults, build scripts, and transitive native libraries. Classify separately: FFmpeg subprocess, FFmpeg libraries, other C/C++ runtime, optional bridge/compiler, assembly tooling, hardware/driver requirement, and Rust-only configuration. Verify availability through library capabilities rather than OS-gated application behavior, as required by OT-2 AGENTS.md.

Deliverable: cited comparison matrix with pass/fail/unknown for each requirement. Reject candidates that cannot document a required capability or fail the agreed deployment constraints. Flag any proposed tool/dependency installations for approval before executing them.

### 3. Run isolated feasibility spikes

After any required dependency approvals, run independent spikes so a codec decision does not force a player or packaging rewrite:

1. **Encoding:** feed identical recorded JPEG frames through `openh264`, `rusty_h264`, and `oxideav-h264`; include Chromium `openh264rs` only if its public API and Rust-only build prove viable. Include JPEG decode, color conversion, and buffer copies in measurements. Preserve one complete access unit per stored sample, SPS/PPS initialization, capture timestamps, and the current all-IDR baseline. Test failure, restart, flush, and final-frame delivery. Explore a longer GOP only as a separate measured option, because seeking and 8x playback depend on keyframes.
2. **Packaging:** compare GStreamer's `hlsmultivariantsink` callbacks, `oxideav-mp4`, and one narrow Rust muxer adapter against existing encoded DVR samples. Check Annex B to AVC conversion, timestamps, sample durations, dimensions, codec configuration, gaps, half-open clip bounds, long-range exports, live updates, and 8x sample reduction. Determine which layer owns rolling retention and playlist updates. Compare decoded content and timeline behavior; byte-for-byte equality is unnecessary.
3. **Playback:** connect `hls.js` to the existing media gateway. Verify authorization on every request, relative asset URLs, seeking, live-edge catch-up, discontinuities, abort/unmount cleanup, and normal/8x switching. Exercise native HLS separately where required.
4. **Broad framework comparison:** estimate the GStreamer runtime/plugin footprint and remaining app code from the packaging comparison. Build a wider capture/decode/convert/encode pipeline only if it could materially outperform the focused sink integration in maintenance cost or behavior.
5. **Codec migration contingency:** test `rav1e` only if Rust-native encoding remains a priority and AV1 contract/browser changes are acceptable. Use `less-avc` only if lossless throughput and storage requirements justify its narrower format.

Deliverable: small reproducible spike results, including failures and remaining custom code. Do not merge a rewrite as part of this research.

### 4. Validate and recommend an architecture

Run a 30-minute live/DVR exercise on the leading combination. Include reconnects, missing frames, source loss, server restart, clip export, seeking across gaps, and 8x playback. Use actual browsers to decode output; synthetic MP4 headers and metadata tests alone are insufficient. Confirm the encoder path works with FFmpeg absent and inspect native linkage for FFmpeg libraries.

Run applicable existing `mise` checks in each changed repository, including `mise run quality`, before marking implementation work complete. Retain TCK metadata/privacy checks and add media verification in the owning implementation repository. Keep upstream A2A protocol conformance separate from A2A-LAB compatibility.

Deliverable: decision record comparing maintenance reduction, compatibility, measured latency/CPU/memory/bitrate, deployment cost, and upstream maturity. Document which code remains and why. Produce atomic implementation tasks only after the decision, split by owner and replacement boundary, with observable acceptance criteria and rollback steps.

## Suggested direction before prototypes

Start with `hls.js`, then compare GStreamer's current 1.28 CMAF HLS sinks against a narrow Rust muxer adapter. Compare `rusty_h264` against the OpenH264 wrapper for encoding. Treat AV1 as a separate migration decision.

This is an initial hypothesis, not an adoption decision. The research succeeds when it either demonstrates a compatible reduction in custom code or documents why the current boundary is still preferable, with reproducible evidence.

## Research status

Desk research and code-path review complete. An isolated, temporary dependency compile verified that the pinned `openh264`, `rusty_h264`, `oxideav-h264`, and `oxideav-mp4` registry packages resolve together; the temporary project was removed. No project manifest or sibling worktree was changed. No performance, footage, or browser experiment was run. Keep remaining work at the research-plan level unless the user later asks to authorize prototype work; target hardware and browser evidence would require separate implementation-worktree evaluation.
