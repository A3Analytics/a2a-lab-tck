---
id: doc-6
title: Semantic video frame indexing and decision records plan
type: technical
audience: technical
created_date: '2026-11-06 21:56'
---

# Semantic video frame indexing and decision records plan

## Goal and delivery boundary

Every successfully saved video frame receives a durable semantic-index entry and
an explicit processing state. Operators can search hours or days of retained video
using text or a reference frame, resolve the exact evidence, and inspect multiple
versioned Jev decisions about that frame. Initial decision examples:
pipette tip attached to a pipetter, foreign object present, and plate misaligned.

This is a researched design and implementation backlog, revised November 7, 2026.
No model, vector database, or production indexing implementation was installed or
benchmarked during planning. Recommendations below are engineering inferences
from primary sources and the inspected code, subject to the evaluation gates.

All delivery tasks live in this TCK backlog and explicitly name their target
repository. Shared contracts, provider boundaries, transport bindings, and UI
belong in `a2a-lab-dev-kit-rs`; persisted indexing, inference adapters, and camera
application integration belong in `a2a-lab-ot2`; reusable conformance checks belong
in `a2a-lab-tck`. Sibling repositories and their backlogs are unchanged by this
planning work. Each task is one PR in one owning repository.

Related: [doc-5 configuration plan](../configuration/doc-5%20-%20A2A-LAB-configuration-implementation-plan.md),
[doc-2 streaming research](doc-2%20-%20Video-streaming-package-research-plan.md),
[doc-3 codec checklist](doc-3%20-%20OpenH264-current-video-use-case-checklist.md),
[doc-4 camera formats](doc-4%20-%20Camera-video-formats-and-frame-rates.md), and
[doc-1 profile contract](../../reference/compliance/doc-1%20-%20A2A-LAB-TCK-profile.md).
Doc-5 intentionally deferred semantic indexing; this is its separate successor
feature. Existing configuration v1 and historical compliance profiles remain
unchanged. No publication, robot-control policy, or dependency installation is
authorized by the implementation plan itself.

## Inspected code and constraints

Starting snapshot: TCK HEAD `559b9da`, dev-kit HEAD `2e0f877`, OT-2 HEAD `1f6b720`,
plus their current worktree contents. Concurrent work can change these; tasks must
recheck interfaces rather than rely on line numbers. The TCK currently pins a
different immutable dev-kit revision in `Cargo.toml`. Existing changes to
`mise.toml` and other Backlog documents were left untouched.

| Boundary | Existing starting points | Consequence |
| --- | --- | --- |
| Successful media save | OT-2 `src/videos/dvr/store.rs`, `chunk.rs`, `migrations/005_video_dvr_chunks.sql` | SQLite `dvr_frames` references payload offsets in minute chunks. Register semantic intent at the committed saved-row boundary. |
| Capture isolation | OT-2 `src/videos/catalog.rs`: `queue_disk`, `write_disk` | The disk queue can drop camera frames. Index only successfully saved frames; do not claim those dropped frames were saved or indexed. |
| Current row identity | `dvr_frames` primary key `(source_id, track_id, start_us)` with payload-updating conflict handling | Timestamp alone is insufficient immutable evidence identity. Payload replacement must create a distinct semantic revision. |
| Media access | dev-kit `src/videos/dvr_store.rs`, `provider.rs`, `model.rs`; OT-2 `src/videos/dvr/provider.rs` | Reuse retained-window lookup, codec initialization, track epochs, source scope, and authenticated media resolution. |
| Pixel conversion | dev-kit `src/videos/mjpeg_decode.rs`, `color_pipeline.rs`, `jpeg_derivative.rs`, `h264_sample.rs` | Save-time pixels can reduce redundant decode, but replay needs a bounded retained-media evidence reader and verified decoder capability. |
| Configuration | dev-kit `src/config/mod.rs`, `video.rs`; OT-2 `src/lab_config/` | Current v1 rejects indexing keys. Add explicit v2 support rather than changing v1 interpretation or borrowing chat settings. |
| Decision model | OT-2 `src/agent.rs` has chat providers; no Jev implementation was found in inspected source | TypeSafe AI publishes Jev's decision/image API. Add an adapter behind a neutral evaluator contract; keep it separate from chat and embeddings. |
| TCK | `src/compliance.rs`, `compliance_runner.rs`, `src/compatibility/`, `examples/generate-schemas.rs` | Test deterministic metadata/transport behavior. Real vision quality and sustained throughput require separate evidence. |

The initial feature covers saved RGB/color video frames. It does not automatically
include depth/IR frame sets, still-image catalogs, new RTSP/file inputs, a camera
backend migration, or codec/player replacement. H.264 decode availability must be
verified independently from an encoder being available. Capability checks follow
available libraries, with no OS-specific feature gates.

## Research and recommendation

Use Luna as the default configurable semantic description model. Each successfully
saved frame receives a bounded visual description; a separately configured text
embedding encoder turns that description into a searchable vector. Luna is chosen
by the user, so no vision-model comparison or SigLIP/PE/Qwen adoption gate precedes
this implementation. The earlier encoder research remains background evidence.

