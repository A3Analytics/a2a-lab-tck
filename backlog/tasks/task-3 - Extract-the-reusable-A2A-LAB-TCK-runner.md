---
id: TASK-3
title: Extract the reusable A2A-LAB TCK runner
status: Done
assignee:
  - '@backlog-implementer'
created_date: '2026-10-09 19:59'
updated_date: '2026-10-09 20:23'
labels:
  - tck
  - runner
  - backlog-implementer
dependencies:
  - TASK-2
references:
  - >-
    https://github.com/A3Analytics/a2a-lab-dev-kit-rs/blob/v0.1.0/src/compliance_runner.rs
  - >-
    https://github.com/A3Analytics/a2a-lab-dev-kit-rs/blob/v0.1.0/src/bin/a2a-lab-compliance.rs
  - >-
    https://github.com/A3Analytics/a2a-lab-dev-kit-rs/blob/v0.1.0/tests/compliance_runner.rs
priority: high
type: feature
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Provide the standalone library and command that execute the extracted contract against caller-started A2A-LAB A2A and MCP endpoints. Preserve deterministic evidence and current defaults while keeping implementation fixtures, hardware, and model-provider credentials outside the TCK. The runner requires a public immutable devkit revision that exposes only the typed malformed-image seam needed by profile 1.1.0; arbitrary unchecked constructors and raw protocol escape hatches remain out of scope.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The library runner executes every required profile case over A2A and MCP, including the three malformed-image cases through the public typed devkit TCK seam, normalizes only documented protocol-envelope differences, and reports actionable parity, timeout, transport, and expected-error diagnostics without aborting the remaining matrix
- [x] #2 The basic suite runs operation cases only; the default full suite runs all basic cases and all required linked scenarios; the full-suite deterministic A2A agent-message capability check is enabled by default and has an explicit opt-out
- [x] #3 The CLI accepts the published v0.1.0 URLs, fixture, profile, suite, LLM-check, timeout, report, and implementation identity contract; exits 0 for compliance, 1 for a completed noncompliant report, and 2 for setup/configuration failures
- [x] #4 Completed compliant and noncompliant runs retain a validated, machine-readable JSON report at the caller-selected path, including effective suite and enabled-check evidence
- [x] #5 Focused tests cover success, unavailable capabilities, basic/full selection, all four A2A/MCP handoff paths, all three malformed-image invalid cases on both interfaces, deterministic LLM success and isolated failures, timeout, normalized mismatch, report round-trip, and CLI exit behavior without Docker, hardware, or external credentials
- [x] #6 The standalone repository quality gate passes with a one-way dependency on one public immutable devkit revision containing the typed TCK seam, with no direct lower-level protocol dependency, arbitrary raw request path, public unchecked constructor, or devkit dependency on the TCK
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. After the devkit typed-seam prerequisite is implemented and its revision is publicly fetchable, update the standalone devkit pin and verify the malformed-image request seam from this external crate.
2. Adapt the published v0.1.0 runner to the standalone contract, using the typed seam for the three invalid image cases and public devkit clients for all execution.
3. Add the public a2a-lab-compliance command with preserved default full suite, default-on LLM check, opt-out, report, and exit-code behavior.
4. Extract focused runner and CLI tests covering the required matrix, malformed-image rejection on both interfaces, handoffs, failure isolation, report round trips, and command exits without external services.
5. Run focused checks and mise run quality, verify dependency direction and every acceptance criterion, then record notes and summary and mark TASK-3 Done.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
- Blocked before code extraction: v0.1.0 src/compliance_runner.rs constructs intentionally invalid ListImageSourcesRequest, ListImagesRequest, and SearchImagesRequest values through pub(crate) compliance_unchecked constructors. The pinned public devkit API validates and rejects these values and exposes no raw A2A operation or MCP tool-call entry point. Exact standalone execution therefore requires either a public devkit client escape hatch, direct lower-level protocol dependencies, or delegation to the embedded devkit runner; all three conflict with the intended architecture. No acceptance criteria were checked.
- Plan revised after the blocker investigation: the devkit backlog now has a prerequisite for a narrow typed public seam covering exactly the three malformed-image cases. Resume extraction only after that devkit change is available at a public immutable revision; broad unchecked constructors and generic raw A2A/MCP calls are explicitly excluded.

- Resolved the prerequisite by pinning the public devkit seam at remote main commit 928eae822318f650f8fe2f187e99091d4b46647b and regenerating Cargo.lock to the same full SHA.
- Extracted the reusable library runner and a2a-lab-compliance CLI. The runner uses TckMalformedImageRequests only for the three profile-defined malformed image cases and public typed clients for all A2A/MCP execution.
- Preserved basic/full/default behavior, all linked handoffs, default-on deterministic agent-message evidence with --no-llm-check, timeout/transport/parity/expected-error diagnostics, validated report retention, and exit classes 0/1/2.
- Added focused external tests, including explicit proof that all three malformed image requests execute on both interface adapters. mise run quality passed: format, check, Clippy with warnings denied, and 21/21 nextest tests. IDE diagnostics reported no errors.
- Dependency audit passed: cargo tree has one direct devkit dependency at the immutable seam commit; the TCK declares no reqwest, rmcp, a2a-client, raw JSON, or unchecked seam, and the pinned devkit manifest has no a2a-lab-tck dependency.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Extracted the profile 1.1.0 A2A-LAB runner and CLI into the standalone TCK, preserving the published v0.1.0 suite, LLM-check, diagnostic, report, and exit behavior.

Pinned the one-way public devkit dependency to 928eae822318f650f8fe2f187e99091d4b46647b and used its narrow typed malformed-image seam for exactly three cases. mise run quality passes with 21 tests.
<!-- SECTION:FINAL_SUMMARY:END -->
