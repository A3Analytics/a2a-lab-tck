---
id: doc-1
title: A2A-LAB TCK profile
type: specification
audience: public
created_date: '2026-10-09 20:05'
---

# A2A-LAB TCK profile

The A2A-LAB TCK is the official A3 Analytics Technology Compatibility Kit for the A2A-LAB conformance profile. It is distinct from the upstream A2A protocol TCK: upstream protocol conformance is an input to A2A-LAB compatibility, while this TCK verifies lab operations and their cross-interface behavior.

## Profile 1.1.0 suites

The `basic` suite contains the 47 stable operation-level cases from profile 1.0.0. Every required case exercises equivalent typed lab behavior over both A2A and MCP.

The `full` suite is a strict superset. It contains all 47 basic cases and four producer-to-consumer paths—A2A→A2A, MCP→MCP, A2A→MCP, and MCP→A2A—for each of logs, metrics, tasks, and images. Those 16 linked scenarios test identifier handoff, successful availability, and task progress where applicable.

A full run may also enable `a2a_agent_message`. Its `agent-message.a2a` evidence confirms advertisement, MCP task discovery, an A2A request grounded in the discovered task identifier, and a nonblank A2A context identifier. It does not prove a particular internal tool call, plan quality, general answer quality, or reliability on other prompts. Opting out omits the check and its scenario result; it does not report a skip.

## Fixtures and reports

A fixture declaration selects profile `1.1.0`, a half-open time range, and deterministic identifiers for logs, metrics, tasks, and images. Each capability is either `required` or `unavailable`. Required capabilities make their success, boundary, invalid, and not-found cases required while skipping unavailable cases. Unavailable capabilities require unavailable cases and skip the others.

Reports contain every stable case identifier exactly once. Required cases cover both A2A and MCP and cannot be skipped in a compliant report. Failed and skipped cases require diagnostics. Full reports also contain every enabled linked scenario with its exact producer-to-consumer handoff; enabling the LLM check requires `agent-message.a2a` evidence.

The versioned machine-readable artifacts are:

- `schemas/compliance-fixtures-1.1.0.json`
- `schemas/compliance-result-1.1.0.json`

The result schema identifier is `https://a2a-lab.dev/schemas/compliance-result-1.1.0.json`.

## Intentional envelope differences

Parity compares shared lab values after removing protocol transport envelopes. A2A returns task artifacts while MCP returns structured tool content; each protocol uses its native error and status envelope; MCP may repeat image base64 in text and structured content; and protocol task or context identifiers are not shared A2A-LAB result fields.

## Boundaries

The TCK verifies only the deterministic behavior represented by the selected suite and enabled checks. It does not establish certification, exhaustive application correctness, hardware correctness, model-provider quality, or upstream A2A protocol TCK status. Passing an A2A-LAB suite does not imply passing the upstream A2A TCK, and vice versa.
