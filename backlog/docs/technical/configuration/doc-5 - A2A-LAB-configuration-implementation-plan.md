---
id: doc-5
title: A2A-LAB configuration implementation plan
type: technical
audience: technical
created_date: '2026-11-05 21:51'
---

# A2A-LAB configuration implementation plan

## Goal and delivery boundary

An operator can select video sources and the chat provider/model in one versioned
`a2a-lab.json` file. The first consuming application is `a2a-lab-ot2`; shared
configuration types and loading belong to `a2a-lab-dev-kit-rs`. This repository
owns reusable configuration contract checks, not camera or model-provider clients.
All delivery tasks are recorded in this repository's backlog and name their
implementation repository explicitly. Sibling backlogs are not modified.

This is a proposed v1 design, not implemented behavior. JSON uses the existing
Serde/serde_json dependencies in all three repositories. No new tool or dependency
installation is needed for planning; any implementation requiring one must follow
the owning repository's approval policy.

Semantic video indexing is deferred in full: no indexing section, embeddings,
index stores, indexing model, scheduler, or placeholder runtime. A later version
can add a separate section without repurposing chat settings. Other exclusions:
RTSP/HTTP/file ingestion, new camera backends, new LLM providers, hot reload,
browser configuration editing, and migration of unrelated robot/OIDC/SiLA settings.

## Existing boundaries

Source inspection on November 5, 2026 found:

- OT-2 `src/bin/a2a-lab-ot2.rs`: Clap flags/env/defaults, `video_config`,
  `merged_video_config`, and provider/model selection during `serve`.
- OT-2 `src/agent.rs`: `ModelProvider`, `selected_model`, and `LabAgent::from_provider`;
  existing provider identifiers are `bedrock`, `openai`, and `anthropic`.
- OT-2 `src/videos/serve.rs` and `src/videos/catalog.rs`: fixed `external-camera`
  and `camera` slots, injected JPEG producers, shared capture/DVR settings.
- Dev-kit `src/videos/capture.rs`, `capture_profile.rs`, `capture_session.rs`, and
  `nokhwa_device.rs`: distinct MJPEG and packed YUY2/YUYV camera input formats,
  advertised-profile selection, negotiation checks, and injectable raw capture.
  `jpeg_derivative.rs` and `color_pipeline.rs` already handle these inputs for
  downstream JPEG/H.264 processing; OT-2's current `LiveJpegSource` interface is
  JPEG-only and must be adapted for configured raw input.
- OT-2 `src/videos/dvr/`: storage already keyed by video source identifier.
- OT-2 `.mise/scripts/agent-image-lib.sh` and `agent-run.sh`: container arguments.
- TCK `Cargo.toml`, `examples/generate-schemas.rs`, `tests/contract.rs`, and
  `tests/compatibility_contract.rs`: immutable dev-kit dependency and contract checks.

The OT-2 onboard camera is stills-only; Orbbec/frame-set configuration is a separate
existing path. Neither becomes a supported live video backend through this work.
Camera/codec internals and the media gateway remain existing implementation
boundaries. Capability availability comes from libraries, without OS-based gates.

## Proposed v1 contract

```json
{
  "schema_version": 1,
  "chat": {
    "provider": "bedrock",
    "model": "operator-selected-model-id"
  },
  "videos": {
    "sources": [
      {
        "id": "external-camera",
        "name": "Bench camera",
        "description": "Overview of the bench",
        "format": "mjpeg",
        "device_index": 4
      },
      {
        "id": "camera",
        "name": "Detail camera",
        "format": "yuy2",
        "device_index": 6
      }
    ],
    "fps": 5,
    "live_retention": "5s",
    "dvr": {
      "retention": "3d",
      "storage_path": "./state/video-dvr"
    }
  }
}
```

The model ID is illustrative; the plan makes no claim about model availability.
The example device indices are operator choices, not automatic discovery.

- `schema_version` is required and must equal integer `1`. Reject unknown keys
  at every nesting level and unsupported versions with a field path and safe
  diagnostic. Reject malformed JSON and duplicate JSON object keys.
- `chat` and `videos` are independently optional. Chat provider/model are optional
  overrides; provider is one of the three existing identifiers, and an explicitly
  supplied model must contain non-whitespace text. Model defaults stay in the
  consuming application, avoiding duplicate provider-default constants in dev-kit.
