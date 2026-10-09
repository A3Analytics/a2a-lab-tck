#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
devkit_root="${A2ALAB_DEV_KIT_ROOT:-$root/../a2a-lab-dev-kit-rs}"
ot2_root="${A2ALAB_OT2_ROOT:-$root/../a2a-lab-ot2}"
devkit_revision="9d5327868d96b3e800fd89f6debf434bcc12709d"
seam_revision="928eae822318f650f8fe2f187e99091d4b46647b"
standalone_revision="$(git -C "$root" rev-parse HEAD)"
work="$(mktemp -d)"
evidence="$root/target/compatibility"
build_root="$evidence/build"
fixture_pid=""

cleanup() {
  if [[ -n "$fixture_pid" ]] && kill -0 "$fixture_pid" 2>/dev/null; then
    kill -INT "$fixture_pid" 2>/dev/null || true
    wait "$fixture_pid" 2>/dev/null || true
  fi
  rm -rf "$work"
}
trap cleanup EXIT

for repository in "$devkit_root" "$ot2_root"; do
  [[ -d "$repository/.git" ]] || {
    printf 'compatibility repository not found: %s\n' "$repository" >&2
    exit 2
  }
done
git -C "$devkit_root" cat-file -e "$devkit_revision^{commit}"
git -C "$devkit_root" cat-file -e "$seam_revision^{commit}"

mkdir -p "$work/devkit" "$evidence" "$build_root"
rm -rf "$evidence/reports"
mkdir -p "$evidence/reports"
git -C "$devkit_root" archive "$devkit_revision" | tar -x -C "$work/devkit"

CARGO_TARGET_DIR="$build_root/devkit" \
  mise exec -- cargo build --offline --locked \
  --manifest-path "$work/devkit/Cargo.toml" --bin a2a-lab-compliance
CARGO_TARGET_DIR="$build_root/tck" \
  mise exec -- cargo build --offline --locked \
  --manifest-path "$root/Cargo.toml" --bin a2a-lab-compliance
CARGO_TARGET_DIR="$build_root/ot2" \
  mise exec -- cargo build --offline --locked \
  --manifest-path "$ot2_root/Cargo.toml" --bin a2a-lab-ot2

devkit_bin="$build_root/devkit/debug/a2a-lab-compliance"
tck_bin="$build_root/tck/debug/a2a-lab-compliance"
ot2_bin="$build_root/ot2/debug/a2a-lab-ot2"

stop_fixture() {
  if [[ -n "$fixture_pid" ]] && kill -0 "$fixture_pid" 2>/dev/null; then
    kill -INT "$fixture_pid" 2>/dev/null || true
    wait "$fixture_pid" 2>/dev/null || true
  fi
  fixture_pid=""
}

run_case() {
  local implementation="$1"
  local binary="$2"
  local case_name="$3"
  local variant="$4"
  local suite="$5"
  local llm_check="$6"
  local readiness="$work/$implementation-$case_name-readiness.json"
  local fixture_log="$work/$implementation-$case_name-fixture.log"
  local fixtures="$work/$implementation-$case_name-fixtures.json"
  local report="$evidence/reports/$implementation-$case_name.json"
  local status_file="$evidence/reports/$implementation-$case_name.status"

  "$ot2_bin" fixture --variant "$variant" >"$readiness" 2>"$fixture_log" &
  fixture_pid=$!
  for _ in $(seq 1 200); do
    [[ -s "$readiness" ]] && break
    if ! kill -0 "$fixture_pid" 2>/dev/null; then
      cat "$fixture_log" >&2
      exit 1
    fi
    sleep 0.05
  done
  [[ -s "$readiness" ]] || {
    cat "$fixture_log" >&2
    printf 'fixture did not become ready\n' >&2
    exit 1
  }

  python3 - "$readiness" "$fixtures" <<'PY'
import json
import pathlib
import sys

ready = json.loads(pathlib.Path(sys.argv[1]).read_text().splitlines()[0])
pathlib.Path(sys.argv[2]).write_text(json.dumps(ready["fixtures"]) + "\n")
PY
  local a2a_url
  local mcp_url
  a2a_url="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["a2a_url"])' "$readiness")"
  mcp_url="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["mcp_url"])' "$readiness")"
  local arguments=(
    --a2a-url "$a2a_url"
    --mcp-url "$mcp_url"
    --fixtures "$fixtures"
    --report "$report"
    --implementation-name a2a-lab-ot2
    --implementation-version compatibility-fixture
    --profile 1.1.0
    --suite "$suite"
    --timeout-milliseconds 2000
  )
  if [[ "$llm_check" == "false" ]]; then
    arguments+=(--no-llm-check)
  fi

  set +e
  "$binary" "${arguments[@]}"
  local status=$?
  set -e
  printf '%s\n' "$status" >"$status_file"
  stop_fixture
}

