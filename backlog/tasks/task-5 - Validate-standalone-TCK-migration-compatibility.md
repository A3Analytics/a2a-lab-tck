---
id: TASK-5
title: Validate standalone TCK migration compatibility
status: Done
assignee:
  - '@me'
created_date: '2026-10-09 19:59'
updated_date: '2026-10-09 20:36'
labels:
  - tck
  - compatibility
  - backlog-implementer
dependencies:
  - TASK-2
  - TASK-3
  - TASK-4
references:
  - 'https://github.com/A3Analytics/a2a-lab-dev-kit-rs/releases/tag/v0.1.0'
priority: high
type: task
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Prove that the extracted contract, runner, CLI, report, and action preserve the intended public devkit v0.1.0 behavior before consumers migrate. Compare only the shared A2A-LAB TCK contract; naming, repository coordinates, and added provenance are intentional differences.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A bounded compatibility matrix compares devkit v0.1.0 and standalone behavior for compliant and noncompliant basic runs, default full/default-on LLM runs, explicit LLM opt-out, invalid configuration, unavailable capability, timeout, and A2A/MCP mismatch cases
- [x] #2 Compared completed runs agree on exit class, profile and suite selection, enabled checks, compliant status, ordered case/scenario identifiers, required/outcome values, and normalized diagnostics; machine-readable reports validate against the standalone schema
- [x] #3 Action compatibility checks compare all existing inputs and outputs, defaults, report retention, and noncompliance failure behavior while explicitly allowing repository/name/branding and provenance differences
- [x] #4 Every unexplained difference reports the compared case, devkit observation, standalone observation, and affected public contract, and fails the validation
- [x] #5 The validation records devkit revision 9d5327868d96b3e800fd89f6debf434bcc12709d and the tested standalone revision and passes without network model services, Docker, or hardware
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Build a bounded local compatibility harness against devkit revision 9d5327868d96b3e800fd89f6debf434bcc12709d and the OT-2 deterministic fixture.
2. Compare contract/schema/report/CLI/Action behavior across the required compliant, noncompliant, LLM, error, timeout, and protocol-mismatch cases, failing on unexplained normalized differences.
3. Prove standalone-to-devkit one-way dependency architecture and record tested revisions without network model services, Docker, or hardware.
4. Run focused validation and mise run quality, then check acceptance criteria and complete task metadata.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
- Added `mise run compatibility` and `.mise/scripts/compatibility.sh`, which archive and build devkit v0.1.0 commit 9d5327868d96b3e800fd89f6debf434bcc12709d locally, build the standalone TCK and OT-2 deterministic fixture offline, and retain machine-readable evidence under target/compatibility.
- Compared compliant basic, compliant default full/default-on LLM, compliant full LLM opt-out, controlled noncompliance, A2A/MCP mismatch, invalid configuration, unavailable capability, and timeout behavior. Exit classes, profile/suite/check selection, compliant status, ordered case/scenario IDs, required/outcome values, and normalized diagnostics agree; unexplained differences are empty.
- The validation compares inherited Action inputs, outputs, defaults, runner arguments, report/noncompliance semantics, and allows only naming/branding/provenance differences. Existing Action tests verify retained reports and outputs after exit 1.
- Evidence records standalone base revision 7cbfe5db652de8cd853c46139f50fb096b14c225, tested source SHA-256 839913193e5d69b7c8f3c30adce928ec87e353ab5dc65949350c6543cde2a2a1, OT-2 revision 8366f1b653931a4956b73e6b977a17deb70b8a63, and no network model service, Docker, or hardware use.
- Dependency proof passed: a2a-lab-tck depends one-way on public devkit typed-seam commit 928eae822318f650f8fe2f187e99091d4b46647b; that devkit manifest has no TCK dependency.
- Verification passed: `mise run compatibility`, focused migration tests 4/4, `mise run quality` 36/36 tests plus fmt/check/Clippy, `git diff --check`, and IDE diagnostics. Devkit and OT-2 source worktrees were not changed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added a reproducible offline migration validator that compares devkit v0.1.0 and standalone TCK behavior against the OT-2 fixture across the required matrix, validates Action and report contracts, and proves one-way dependency architecture. The final evidence has no unexplained differences; `mise run compatibility` and `mise run quality` pass without network model services, Docker, or hardware.
<!-- SECTION:FINAL_SUMMARY:END -->
