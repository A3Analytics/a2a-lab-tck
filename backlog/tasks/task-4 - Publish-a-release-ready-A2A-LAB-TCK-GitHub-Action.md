---
id: TASK-4
title: Package a release-ready A2A-LAB TCK GitHub Action
status: Done
assignee:
  - '@me'
created_date: '2026-10-09 19:59'
updated_date: '2026-10-09 20:26'
labels:
  - tck
  - github-action
  - backlog-implementer
dependencies:
  - TASK-3
references:
  - 'https://github.com/A3Analytics/a2a-lab-dev-kit-rs/blob/v0.1.0/action.yml'
  - >-
    https://github.com/A3Analytics/a2a-lab-dev-kit-rs/blob/v0.1.0/.github/actions/a2a-lab-compliance/run.sh
  - >-
    https://github.com/A3Analytics/a2a-lab-dev-kit-rs/blob/v0.1.0/tests/github_action.rs
priority: high
type: feature
ordinal: 4000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Package the standalone runner as the canonical A3Analytics/a2a-lab-tck composite action and provide consumer guidance that preserves immutable provenance, reports, and consumer-owned badge status. This task prepares release assets and validation but does not push commits, create tags, or publish a GitHub release.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Repository-root action.yml identifies the A2A-LAB Technology Compatibility Kit by A3 Analytics and preserves the published action inputs, outputs, default full suite, default-on LLM check, report path behavior, and compliance/setup exit semantics
- [x] #2 Action tests verify valid default, basic, explicit opt-out, invalid input, compliant, and noncompliant paths and prove that reports and outputs remain available after a completed failing run
- [x] #3 A consumer workflow example pins A3Analytics/a2a-lab-tck to a full 40-character commit SHA, starts caller-owned deterministic endpoints, uploads the JSON report with if: always(), and requires no sibling checkout or private token
- [x] #4 Badge guidance uses the consumer repository's A2A-LAB compliance workflow badge and states that status is bounded to the tested consumer commit, profile, suite/check selection, and immutable TCK revision rather than certification or upstream A2A TCK status
- [x] #5 Repository CI runs the standalone quality gate and an action smoke path on the documented runner image without external model credentials, hardware, Docker, or unpublished sibling content
- [x] #6 Version and provenance documentation identifies profile 1.1.0, the inherited devkit v0.1.0 action contract, and the first standalone release process while making push/tag/release authorization an explicit external checkpoint
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Port the inherited v0.1.0 composite Action contract and wrapper into the standalone repository, preserving validation, runner arguments, outputs, reports, and exit semantics.
2. Add focused Action contract and wrapper tests for defaults, basic, LLM opt-out, invalid configuration, compliant and completed noncompliant runs.
3. Add consumer workflow, badge, version, provenance, and release-checkpoint guidance with immutable standalone TCK references and explicit separation from the upstream A2A protocol TCK.
4. Add standalone CI quality and credential-free Action smoke jobs on the documented runner image.
5. Run focused checks and mise run quality, verify all acceptance criteria, then record notes and summary and mark TASK-4 Done.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
- Added the canonical A3 Analytics repository-root composite Action with the inherited devkit v0.1.0 input/output contract, profile 1.1.0, default full suite, default-on LLM check, retained reports, and exit classes 0/1/2.
- Added 11 focused Action tests covering defaults, basic, LLM opt-out, invalid inputs, compliant and noncompliant completion, output/report retention, consumer provenance guidance, and CI smoke requirements.
- Added an ubuntu-24.04 CI workflow for mise run quality and a local composite Action smoke path using a deterministic credential-free runner override; no hardware, Docker, external model credentials, sibling checkout, or private token is required.
- Added the consumer workflow and badge example, README Action interface and badge boundaries, and PROVENANCE release metadata. The example intentionally uses a 40-hex placeholder because this task is not authorized to create or publish the release commit; publication must replace it with the authorized immutable standalone revision.
- Verification passed: focused github_action tests 11/11, mise run quality 32/32 tests plus fmt/check/Clippy, git diff --check, executable wrapper check, and IDE diagnostics with no errors.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Packaged A3Analytics/a2a-lab-tck as the canonical profile 1.1.0 composite GitHub Action while preserving the inherited devkit v0.1.0 controls, outputs, report retention, and exit behavior. Added a self-contained consumer workflow/badge example, immutable provenance and release-checkpoint guidance, credential-free ubuntu-24.04 CI smoke coverage, and 11 focused Action tests.

Verification: mise run quality passed with 32/32 tests, focused Action tests passed 11/11, git diff --check passed, and IDE diagnostics were clean. No push, tag, release, or TASK-5 work was performed; publication must replace the example SHA placeholder with the authorized release commit.
<!-- SECTION:FINAL_SUMMARY:END -->