- A present `videos` object requires `sources`. An empty list explicitly selects
  no file-configured sources. An omitted `videos` section preserves legacy video
  behavior. Optional `fps` is a positive integer representable by the runtime;
  device index is a nonnegative u32 including zero. Retention uses duration
  strings as specified below.
  `device_index` is the local capture-library camera index. Indexed-device support
  depends on the active library/backend; the numeric mapping is not portable
  across machines/platforms or guaranteed stable after reconnects or enumeration
  changes. Operators must select/verify it on each deployment. Keep `id` stable
  when updating the index so retained history stays associated with that source.
  Serial/path-based device selection is outside this v1 config contract.
- Each source requires a unique valid `VideoSourceId`, `format`, and device index.
  `format` selects the camera input format: `mjpeg` requests encoded JPEG frames,
  and `yuy2` requests packed YUY2 bytes (`Y0 U0 Y1 V0`). Accept `yuyv` as an alias
  of `yuy2` and normalize it to `yuy2`. Map these to the existing `CaptureFormat`
  and `CaptureFormatPreference` types, not the playback `VideoCodec` enum.
  H.264 is an output encoding in the existing pipeline, not a supported input in
  this v1 contract; reject `h264`, `h265`, and other unsupported input formats.
  Input selection must not select or change DVR/output codecs. Retain existing
  application output policy and media interfaces; there is no output-format
  setting in this feature. Capture libraries remain internal details, with no
  public `backend` field.
  IDs are operator-defined, with existing IDs preserved during legacy translation.
  No fixed two-source limit is imposed. Reject repeated device indices even when
  formats differ so two producers do not open the same configured device;
  different indices aliasing one physical camera remain a runtime limitation.
  Request the selected input when opening the device and verify the negotiated
  input and raw sample layout. Explicit input formats must not silently fall back
  to another format or relabel converted bytes. An unavailable device mode or
  capture capability is an explicit source failure, isolated from healthy sources.
  File validation checks the request without opening the device and cannot prove
  that a particular camera supports it. Reuse the existing raw capture/conversion
  APIs; do not implement a new capture backend, decoder, or encoder.
- Optional name/description must contain non-whitespace text when present. Name
  defaults to ID and description to name; legacy translation preserves its current
  display strings. Frame rate and retention settings apply to all sources in v1.
  Omitted runtime settings retain current application defaults.
- `videos.live_retention` is the in-memory live window. DVR settings belong in
  the optional `videos.dvr` block: `retention` is the DVR retention time and
  `storage_path` is the storage directory. The two DVR fields are independently
  optional; an omitted block or field retains its current application default. Example defaults
  remain `"5s"` for live memory and `"3d"` for DVR. Reject unknown DVR fields and
  the superseded flat `disk_retention_secs`, `dvr_root`, and `live_retention_secs`
  keys in this unpublished file contract; do not introduce aliases for them.
- A supplied `videos.dvr.storage_path` must contain non-whitespace text.
  File-relative paths resolve against the selected file's directory, independent of process working directory;
  absolute paths remain absolute. Do not expand shell variables or `~`.
