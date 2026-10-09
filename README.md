# A2A-LAB Technology Compatibility Kit

This is the authoritative
[`A3Analytics/a2a-lab-tck`](https://github.com/A3Analytics/a2a-lab-tck)
repository for reusable conformance tooling for the A2A-LAB profile.

This project is not the upstream A2A protocol TCK. The upstream TCK validates
the base A2A protocol; this repository validates the additional behavior and
compatibility requirements defined by A2A-LAB. Passing either TCK does not
imply passing the other.

## GitHub Action

The repository-root composite Action runs profile 1.1.0 against deterministic
A2A and MCP endpoints started by the consumer. It inherits the published
devkit v0.1.0 Action contract:

- required `a2a-url`, `mcp-url`, and `fixtures` inputs;
- optional `profile`, `suite`, `llm-check`, `report-path`,
  `timeout-milliseconds`, `implementation-name`, and
  `implementation-version` inputs;
- `profile`, `suite`, `llm-check`, `compliant`, and `report-path` outputs.

The default is the `full` suite with the deterministic LLM check enabled.
Set `suite: basic` for operation cases only or `llm-check: "false"` to omit
the LLM check without model credentials. A completed compliant run exits 0; a
completed noncompliant run exits 1 after retaining its report and outputs;
invalid setup or configuration exits 2.

Copy
`.github/examples/a2a-lab-compliance.yml` into the consumer repository. Replace
its 40-character placeholder with the authorized standalone TCK release commit
and keep the full SHA pin. The workflow needs no sibling checkout or private
token.

The badge belongs to the consumer repository's A2A-LAB compliance workflow.
Its status is bounded to the tested consumer commit, selected profile, suite
and checks, and immutable TCK revision. It is not certification and does not
represent upstream A2A protocol TCK status.

## Setup

Install the tools pinned by `mise.toml`:

```bash
mise install
```

## Development

Run the complete local quality gate:

```bash
mise run quality
```

Individual tasks are available for focused work:

```bash
mise run fmt-check
mise run check
mise run clippy
mise run test
```

Use `mise tasks` for the full task list. Backlog.md project documentation lives
under `backlog/docs/`; do not add loose Markdown documentation.

## Release provenance

The first standalone release carries profile 1.1.0 and the inherited devkit
v0.1.0 Action interface. `PROVENANCE` records the source release and immutable
source revision. Preparing and validating these assets does not authorize a
push, tag, or GitHub release; publication remains an explicit external
authorization checkpoint.