---
id: TASK-2
title: Extract the versioned A2A-LAB TCK contract
status: Done
assignee:
  - '@me'
created_date: '2026-10-09 19:59'
updated_date: '2026-10-09 20:09'
labels:
  - tck
  - contract
  - backlog-implementer
dependencies:
  - TASK-1
references:
  - 'https://github.com/A3Analytics/a2a-lab-dev-kit-rs/tree/v0.1.0'
  - >-
    https://github.com/A3Analytics/a2a-lab-dev-kit-rs/blob/v0.1.0/src/compliance.rs
  - >-
    https://github.com/A3Analytics/a2a-lab-dev-kit-rs/blob/v0.1.0/backlog/docs/reference/compliance/doc-23%20-%20A2A-LAB-compliance-profile.md
priority: high
type: feature
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make the standalone repository the authoritative home for A2A-LAB profile 1.1.0, deterministic fixture declarations, report validation, and machine-readable schemas. Preserve the published v0.1.0 contract and attribution while allowing the TCK to depend one-way on the public devkit for A2A-LAB domain types; the devkit must not depend on this crate.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The standalone crate exposes profile 1.1.0 with the existing 47-case basic suite and a strictly larger full suite containing the 16 linked capability scenarios plus the A2A agent-message scenario
- [x] #2 Fixture and report JSON contracts retain stable case/scenario identifiers, required/skip semantics, A2A-and-MCP coverage rules, validation diagnostics, and schema identifier https://a2a-lab.dev/schemas/compliance-result-1.1.0.json
- [x] #3 Versioned fixture and result JSON Schemas are retained as machine-readable repository artifacts and automated tests verify that checked-in schemas match generated public types
- [x] #4 Public reference documentation calls this the official A2A-LAB TCK owned by A3 Analytics, distinguishes it from the upstream A2A protocol TCK, and defines suite, report, intentional envelope-difference, and non-certification boundaries
- [x] #5 Git history or repository provenance records the source devkit repository, v0.1.0/9d5327868d96b3e800fd89f6debf434bcc12709d lineage, original paths, and original authors without rewriting the target repository's template commit
- [x] #6 The target dependency graph has a one-way a2a-lab-tck to public a2a-lab-dev-kit dependency and contains no devkit-to-TCK dependency
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Extract profile 1.1.0 contract types, stable cases/scenarios, JSON validation, and schema generation while reusing only public devkit domain types pinned to the v0.1.0 lineage.
2. Add versioned fixture/result schema artifacts and compatibility tests covering identifiers, suite composition, evidence semantics, validation, JSON round trips, and schema drift.
3. Add official public TCK reference documentation and a provenance record preserving source paths, revision, and authorship.
4. Run focused tests and mise run quality, verify the one-way dependency graph, then check all acceptance criteria and complete TASK-2.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
- Extracted profile 1.1.0 with all 47 published case IDs and 17 full-suite scenario IDs; focused tests verify basic/full composition, JSON round trips, required/skip and LLM evidence validation, and field-level diagnostics.
- Added generated fixture/result JSON Schemas with drift tests, the official public A3 Analytics TCK profile specification, and PROVENANCE for v0.1.0 commit 9d5327868d96b3e800fd89f6debf434bcc12709d and original paths/authorship.

- Verified cargo tree: a2a-lab-tck depends on the public devkit pinned to v0.1.0 lineage commit 9d532786; the devkit Cargo manifest contains no TCK dependency.

- Verification: mise run quality passed (format, check, Clippy with warnings denied, and 6/6 nextest tests); schema drift tests and cargo tree evidence passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Extracted the published A2A-LAB profile 1.1.0 contract into the standalone TCK with stable 47-case basic and 17-scenario full composition, fixture/report validation, versioned schemas, public reference documentation, and source attribution.

Pinned the one-way public devkit dependency to v0.1.0 lineage commit 9d5327868d96b3e800fd89f6debf434bcc12709d. mise run quality passes with 6 tests.
<!-- SECTION:FINAL_SUMMARY:END -->
