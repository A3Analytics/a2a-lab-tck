---
id: TASK-1
title: Bootstrap the standalone A2A-LAB TCK repository
status: Done
assignee:
  - '@me'
created_date: '2026-10-09 19:59'
updated_date: '2026-10-09 20:03'
labels:
  - tck
  - bootstrap
  - backlog-implementer
dependencies: []
references:
  - AGENTS.md
  - mise.toml
  - README.md
priority: high
type: chore
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Turn the template checkout into the canonical A3 Analytics repository for the A2A-LAB Technology Compatibility Kit. The repository must clearly own A2A-LAB conformance tooling, remain distinct from the upstream A2A protocol TCK, and provide reproducible Rust and mise quality gates before extracted behavior lands.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Repository metadata, package naming, README, and agent guidance consistently identify A3Analytics/a2a-lab-tck as the authoritative A2A-LAB Technology Compatibility Kit and explicitly distinguish it from the upstream A2A protocol TCK
- [x] #2 Pinned Rust 1.98.1 and required test tooling are available through mise tasks for format checking, warning-denied checking, Clippy, tests, and one aggregate quality gate without installing tools during task execution
- [x] #3 A minimal Rust workspace builds and its aggregate quality gate passes from a clean checkout
- [x] #4 Backlog.md project metadata is named A2A-LAB TCK and repository documentation follows the configured Backlog.md documentation policy
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Replace template identity in AGENTS.md and README.md with the canonical A3Analytics/a2a-lab-tck scope and explicit upstream A2A protocol TCK distinction.
2. Add a dependency-free Rust 2024 workspace skeleton named a2a-lab-tck with repository metadata and pinned Rust 1.98.1 toolchain.
3. Configure mise tasks for format checking, warning-denied checking, Clippy, nextest, and aggregate quality without runtime installation.
4. Run focused task checks and the aggregate quality gate, verify all acceptance criteria, then record notes and completion metadata.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
- Replaced template identity in README.md and AGENTS.md; Cargo package metadata points to A3Analytics/a2a-lab-tck and explicitly separates A2A-LAB conformance from the upstream A2A protocol TCK.
- Added a dependency-free Rust 2024 workspace, pinned Rust 1.98.1 toolchain, cargo-nextest 0.9.146, and mise build/fmt/check/clippy/test/quality tasks. No tools or dependencies were installed during implementation.
- Verified `mise run build` and `mise run quality`; formatting, warning-denied check, Clippy, and 1 nextest test pass.
- Confirmed Backlog.md project_name is A2A-LAB TCK, no Backlog docs exist yet, and README.md/AGENTS.md are the only policy exceptions outside Backlog task and skill metadata.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Bootstrapped A3Analytics/a2a-lab-tck as the authoritative standalone A2A-LAB Technology Compatibility Kit repository, explicitly separate from the upstream A2A protocol TCK. Added the dependency-free Rust workspace, pinned mise toolchain, documented quality workflow, and aggregate quality gate; `mise run build` and `mise run quality` pass with all four acceptance criteria verified.
<!-- SECTION:FINAL_SUMMARY:END -->