while read -r case_name variant suite llm_check; do
  run_case devkit "$devkit_bin" "$case_name" "$variant" "$suite" "$llm_check"
  run_case standalone "$tck_bin" "$case_name" "$variant" "$suite" "$llm_check"
done <<'MATRIX'
basic-compliant standard basic false
full-default-llm standard full true
full-llm-opt-out standard full false
full-noncompliant mcp-metric-error full true
a2a-mcp-mismatch mcp-empty-log-sources full true
MATRIX

for implementation in devkit standalone; do
  binary="$devkit_bin"
  [[ "$implementation" == "standalone" ]] && binary="$tck_bin"
  set +e
  "$binary" --suite invalid >"$work/$implementation-invalid.stdout" \
    2>"$work/$implementation-invalid.stderr"
  status=$?
  set -e
  printf '%s\n' "$status" >"$evidence/reports/$implementation-invalid.status"
done

CARGO_TARGET_DIR="$build_root/devkit" \
  mise exec -- cargo test --offline --locked \
  --manifest-path "$work/devkit/Cargo.toml" --test compliance_runner \
  unavailable_capabilities_skip_only_nonrequired_cases
CARGO_TARGET_DIR="$build_root/devkit" \
  mise exec -- cargo test --offline --locked \
  --manifest-path "$work/devkit/Cargo.toml" --test compliance_runner \
  timeout_and_endpoint_failure_are_reported_without_aborting
CARGO_TARGET_DIR="$build_root/tck" \
  mise exec -- cargo test --offline --locked \
  --manifest-path "$root/Cargo.toml" --test compliance_runner \
  unavailable_capabilities_skip_only_nonrequired_cases
CARGO_TARGET_DIR="$build_root/tck" \
  mise exec -- cargo test --offline --locked \
  --manifest-path "$root/Cargo.toml" --test compliance_runner \
  timeout_and_endpoint_failure_are_reported_without_aborting

python3 - "$evidence" "$devkit_root" "$root" "$devkit_revision" \
  "$seam_revision" "$standalone_revision" <<'PY'
import hashlib
import json
import pathlib
import re
import subprocess
import sys

evidence = pathlib.Path(sys.argv[1])
devkit_root = pathlib.Path(sys.argv[2])
root = pathlib.Path(sys.argv[3])
devkit_revision, seam_revision, standalone_revision = sys.argv[4:]
cases = [
    "basic-compliant",
    "full-default-llm",
    "full-llm-opt-out",
    "full-noncompliant",
    "a2a-mcp-mismatch",
]

timestamp = re.compile(
    r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z"
)

def diagnostics(item):
    return [
        timestamp.sub("<timestamp>", value)
        for value in item.get("diagnostics", [])
    ]

def contract(report):
    return {
        "suite_version": report["suite_version"],
        "selected_suite": report["selected_suite"],
        "enabled_checks": report["enabled_checks"],
        "compliant": report["compliant"],
        "cases": [
            {
                "id": item["case_id"],
                "required": item["required"],
                "outcome": item["outcome"],
                "diagnostics": diagnostics(item),
            }
            for item in report["cases"]
        ],
        "scenarios": [
            {
                "id": item["scenario_id"],
                "required": item["required"],
                "outcome": item["outcome"],
                "diagnostics": diagnostics(item),
            }
            for item in report["scenarios"]
        ],
    }

matrix = []
for case in cases:
    source = json.loads((evidence / "reports" / f"devkit-{case}.json").read_text())
    standalone = json.loads(
        (evidence / "reports" / f"standalone-{case}.json").read_text()
    )
    source_status = int(
        (evidence / "reports" / f"devkit-{case}.status").read_text()
    )
    standalone_status = int(
        (evidence / "reports" / f"standalone-{case}.status").read_text()
    )
    differences = []
    if source_status != standalone_status:
        differences.append(
            {
                "contract": "CLI exit class",
                "devkit": source_status,
                "standalone": standalone_status,
            }
        )
    if contract(source) != contract(standalone):
        differences.append(
            {
                "contract": "normalized compliance report",
                "devkit": contract(source),
                "standalone": contract(standalone),
            }
        )
    matrix.append(
        {
            "case": case,
            "devkit_exit": source_status,
            "standalone_exit": standalone_status,
            "suite": standalone["selected_suite"],
            "enabled_checks": standalone["enabled_checks"],
            "compliant": standalone["compliant"],
            "differences": differences,
        }
    )