The model roles are fixed:

| Role | Selected behavior | Configuration and authority |
| --- | --- | --- |
| Frame description | Luna by default | Description provider/model, prompt version, output limit, and credentials are independently configurable. Output is search metadata only. |
| Retrieval vectors | Embed persisted description text and query text in one compatible space | Configure embedding provider/model/dimensions separately. This encoder neither describes pixels nor decides lab predicates. |
| Frame decisions | Jev only | Configure a pinned Jev version, endpoint/credentials, predicates and deterministic interpretation policy. No LLM decision model, judge, router, or fallback is supported. |
| Pilot input | Existing retained color video on Chupacabra | Discover actual source IDs and retained time ranges at pilot start; use those recordings and their actual available duration. |

Jev receives the exact saved images/ROIs, any bounded original-frame history, and
operator-authored rules/reference images. Luna descriptions, chat responses,
embedding similarities, and LLM-generated criteria cannot be decision evidence or
produce/repair/arbitrate a model decision. Description failure does not block Jev;
Jev failure remains failed/not-evaluated or unknown according to the explicit
policy, with no LLM fallback. Pure code may validate and threshold Jev outputs.

Existing OT-2 `src/agent.rs` provides provider-specific Luna identifiers:
`DEFAULT_BEDROCK_MODEL = global.openai.gpt-5.6-luna` and
`DEFAULT_OPENAI_MODEL = GPT_5_6_LUNA`. Reuse those identifiers for the applicable
provider rather than guessing a new API ID. Description settings have their own
resolved values and never inherit a subsequent chat setting change implicitly.
Changing the description model remains supported and starts a new description
provenance/generation; it cannot change the Jev-only decision policy.

Keep the asynchronous durable manifest, bounded workers, source/time filters,
retention, temporal grouping and immutable evidence identity from the research.
SQLite is authoritative; Qdrant is the initial vector-store candidate. Validate it
at Chupacabra's actual retained volume. LanceDB/pgvector comparison and larger
synthetic scale tests can follow if measured limits justify them; neither new
hardware nor an external corpus is required for the initial pilot.

Relevant research already reviewed on November 6, 2026:

- [Long Video Haystack](https://arxiv.org/abs/2504.02259) and
  [multi-granularity retrieval](https://arxiv.org/abs/2504.08384) motivate bounded
  retrieval and temporal navigation. Keep every saved frame discoverable; scene
  cuts or summaries cannot replace frame coverage.
- [Qdrant storage](https://qdrant.tech/documentation/manage-data/storage/) and
  [payload indexes](https://qdrant.tech/documentation/concepts/payload/) support
  filtered persistent search; measure freshness, deletion and restart behavior.
- [LanceDB Rust API](https://docs.rs/lancedb/latest/lancedb/) and
  [pgvector](https://github.com/pgvector/pgvector) are future alternatives if needed.
- [SQLite WAL](https://www.sqlite.org/wal.html) motivates short transactions and
  durable reconciliation between metadata and vector writes.
- The earlier [SigLIP 2](https://arxiv.org/abs/2502.14786),
  [PE-Core](https://huggingface.co/facebook/PE-Core-L14-336) and
  [Qwen embedding](https://huggingface.co/Qwen/Qwen3-VL-Embedding-2B) investigation
  is not the selected description pipeline.

Luna descriptions can omit visual details. Preserve original evidence, measure
search misses on actual Chupacabra frames, and keep Jev's independent image-based
decision path intact. Descriptions and retrieval relevance never establish a
positive or negative predicate.

## Saved-frame identity and coverage

A saved frame is a successfully committed DVR media sample, with payload bytes
still resolvable or an explicit unavailability state. MJPEG and H.264 samples are
included. Alternate tracks/renditions can contain the same capture; preserve an
entry for each saved sample and group them only when original-capture linkage is
known. Do not infer equivalence merely from equal wall-clock timestamps.

Proposed immutable `SavedFrameRef` fields:

- `frame_id`: opaque stable versioned ID derived from a persisted store namespace,
  source, track/epoch, original capture timestamp, and payload SHA-256 revision,
  using unambiguous canonical encoding. Keep a separate revision when an existing
  DVR key is overwritten with different bytes.
- Store/source/track identity, original capture UTC and media timing where
  available; optional capture-session/sequence identity for verified grouping.
- Immutable payload digest and a private media locator. API responses return
  authorized media references rather than filesystem paths.
- Media availability and indexing generation/state. Evaluation time is separate
  from capture time. Clock reversals and epoch boundaries must not merge frames.

Each retained saved revision has a manifest record even when inference is down.
For an enabled source, every decodable saved frame eventually receives its own
persisted description and description-vector association; there is no silent
periodic/keyframe-only policy.
At observation time, pending, leased, described, embedded/vector-pending, ready, retryable
failure, terminal failure, expired, and superseded-media states are explicit.
An embedding-ready flag requires the selected vector backend's acknowledgement,
not just an in-memory description or embedding result.
Description records bind exact frame/ROI revision, provider/resolved model, prompt
digest, description text/hash, generation and generation time. A ready retrieval
entry requires a persisted description and its corresponding acknowledged vector.

Byte/pixel-exact reuse under identical preprocessing and embedding space can be
added behind verified identity and reference counts. Perceptual or near-duplicate
reuse is off initially. Every alias must still resolve to its original frame and
time, and deletion must not remove vectors referenced by retained frames.

Coverage uses per-frame accounting plus bounded intervals/gaps. A maximum indexed
timestamp alone cannot prove completeness when jobs complete out of order. A
search with incomplete coverage states that limitation; no returned match is not
evidence of an absent pipette tip or an aligned plate.

## Incremental processing and recovery

```text
existing camera / encoder / bounded disk queue
                    |
        payload append + saved DVR row
                    |
    SQLite transaction: manifest + indexing intent
                    |
      bounded lease / decode / ROI / Luna description
                    |
    SQLite: persisted description + provenance
                    |
       configured text embedding encoder
                    |
    SQLite: text embedding + vector-write intent
                    |
       idempotent vector upsert / acknowledgement
                    |
                 ready frame

exact retained frame evidence -> Jev-only decision jobs
                             -> immutable Jev decisions in SQLite

scoped query -> matching text embedding -> filtered ANN
             -> authoritative revision/retention checks
             -> optional decision filters / temporal grouping
             -> exact frame and bounded neighbor playback
```

1. Registration occurs in the saved-row SQLite transaction; model/vector calls
   never run on capture or media append. A notification only wakes workers and
   is not the durable record of work.
2. Existing media rows are discovered by bounded reconciliation. Filesystem
   append and SQLite commit are not jointly atomic: crashes can leave appended
   bytes without a row or a damaged row after power failure. Reuse DVR recovery,
   verify payload digest/readiness, and record corrupt evidence explicitly.
3. Description and embedding stages have independent durable checkpoints. Reuse a
   committed description after embedding failure instead of calling Luna again.
   Workers claim leases with compare-and-set ownership, finite deadlines,
   bounded batches/bytes/pixels, capped concurrency, and jittered backoff. Reclaim
   expired leases; poison frames do not block later frames from that source.
4. A durable vector-write intent bridges SQLite and the selected backend. Point
   IDs include frame revision, model space, ROI, and generation. Upserts are
   idempotent; acknowledgement and intent replay tolerate either crash order.
5. Retention tombstones are authoritative. Late model/upsert completion cannot
   revive expired or replaced evidence. Reads validate source access and current
   retained revision even before physical vector deletion succeeds.
6. Bound memory queues separately from on-disk outstanding work. A disk/backlog
   limit pauses inference/job materialization visibly, while committed DVR rows
   and a reconciliation cursor preserve discovery of all saved frames. Report
   deferred frames; do not silently drop indexing intent or block the camera.
   Existing media retention limits total retained work; if an outage exceeds
   retention, report expired-unindexed frames rather than claiming coverage.

Avoid repeated JPEG/H.264 decode when safe save-time pixels exist, but do not
make transient pixel caches authoritative. Backfill must work from persisted
media using bounded codec dependencies, SPS/PPS/epoch handling, and exact frame
matching. RGB resize/crop, orientation, interpolation, normalization, and any
text-query template are part of the versioned indexing provenance. Description
and text-embedding versions are separate; re-embedding retained text does not
regenerate descriptions, while changing Luna/prompt explicitly creates a new
description generation.

## Storage and sizing

SQLite stores manifest identity, bounded descriptions/provenance, state, job
leases/intents, Jev decision history,
small evidence metadata, and coverage. It does not execute a full multi-day
Python/Rust cosine scan per interactive query. Media remains in existing chunks;
do not store another full image copy merely to index it.

The vector backend stores frame vectors and filter payloads: store, source,
track/epoch, capture time, frame revision, generation, model space, description digest/generation, ROI, and
retained state. Use ANN with indexed metadata filters and bounded exact rescoring
where supported. Keep namespaces per compatible embedding space, not per frame
or minute. Partition/shard only when measurements justify it. Metadata filters
are a retrieval optimization, not authorization; enforce access in the service.

For `C` sources, saved rate `F`, retention `D` days, dimension `E`, bytes/value
`B`: `N = C * F * 86400 * D`, raw vector bytes `N * E * B`.

| Sources | Saved fps/source | Days | Frame vectors | Raw 768d fp16 | Raw fp32 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | 5 | 1 | 432,000 | 0.62 GiB | 1.24 GiB |
| 1 | 5 | 3 | 1,296,000 | 1.85 GiB | 3.71 GiB |
| 4 | 5 | 3 | 5,184,000 | 7.42 GiB | 14.83 GiB |
| 1 | 30 | 1 | 2,592,000 | 3.71 GiB | 7.42 GiB |
| 4 | 30 | 3 | 31,104,000 | 44.49 GiB | 88.99 GiB |

These are future capacity illustrations, not pilot input requirements, measured index sizes or the selected
model's promised dimension. ANN graphs, payload indexes, SQLite rows, WAL,
replication, retained media, temporary rebuild generations, and ROI vectors add
cost. Bounded description text adds `N * mean_description_bytes`, plus indexes
and metadata; measure it on the pilot. One extra ROI description/vector per frame
roughly doubles that part of the count. Quantization
or lower precision is a benchmarked optimization, not a free reduction; retain
the selected metric and compare recall against exact fp32 search.

Re-embedding builds a new generation alongside the active one. Queries use one
space/generation at a time; cutover is explicit and atomic only after measured
coverage. Capacity planning includes both generations and rollback retention.
Disk-backed storage, expiration, and compaction need measurement under ingest,
not just a static search benchmark.

## Retrieval and temporal organization

Text queries and persisted Luna descriptions use the same configured text encoder.
A saved reference-frame query reuses its description/vector; it does not require
a second image encoder or ask an LLM to decide relevance. An unindexed reference
frame reports pending/unavailable rather than silently calling a decision model.

Require finite source/time scope and bound top-k, page size, candidates, neighbors,
and output bytes. Filter by source/time/embedding space/description generation/ROI,
validate manifest revisions and authorization, and use bounded stale-hit refill.
Return actual frame time, description provenance, evidence link, and explicit
partial indexing/candidate-budget state. Similarity is a retrieval score only.

Fixed windows (initial experiment: 10 seconds, with 60-second navigation buckets)
can group adjacent hits into navigable moments. Preserve direct frame candidates
and drill-down, without grouping across gaps, sources or unrelated codec epochs.
All-frame accounting still includes a brief event whose description is poor.

Use deterministic ranking/grouping and exact time navigation. No LLM reranker,
judge, generated narrative conclusion, or automatic decision is part of search.
Decision filters read only persisted Jev decisions. Luna text is labeled generated
search metadata in the UI and is never presented as a verified Jev assessment.

## Decision records and evaluator path

### Jev research and concrete integration

Jev is [TypeSafe AI's System One decision model](https://docs.typesafe.ai/introduction/quickstart).
It consumes state and named questions and returns typed answers. This public model
matches the user's description; it is not an unidentified internal project.
Jev evaluates bounded questions rather than producing a free-form answer.
Per the user's model policy, it is the sole production frame decision model.

| Jev primitive | Documented return | Proposed lab use, subject to evaluation |
| --- | --- | --- |
| [Noul](https://docs.typesafe.ai/primitives/noul) | Probability that a proposition is true | Tip attached; foreign object visible; plate misaligned against a supplied reference |
| [Choice](https://docs.typesafe.ai/primitives/choice) | Selected option, option probabilities, confidence | Present/absent/occluded or object category with an explicit unknown/other option |
| [Score](https://docs.typesafe.ai/primitives/score) | Position on described ordered levels, level probabilities, confidence | Qualitative alignment severity against an explicit rubric; not millimeters or pose calibration |
| [Box](https://docs.typesafe.ai/primitives/box) | One normalized bounding box and confidence | Evidence location for a named tip, object, or plate |

The [current model reference](https://docs.typesafe.ai/models) lists
`jev-1.14.0` with all four primitives and text/image inputs; it does not ingest
video or audio natively. Pin that version and record the response's resolved
model, rather than using a moving alias. It ingests shared state once for several
questions. Published input pricing is $0.042 per million tokens, with free output;
actual image cost must come from measured usage. Organization TPS/RPS quotas are
deployment inputs, not assumed throughput guarantees.

[Image support](https://docs.typesafe.ai/images) accepts still PNG/JPEG assets in
state, instructions, or criteria, with base64 or multipart uploads. Current bounds
are 280–1568 pixels per side, 16 MiB/image and separate 32 MiB request/expanded-image
limits. Rejected images fail the request, rather than producing partial answers.
Use bounded frame/ROI preprocessing with recorded transforms. A short ordered
frame history or reference image can provide context without sending a whole video.

[TypeSafe's release note](https://typesafe.ai/blog/jev-1-14) and image documentation
mark images and Box experimental and below the vendor's production-readiness
standard. They require `TypeSafe-Danger-Zone-Images: true`. Keep this integration
an explicit opt-in evaluator and measure real lab accuracy before adoption.

Box always returns coordinates, even when the requested object is absent. Pair
localization with a separate presence/observability question; a returned rectangle
alone is not evidence that a tip or foreign object exists. One Box question returns
one target. Map its coordinates through any crop/resize transform back to the
original saved frame. These behaviors are explicit in the
[Box reference](https://docs.typesafe.ai/primitives/box).

Preserve Noul probability, Choice/Score distributions, rubric score, and vendor
confidence separately. Noul has no additional confidence field. For Choice/Score,
confidence summarizes the distribution and is not another class probability;
see [confidence semantics](https://docs.typesafe.ai/confidence). The application
maps observable evidence and locally validated thresholds into present/absent/
unknown and stores that policy version. A 0.5 default is not a validated lab rule.

Proposed adapter: authenticated `POST https://api.typesafe.ai/v1/systemone`, pinned
`jev-1.14.0`, exact-frame image asset once in state, three independent lab questions,
and optional observability/reference/Box questions. Record predicate-level outputs,
resolved model, question/criteria digests, usage, thresholds, and evidence revision.
Use request IDs to associate results; retry transient/429 errors with bounded
backoff, and never turn HTTP/parse errors into negative observations. No native
Rust SDK is required: the documented HTTP boundary can use the application's
existing approved client stack.

For frame-rate scheduling, several predicates on one frame can share a request.
At 5 fps, one request per saved frame means five requests per second per camera,
432,000 per day. This is arithmetic, not measured Jev capacity. Benchmark billed
tokens, latency, quotas, and useful decision coverage before selecting an all-frame
policy. Per-frame descriptions use configurable Luna and their vectors use the
separate text retrieval encoder;
Jev's typed decision API is not a vector-index API.

### Common decision contract

Only the authorized Jev adapter or a configured authenticated Jev worker can append
production model decisions through the optional API for an exact `SavedFrameRef`.
Require matching evaluation ownership and recorded Jev response provenance; a
client-supplied producer label alone is not proof. Luna, chat, other LLMs and
alternative vision decision models cannot write model decisions. Human labels
are evaluation-corpus annotations, not substitute model decisions. Description
and embedding readiness are not prerequisites.

Proposed record shape, with illustrative values rather than a finalized API:

```json
{
  "decision_id": "opaque-id",
  "idempotency_key": "evaluation-id/predicate/subject",
  "frame_id": "immutable-saved-frame-revision",
  "predicate": "lab.pipette_tip_attached",
  "predicate_version": 1,
  "subject": {"kind": "pipetter", "id": "left/channel-1"},
  "outcome": "present",
  "value": {"tip_count": 1},
  "evidence": [{"frame_id": "immutable-saved-frame-revision", "roi": [0.1, 0.2, 0.3, 0.4]}],
  "producer": {"id": "typesafe-jev", "model_revision": "pinned-revision"},
  "evaluation_id": "request-id",
  "evaluated_at": "2026-11-06T20:00:01Z",
  "config_digest": "sha256:example",
  "prompt_digest": "sha256:example",
  "threshold_policy_digest": "sha256:example",
  "probability": null,
  "probabilities": null,
  "score": null,
  "confidence": null,
  "calibration_id": null,
  "supersedes": null
}
```

The referenced frame supplies capture time. An actual schema must validate IDs,
digests, field limits, producer authorization, predicate/value compatibility, and
evidence scope. `roi` means `[x_min,y_min,x_max,y_max]`, normalized to the original
oriented color frame; preserve the model-input transform. Structured values may
carry alignment displacement/angle and units when a calibration/reference exists.
Store concise structured evidence/reason codes, not private model reasoning.

| Initial predicate | Required subject/reference | Outcome semantics |
| --- | --- | --- |
| `lab.pipette_tip_attached` v1 | Pipetter/mount and channel; tip evidence ROI | Present or absent only when observable; occluded/out-of-view is unknown. |
| `lab.foreign_object_present` v1 | Bench/deck region, object evidence, versioned expected-scene policy | A visible object is foreign relative to configured expectations; uncertain/unconfigured expectations can produce unknown. |
| `lab.plate_misaligned` v1 | Plate/deck slot, expected pose or alignment reference, optional calibration | Misalignment must state its reference/tolerance; missing reference or unobservable pose produces unknown. |

`present`, `absent`, and `unknown` are decision outcomes. `not_evaluated`, timed-out,
failed, unsupported, and cancelled describe evaluation coverage/job state, never
an implicit negative. Scores are optional raw model values unless an identified
calibration makes them probabilities. Record calibration/model changes separately.

Append-only records preserve disagreement. Identical idempotent replay returns
the original record; a different payload under the same key conflicts.
Supersession is an explicit acyclic link within the same frame/predicate/subject
and authorized Jev producer stream. Separate Jev versions, predicate policies or
evaluations retain their own assessments rather than overwrite history. Query policy must
choose full history or latest per producer; no implicit global latest-wins truth.

The Jev-only evaluator interface declares supported predicates, model/runtime identity,
evidence modalities, context bounds, deadlines, and output limits. The durable
runner accepts explicit frame/range requests, with separately declared all-frame
or sampled policies. Decision computation is expensive and independent of the
required per-frame embedding path; report its actual coverage. A single image
decision cannot establish temporal actions such as successful liquid transfer.

Jev's public runtime/API is now identified above. Remaining deployment inputs are
an authorized TypeSafe account/API key and quotas, representative lab evidence,
the chosen reference images/policies, and passing the real-image adoption gate.
The adapter task can implement against public request/response fixtures without
credentials; its paid live smoke run remains opt-in. Authorized Jev producer APIs
and deterministic Jev test doubles remain independently useful. No robot actuation or safety
interlock is implied by recording or searching a decision.

Decision-filtered search joins exact frame revisions and explicit predicate,
subject, producer/version, outcome, and history policy. Filter eligible evidence
before final top-k or use bounded refill with an explicit partial-result flag.
Do not fetch ten vectors and quietly discard nine by decision afterward.
Metadata-only decision queries remain available during a vector-service outage.

## Configuration and model-role enforcement

The v2 shape below is illustrative; TASK-43 finalizes validation and schemas.
Descriptions use an independent provider/model selection with Luna as the default.
Text embeddings require an explicitly configured compatible encoder. Decisions
accept only the Jev provider with a pinned supported Jev version. Unknown provider
values, decision fallback/judge settings, and generated-description decision inputs
are validation errors before startup. A description override cannot change this.

```json
{
  "schema_version": 2,
  "semantic_indexing": {
    "description": {
      "provider": "bedrock",
      "model": "global.openai.gpt-5.6-luna"
    },
    "embeddings": {
      "provider": "operator-configured-text-embedding-provider",
      "model": "operator-configured-text-embedding-model"
    },
    "decisions": {
      "provider": "jev",
      "model": "jev-1.14.0"
    }
  }
}
```

Embedding identifiers above are placeholders for deployment configuration, not
claims that a provider/model exists. Credential references, output/token limits,
ROI policy, retry/concurrency/backlog and query bounds are explicit independent
settings. Never put raw secret values in examples. Description omission resolves
to the relevant provider's Luna default; explicitly enabling indexing still
requires a valid text embedding configuration. Existing v1 files keep their
original behavior.

The production decision call graph has only the Jev adapter. It has no reference
to a description/chat client and never requests LLM-generated rules or captions.
Jev result provenance and evaluation ownership are checked before append. Pure
validation, coordinate transforms and versioned threshold code can interpret
Jev outputs; code cannot manufacture missing model evidence. Jev outage or
uncertainty remains visible, without a fallback request. Tests use throwing
Luna/chat spies to prove zero calls on the decision path; normal description
indexing is allowed to call its configured LLM.

## Evaluation gates

The first three tasks inventory existing Chupacabra video and measure the selected
Luna description pipeline and vector backend at its actual volume. Do not acquire
new recordings, change cameras, provision more hosts, or require a 72-hour live run
for this pilot. Keep existing capture/services unchanged. Enumerate retained source
IDs, codecs, timestamps and counts when the pilot starts; recorded camera inventory
in doc-3/doc-4 is historical, not proof of current availability.

Label a bounded subset of available frames and split by retained recording/run or
disjoint time blocks, avoiding adjacent-frame leakage. Missing examples of a lab
predicate are reported as untested, not filled by inventing a new capture or
external dataset. Measure description/search quality separately from Jev outcomes. Include one-frame events,
multi-minute static scenes, occlusion, lighting changes, multiple cameras,
codec epochs, genuine gaps, and rare events near window boundaries. Synthetic
fixtures validate plumbing but cannot close real visual-accuracy gates.

Future production adoption budgets below remain useful targets. The initial
Chupacabra pilot reports observed values and unavailable evidence; its completion
does not require new inputs or those production targets to pass:

| Gate | Initial target | Evidence |
| --- | --- | --- |
| Saved-frame accounting | 100% of committed retained revisions in an explicit ready/pending/failure state; zero silent omissions | Manifest/DVR reconciliation through crash and retention fixtures |
| Inference capacity | At least 1.5 times aggregate configured saved fps, including decode/preprocessing | Existing Chupacabra sources at their saved rate; larger topologies are future projections |
| Healthy indexing freshness | p95 saved-to-searchable lag at most 60 seconds | Concurrent live ingest and retrieval, reported by source |
| Outage recovery | Thirty-minute provider outage catches up within 60 minutes after recovery at the target load | Durable backlog replay while ingest continues |
| ANN fidelity | Recall@10 at least 0.95 versus exact search under the same filters | Static and ingest-active vectors; report top-k underfill |
| Interactive search | p95 at most 2 seconds for 1.296M vectors; at most 5 seconds for 5.184M | Include text encoding, filters, manifest joins, and bounded grouping |
| Lab semantic retrieval | Event recall@10 at least 0.90 overall; report each predicate/rare-event subset and nDCG@10 separately | Held-out labeled real recordings; thresholds do not imply detection certification |
| Decision quality | Initial evaluation goal: precision at least 0.95, recall at least 0.90 per predicate, with abstention/evaluated coverage reported | Separate present/absent/unknown, per-subject confusion tables and calibration evidence |
| Resource growth | RAM within configured ceiling; disk within measured budget including two generations; queues within declared caps | Replay of existing retained Chupacabra video; longer live soaks are separate future work |

The corpus task must state label counts and denominators; if a slice is too small,
report uncertainty rather than claiming the threshold was established. False
negative cost and acceptable abstention depend on the later operational use.
Neither this plan nor generic retrieval scores authorize a robotic control rule.

## Backlog and delivery order

The task register below is populated from CLI-created backlog items.
Current scope: configurable Luna descriptions, Jev-only decisions with no LLM
fallback, and a pilot using only video already retained on Chupacabra. All remain
`To Do`, acceptance criteria unchecked, and without Implementation Plans until
implementation starts. Dependencies point only to lower-numbered items. External
gates (approved dependencies/hardware/evidence and TypeSafe access) are explicit
prerequisites, not invented completed work.

| Task | Target repository | Bounded deliverable | Depends on |
| --- | --- | --- | --- |
| TASK-20 | a2a-lab-ot2 | [Define an evaluation subset from existing Chupacabra video](../../../tasks/task-20%20-%20Define-a-labeled-lab-video-retrieval-evaluation-corpus.md) | — |
| TASK-21 | a2a-lab-ot2 | [Validate configurable Luna frame descriptions on Chupacabra video](../../../tasks/task-21%20-%20Compare-compact-and-multimodal-frame-embedding-models.md) | TASK-20 |
| TASK-22 | a2a-lab-ot2 | [Measure vector indexing at Chupacabra retained-video volume](../../../tasks/task-22%20-%20Compare-persistent-vector-stores-at-multi-day-frame-volumes.md) | TASK-21 |
| TASK-23 | a2a-lab-dev-kit-rs | [Add stable saved-frame and embedding provenance contracts](../../../tasks/task-23%20-%20Add-stable-saved-frame-and-embedding-provenance-contracts.md) | — |
| TASK-24 | a2a-lab-ot2 | [Persist a semantic frame manifest beside the DVR index](../../../tasks/task-24%20-%20Persist-a-semantic-frame-manifest-beside-the-DVR-index.md) | TASK-23 |
| TASK-25 | a2a-lab-ot2 | [Register indexing work at the saved-frame commit boundary](../../../tasks/task-25%20-%20Register-indexing-work-at-the-saved-frame-commit-boundary.md) | TASK-24 |
| TASK-26 | a2a-lab-ot2 | [Read bounded frame evidence from retained DVR media](../../../tasks/task-26%20-%20Read-bounded-frame-evidence-from-retained-DVR-media.md) | TASK-24 |
| TASK-27 | a2a-lab-dev-kit-rs | [Define frame description and text embedding provider contracts](../../../tasks/task-27%20-%20Define-a-model-neutral-frame-embedding-provider.md) | TASK-23 |
| TASK-28 | a2a-lab-ot2 | [Implement configurable Luna description and text indexing adapters](../../../tasks/task-28%20-%20Implement-the-selected-frame-embedding-provider-adapter.md) | TASK-21, TASK-27 |
| TASK-29 | a2a-lab-ot2 | [Process durable description and embedding jobs with bounded retry](../../../tasks/task-29%20-%20Process-durable-embedding-jobs-with-bounded-retry-and-leases.md) | TASK-25, TASK-26, TASK-28 |
| TASK-30 | a2a-lab-ot2 | [Persist and reconcile frame vectors in the selected backend](../../../tasks/task-30%20-%20Persist-and-reconcile-frame-vectors-in-the-selected-backend.md) | TASK-22, TASK-29 |
| TASK-31 | a2a-lab-dev-kit-rs | [Define versioned frame decision records and lab predicates](../../../tasks/task-31%20-%20Define-versioned-frame-decision-records-and-lab-predicates.md) | TASK-23 |
| TASK-32 | a2a-lab-dev-kit-rs | [Define bounded semantic frame search and coverage contracts](../../../tasks/task-32%20-%20Define-bounded-semantic-frame-search-and-coverage-contracts.md) | TASK-27, TASK-31 |
| TASK-33 | a2a-lab-ot2 | [Implement scoped semantic retrieval over retained frames](../../../tasks/task-33%20-%20Implement-scoped-semantic-retrieval-over-retained-frames.md) | TASK-30, TASK-32 |
| TASK-34 | a2a-lab-dev-kit-rs | [Expose optional semantic search through A2A and MCP](../../../tasks/task-34%20-%20Expose-optional-semantic-search-through-A2A-and-MCP.md) | TASK-32 |
| TASK-35 | a2a-lab-ot2 | [Group semantic hits into bounded temporal result windows](../../../tasks/task-35%20-%20Group-semantic-hits-into-bounded-temporal-result-windows.md) | TASK-33 |
| TASK-36 | a2a-lab-ot2 | [Persist immutable frame decisions and supersession history](../../../tasks/task-36%20-%20Persist-immutable-frame-decisions-and-supersession-history.md) | TASK-24, TASK-31 |
| TASK-37 | a2a-lab-dev-kit-rs | [Expose authorized Jev decision append and history APIs](../../../tasks/task-37%20-%20Expose-optional-decision-append-and-history-APIs.md) | TASK-31 |
| TASK-38 | a2a-lab-ot2 | [Connect decision APIs to the DVR decision store](../../../tasks/task-38%20-%20Connect-decision-APIs-to-the-DVR-decision-store.md) | TASK-36, TASK-37 |
| TASK-39 | a2a-lab-dev-kit-rs | [Define a Jev-only frame decision evaluator contract](../../../tasks/task-39%20-%20Define-pluggable-frame-decision-evaluator-contracts.md) | TASK-31 |
| TASK-40 | a2a-lab-ot2 | [Run durable Jev-only frame decision evaluations](../../../tasks/task-40%20-%20Run-durable-frame-decision-evaluations-on-demand.md) | TASK-29, TASK-38, TASK-39 |
| TASK-41 | a2a-lab-ot2 | [Add a TypeSafe Jev image decision adapter](../../../tasks/task-41%20-%20Add-a-contract-driven-JEV-frame-decision-adapter.md) | TASK-40 |
| TASK-42 | a2a-lab-ot2 | [Filter semantic retrieval by recorded frame decisions](../../../tasks/task-42%20-%20Filter-semantic-retrieval-by-recorded-frame-decisions.md) | TASK-33, TASK-38, TASK-34 |
| TASK-43 | a2a-lab-dev-kit-rs | [Add versioned semantic indexing and decision configuration](../../../tasks/task-43%20-%20Add-versioned-semantic-indexing-and-decision-configuration.md) | TASK-39, TASK-32 |
| TASK-44 | a2a-lab-ot2 | [Wire semantic workers and optional providers into OT-2 serve](../../../tasks/task-44%20-%20Wire-semantic-workers-and-optional-providers-into-OT-2-serve.md) | TASK-43, TASK-33, TASK-34, TASK-40, TASK-41 |
| TASK-45 | a2a-lab-ot2 | [Add resumable retained-frame backfill and model generation rebuilds](../../../tasks/task-45%20-%20Add-resumable-retained-frame-backfill-and-model-generation-rebuilds.md) | TASK-30, TASK-44 |
| TASK-46 | a2a-lab-ot2 | [Reconcile semantic retention with DVR media deletion](../../../tasks/task-46%20-%20Reconcile-semantic-retention-with-DVR-media-deletion.md) | TASK-45, TASK-36, TASK-35 |
| TASK-47 | a2a-lab-ot2 | [Report frame indexing coverage and decision evaluation health](../../../tasks/task-47%20-%20Report-frame-indexing-coverage-and-decision-evaluation-health.md) | TASK-44, TASK-42 |
| TASK-48 | a2a-lab-ot2 | [Verify semantic pipeline crash recovery and bounded operation](../../../tasks/task-48%20-%20Verify-semantic-pipeline-crash-recovery-and-bounded-operation.md) | TASK-46, TASK-47, TASK-42 |
| TASK-49 | a2a-lab-tck | [Add semantic frame and decision schema compatibility checks](../../../tasks/task-49%20-%20Add-semantic-frame-and-decision-schema-compatibility-checks.md) | TASK-37, TASK-34, TASK-43 |
| TASK-50 | a2a-lab-tck | [Add opt-in semantic and decision transport conformance fixtures](../../../tasks/task-50%20-%20Add-opt-in-semantic-and-decision-transport-conformance-fixtures.md) | TASK-49, TASK-48 |
| TASK-51 | a2a-lab-dev-kit-rs | [Show semantic search hits and frame decision history in the video UI](../../../tasks/task-51%20-%20Show-semantic-search-hits-and-frame-decision-history-in-the-video-UI.md) | TASK-34, TASK-37 |
| TASK-52 | a2a-lab-ot2 | [Document semantic deployment configuration and decision producers](../../../tasks/task-52%20-%20Document-semantic-deployment-configuration-and-decision-producers.md) | TASK-44, TASK-45, TASK-46, TASK-47 |
| TASK-53 | a2a-lab-ot2 | [Pilot Luna indexing and Jev decisions on existing Chupacabra video](../../../tasks/task-53%20-%20Measure-a-72-hour-semantic-indexing-and-decision-pilot.md) | TASK-48, TASK-52, TASK-35, TASK-41 |

Implementation can proceed along independent contract, storage, and producer-API
branches once their listed prerequisites exist. Description indexing and Jev jobs can be implemented independently. The combined
pilot uses the delivered adapters against the available Chupacabra recordings,
with missing predicate examples reported as untested. Do not use a broad parent task that implements the whole pipeline.

## Context budget and verification

Each task supplies one target repository, a short starting-file list, four
observable criteria, exclusions, and earlier dependencies. Read this design's
relevant sections and predecessor public interfaces/diffs; do not load every
video file in all three repos or hours of frame data into the implementation
context. A 200k-context agent should handle one bounded PR comfortably; that is a
scope goal, not a token-use guarantee. If a task needs an unrelated rewrite or
multiple model/backend implementations, split it through the CLI first.

Use deterministic fake embedders, vector adapters, evaluators, clocks, media, and
failure injection in normal tests. Real-model accuracy, backend scale, and
longer live duration are separate future evidence tasks. For every implementation run focused
tests and the target repository's `mise run quality`. TCK schema/profile work
also runs `mise run schemas` and `mise run compatibility` against an immutable
delivered dev-kit revision. Historical schemas and profiles remain intact;
metadata conformance never claims actual vision accuracy.

Dependencies/tools must not be installed without the owning repository's required
approval. The Backlog CLI installed here accepts `other`, not `technical`, on
document creation, so this document was created via `-t other` and its generated
frontmatter normalized to the repository's required `type: technical` and
`audience: technical`. Task files are managed only through the CLI.

The tracked document preserves the research and task register for review and
handoff. Backlog task files are gitignored by repository policy and remain local
unless explicitly exported through Backlog. Completion of an implementation task
still requires checked criteria, Implementation Notes, Final Summary, passing
quality checks, and `Done` status through the CLI. Planning does not mark any
implementation task complete.