- Credentials remain in existing provider SDK environment/profile chains:
  `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, and AWS settings. The file has no secret
  fields or interpolation. Parsing/loading never initializes SDKs, opens devices,
  creates storage, binds listeners, or contacts any service. Diagnostics contain
  field paths and error categories, never raw file contents or rejected values.
- Unknown future sections, including `semantic_indexing`, fail clearly in v1.
  Format versioning is independent of the existing A2A-LAB compliance profile.

## Duration strings and library choice

Use duration strings for both `videos.dvr.retention` and `videos.live_retention`:
`"5s"`, `"5m"`, `"1h"`, `"1h30m"`, `"3d"`, or `"1w"`. They describe elapsed
time, with a day fixed at 24 hours and a week at seven such days, independent of
time zones and daylight saving. Both values must be positive, exactly representable
as whole seconds, and within the existing runtime/storage arithmetic limits.
Equivalent expressions such as `"60m"` and `"1h"` resolve to the same duration.
Accept a fractional unit only if its total is whole seconds (`"1.5m"` = 90s);
reject a remaining fraction (`"1.5s"`) rather than truncating or rounding.
Reject blank strings, unitless values, numeric JSON values, zero, negative spans,
malformed/unknown units, calendar months/years, and parse/conversion overflow.
Invalid-value diagnostics identify the configuration path without echoing the value.

Rust's [`std::time::Duration`](https://doc.rust-lang.org/std/time/struct.Duration.html)
represents elapsed time but has no shorthand string parser. Two libraries were
checked on November 5, 2026:

- [`Jiff 0.2.37 Span`](https://docs.rs/jiff/0.2.37/jiff/struct.Span.html) already
  exists as a direct dependency in both dev-kit and OT-2. It parses friendly spans
  including combined units. Its `SpanRelativeTo::days_are_24_hours()` marker makes
  conversion of days/weeks to an elapsed duration explicit. Prefer this existing
  dependency: parse through Jiff, reject month/year components, convert with the
  fixed-day marker, and apply the positive/whole-second/runtime-bound checks.
  Do not hand-write a unit parser or use floating-point totals. Other Jiff syntax
  remains acceptable when it meets the same elapsed-time validation rules.
- [`humantime::parse_duration`](https://docs.rs/humantime/2.4.0/humantime/fn.parse_duration.html)
  is a focused alternative for shorthand units and compounds. It would add a new
  dependency and gives fixed approximations to months/years; neither is needed
  here because Jiff is already available. Do not install it for this change.

The shared config API owns parsing, so consumers use one interpretation. Its
JSON Schema exposes strings with examples and descriptions, not Rust duration
objects or integer seconds; arithmetic and whole-second checks remain runtime
validation. No dependency is installed during this planning work.

## Selection, precedence, and compatibility

`A2ALAB_CONFIG` is the application environment variable for selecting the config
file. Planned usage after file support ships:

```sh
export A2ALAB_CONFIG="/etc/a2alab/a2a-lab.json"
```

The application reads that file without requiring `--config`. Absolute paths are
used directly; relative config-file paths resolve from the process working
directory. Paths containing spaces are supported. A selected empty, missing,
unreadable, or invalid file is an error; do not silently ignore it and use defaults.
An explicit `--config` overrides the environment variable. Without either selector,
legacy behavior continues. File-relative DVR storage paths still resolve from the
selected file's directory. The container launcher maps the host-selected file to
its read-only container mount as defined in TASK-18.

1. Explicit `--config PATH` selects the file, otherwise `A2ALAB_CONFIG` selects it.
   With neither set, retain existing behavior. Do not search parent/home directories
   or implicitly load a file based on its name. Empty explicit paths are errors.
2. For supported settings, resolve explicit CLI > explicitly set environment > file
   > existing defaults. Clap defaults must not masquerade as explicit overrides.
   Validate the selected file before merging; invalid file values are errors even
   if another source would override them. Validate the resolved result as well.
3. Source lists replace rather than concatenate. Legacy camera index overrides
   target `external-camera` or `camera` by ID; replace that entry's device index or
   append it if absent. Preserve an existing file-selected input format; a new
   legacy entry defaults to `mjpeg` because both legacy paths capture MJPEG.
   Other file sources remain. Explicit external-camera
   description overrides target that entry only and do not enable a source.
   File `sources: []` disables sources unless an explicit camera index override
   enables one. Reject repeated device indices after merging regardless of format.
4. CLI/env DVR storage paths preserve their existing working-directory semantics; only
   values originating in the file are relative to its directory. Remember field
   provenance while resolving `videos.dvr.storage_path`. Existing second-based
   flags and environment variables remain compatible: `--video-disk-retention-secs` and
   `A2ALAB_VIDEO_DISK_RETENTION_SECS` override `videos.dvr.retention`,
   `--video-retention-secs` and `A2ALAB_VIDEO_RETENTION_SECS` override
   `videos.live_retention`, and the existing `--video-dvr-root` /
   `A2ALAB_VIDEO_DVR_ROOT` overrides target `videos.dvr.storage_path`.
   Convert duration strings to validated whole seconds only
   at the existing runtime boundary; the public file uses DVR terminology and
   unit-bearing strings. This task does not rename legacy flags/env variables.
5. Resolve provider first. An explicit model from CLI/env/file wins; when absent,
   use the existing Bedrock-specific override for Bedrock, then the selected
   provider's current default. Ignore the Bedrock override for other providers.
   Do not infer a provider from a model ID. No-file behavior retains existing
   parsing/default semantics, including existing legacy normalization.
6. Read the file once at invocation; changes require restart. Add `config validate`
   to validate the file plus supported overrides without running the application.
   It exits zero on valid configuration and nonzero with safe diagnostics on errors;
   it requires no cameras, robot, provider credentials, database, or network.
7. Source IDs remain the identity for A2A/MCP discovery, live media, and retained
   DVR records. Reordering entries does not change identity. Existing still-image
   behavior for legacy IDs must continue; arbitrary new video IDs do not create
   new still-image sources. Do not remove historical DVR data during migration.

## Backlog and delivery order

Each row is one bounded PR. Tasks stay `To Do`, with unchecked criteria and no
Implementation Plan until work starts. Dependencies point only to earlier IDs.
There is no broad parent implementation task requiring the whole feature at once.

| Task | Target repository | Bounded deliverable | Depends on |
| --- | --- | --- | --- |
| TASK-9 | dev-kit | Versioned envelope and chat override types | — |
| TASK-10 | dev-kit | Video definitions, DVR block, and shorthand duration validation | TASK-9 |
| TASK-11 | dev-kit | Explicit file loader and path handling | TASK-10 |
| TASK-12 | TCK | Configuration schema and contract fixtures | TASK-11 |
| TASK-13 | OT-2 | File selection, pure precedence resolution, validate command | TASK-11 |
| TASK-14 | OT-2 | Generalized source catalog and existing raw input adapters | — |
| TASK-15 | OT-2 | Resolved video configuration connected to serve | TASK-13, TASK-14 |
| TASK-16 | OT-2 | Resolved chat configuration connected to serve | TASK-13 |
| TASK-17 | OT-2 | Deterministic file-to-runtime regression coverage | TASK-15, TASK-16 |
| TASK-18 | OT-2 | Read-only container config mount and argument forwarding | TASK-13 |
| TASK-19 | OT-2 | Operator examples and migration documentation | TASK-17, TASK-18 |

TASK-12 and TASK-13 can proceed independently after the shared dev-kit API lands;
TASK-14 is independently useful before file support. Chat wiring can proceed
independently of video wiring. Each consuming repository pins the delivered dev-kit
revision through its existing dependency mechanism; no floating branch dependency
or runtime dependency on the TCK is introduced.

## Context budget and verification

Each task names a short starting file list in its description, reads this document
and its predecessor API/diff, and changes one layer. Do not load all three repos
into context. Keep tests and implementation in the owning repository; use injected
video producers and a recording model factory rather than live devices or paid
model requests. If a task still requires an unrelated subsystem rewrite, split it
through the Backlog CLI before starting that rewrite. This bounds each task well
within a 200k-token implementation context without making a token-use guarantee.

Important regression cases: no file, chat-only file, videos-only file, empty source
list, three sources, both input formats and the YUYV alias, duplicate IDs/devices (including the
same device with different formats), index zero, malformed/unknown version,
unknown keys/duplicate object keys, reordered sources, CLI/env/file/default
precedence, shorthand/compound retention strings, elapsed-day/week semantics,
invalid/zero/negative/fractional-second/overflow durations, legacy seconds
overriding file durations, provider switch without an explicit model, relative DVR storage path from a
different working directory, source failure isolation, restart with existing DVR,
unsupported device modes without input-format fallback, and credentials absent.
Use an injected `ColorCamera` with advertised profiles and MJPEG/YUY2 raw samples
to assert requested, negotiated, and delivered input formats, including mismatch
and invalid packed-layout cases. Verify input selection does not select an output
codec; raw samples must reach existing downstream conversion interfaces correctly.
Contract tests must not mistake parsing a provider/model
name for evidence that the provider exists, is credentialed, or can answer.

Run focused tests and `mise run quality` in every changed implementation repo.
The TCK contract change also runs `mise run schemas` and `mise run compatibility`.
Keep profile 1.3.0 case IDs, fixtures, result schemas, CLI behavior, and GitHub
Action inputs unchanged; these config checks are additive library/contract checks,
not a new requirement on remote implementations. Preserve doc-1's metadata/privacy
boundaries. No release or publication is included.

The task Definition of Done remains: checked acceptance criteria, Implementation
Notes, Final Summary, applicable quality gate passing, and `Done` via CLI. Planning
alone checks no implementation criteria and completes none of these tasks.

Backlog task files are gitignored. This document preserves the task sequence,
decisions, and dependencies for review; use `mise exec -- backlog task list --plain`
and `mise exec -- backlog task <id> --plain` to inspect local task details.