invalid = {}
for implementation in ("devkit", "standalone"):
    invalid[implementation] = int(
        (evidence / "reports" / f"{implementation}-invalid.status").read_text()
    )
if invalid != {"devkit": 2, "standalone": 2}:
    raise SystemExit(f"invalid configuration exit mismatch: {invalid}")

source_action = subprocess.check_output(
    ["git", "-C", str(devkit_root), "show", f"{devkit_revision}:action.yml"],
    text=True,
)
standalone_action = (root / "action.yml").read_text()
field_pattern = re.compile(r"^  ([a-z][a-z-]+):$", re.MULTILINE)
source_fields = field_pattern.findall(source_action)
standalone_fields = field_pattern.findall(standalone_action)
if source_fields != standalone_fields:
    raise SystemExit(
        f"Action input/output field mismatch: {source_fields} != {standalone_fields}"
    )
for value in (
    'default: "1.1.0"',
    "default: full",
    'default: "true"',
    'default: "30000"',
    'default: "a2a-lab-compliance.json"',
):
    if value not in source_action or value not in standalone_action:
        raise SystemExit(f"Action default mismatch: {value}")

source_runner = subprocess.check_output(
    [
        "git",
        "-C",
        str(devkit_root),
        "show",
        f"{devkit_revision}:.github/actions/a2a-lab-compliance/run.sh",
    ],
    text=True,
)
standalone_runner = (
    root / ".github" / "actions" / "a2a-lab-compliance" / "run.sh"
).read_text()
for value in (
    "--a2a-url",
    "--mcp-url",
    "--fixtures",
    "--profile",
    "--suite",
    "--report",
    "--timeout-milliseconds",
    "--implementation-name",
    "--implementation-version",
    "--no-llm-check",
    "exit \"$status\"",
):
    if value not in source_runner or value not in standalone_runner:
        raise SystemExit(f"Action runner contract mismatch: {value}")

manifest = (root / "Cargo.toml").read_text()
if seam_revision not in manifest:
    raise SystemExit("standalone manifest does not pin the typed-seam revision")
seam_manifest = subprocess.check_output(
    ["git", "-C", str(devkit_root), "show", f"{seam_revision}:Cargo.toml"],
    text=True,
)
if "a2a-lab-tck" in seam_manifest:
    raise SystemExit("devkit typed-seam revision depends on the standalone TCK")

unexplained = [
    {"case": item["case"], **difference}
    for item in matrix
    for difference in item["differences"]
]
source_files = [
    root / "Cargo.toml",
    root / "Cargo.lock",
    root / "action.yml",
    root / "PROVENANCE",
    root / ".github" / "actions" / "a2a-lab-compliance" / "run.sh",
    root / ".mise" / "scripts" / "compatibility.sh",
]
for directory in ("schemas", "src", "tests"):
    source_files.extend(path for path in (root / directory).rglob("*") if path.is_file())
source_hash = hashlib.sha256()
for path in sorted(source_files):
    source_hash.update(path.relative_to(root).as_posix().encode())
    source_hash.update(b"\0")
    source_hash.update(path.read_bytes())
    source_hash.update(b"\0")
result = {
    "devkit_revision": devkit_revision,
    "standalone_revision": standalone_revision,
    "standalone_source_sha256": source_hash.hexdigest(),
    "standalone_worktree_dirty": bool(
        subprocess.check_output(
            ["git", "-C", str(root), "status", "--porcelain"], text=True
        ).strip()
    ),
    "ot2_revision": subprocess.check_output(
        ["git", "-C", str(root.parent / "a2a-lab-ot2"), "rev-parse", "HEAD"],
        text=True,
    ).strip(),
    "network_model_services": False,
    "docker": False,
    "hardware": False,
    "matrix": matrix,
    "invalid_configuration": invalid,
    "unavailable_capability": "source and standalone focused runner tests passed",
    "timeout": "source and standalone focused runner tests passed",
    "action_contract": "compatible with branding and provenance differences allowed",
    "dependency_direction": f"a2a-lab-tck -> a2a-lab-dev-kit@{seam_revision}",
    "unexplained_differences": unexplained,
}
(evidence / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
if unexplained:
    raise SystemExit(json.dumps(unexplained, indent=2))
print(json.dumps(result, indent=2))
PY
