#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
devkit_revision="64ea97d3edcac8f51ba802dbed7fc4a595aec763"
historical_extraction="9d5327868d96b3e800fd89f6debf434bcc12709d"
profile_version="1.3.0"
evidence="$root/target/compatibility"
build_root="$evidence/build"

mkdir -p "$evidence" "$build_root"

if ! grep -q "$devkit_revision" "$root/Cargo.lock"; then
  printf 'Cargo.lock does not pin dev-kit revision %s\n' "$devkit_revision" >&2
  exit 1
fi
if ! grep -q "$devkit_revision" "$root/Cargo.toml"; then
  printf 'Cargo.toml does not pin dev-kit revision %s\n' "$devkit_revision" >&2
  exit 1
fi
if grep -q '^a2a-lab-tck =' "$root/Cargo.toml"; then
  printf 'Cargo.toml must not depend on a2a-lab-tck\n' >&2
  exit 1
fi
if ! grep -q "$historical_extraction" "$root/PROVENANCE"; then
  printf 'PROVENANCE must retain historical extraction commit %s\n' "$historical_extraction" >&2
  exit 1
fi
if ! grep -q "$devkit_revision" "$root/PROVENANCE"; then
  printf 'PROVENANCE must record current dev-kit compatibility revision %s\n' "$devkit_revision" >&2
  exit 1
fi
if grep -q '928eae822318f650f8fe2f187e99091d4b46647b' "$root/PROVENANCE"; then
  printf 'PROVENANCE must not present the superseded typed-seam revision as active\n' >&2
  exit 1
fi

CARGO_TARGET_DIR="$build_root" mise exec -- cargo build --offline --locked \
  --manifest-path "$root/Cargo.toml" \
  --bin a2a-lab-compliance \
  --bin a2a-lab-compatibility-matrix

export A2ALAB_COMPLIANCE_BIN="$build_root/debug/a2a-lab-compliance"
mise exec -- cargo run --offline --locked \
  --manifest-path "$root/Cargo.toml" \
  --bin a2a-lab-compatibility-matrix \
  --quiet

# Record devkit_manifest_independent_of_tck using cargo metadata on the pinned git checkout.
python3 - "$evidence/summary.json" "$root" "$devkit_revision" <<'PY'
import hashlib
import json
import os
import pathlib
import subprocess
import sys

summary_path = pathlib.Path(sys.argv[1])
root = pathlib.Path(sys.argv[2])
devkit_revision = sys.argv[3]
summary = json.loads(summary_path.read_text())
source_files = [
    root / "Cargo.toml",
    root / "Cargo.lock",
    root / "action.yml",
    root / "PROVENANCE",
    root / ".github/actions/a2a-lab-compliance/run.sh",
    root / ".mise/scripts/compatibility.sh",
]
for directory in ("schemas", "src", "tests"):
    source_files.extend(path for path in (root / directory).rglob("*") if path.is_file())
source_hash = hashlib.sha256()
for path in sorted(source_files):
    source_hash.update(path.relative_to(root).as_posix().encode())
    source_hash.update(b"\0")
    source_hash.update(path.read_bytes())
    source_hash.update(b"\0")
summary["standalone_source_sha256"] = source_hash.hexdigest()
summary["historical_extraction_revision"] = "9d5327868d96b3e800fd89f6debf434bcc12709d"

def resolve_devkit_manifest_path():
    try:
        metadata = json.loads(
            subprocess.check_output(
                [
                    "cargo",
                    "metadata",
                    "--offline",
                    "--locked",
                    "--format-version",
                    "1",
                    "--manifest-path",
                    str(root / "Cargo.toml"),
                ],
                cwd=root,
                text=True,
            )
        )
        package = next(
            item for item in metadata["packages"] if item["name"] == "a2a-lab-dev-kit"
        )
        source = package.get("source")
        if not source or devkit_revision not in source:
            raise RuntimeError(
                f"resolved a2a-lab-dev-kit source `{source}` does not match {devkit_revision}"
            )
        return pathlib.Path(package["manifest_path"])
    except (subprocess.CalledProcessError, RuntimeError):
        short = devkit_revision[:7]
        cargo_home = pathlib.Path(os.environ.get("CARGO_HOME", pathlib.Path.home() / ".cargo"))
        checkouts = cargo_home / "git/checkouts"
        for repo in sorted(checkouts.iterdir()):
            candidate = repo / short / "Cargo.toml"
            if not candidate.is_file():
                continue
            manifest = candidate.read_text()
            if 'name = "a2a-lab-dev-kit"' in manifest:
                return candidate
        raise SystemExit(
            f"could not resolve pinned dev-kit manifest for {devkit_revision} via cargo metadata or git checkout"
        )

manifest_path = resolve_devkit_manifest_path()
devkit_manifest = manifest_path.read_text()
if any(
    line.split("#", 1)[0].strip().startswith("a2a-lab-tck =")
    or line.split("#", 1)[0].strip().startswith("a2a-lab-tck=")
    for line in devkit_manifest.splitlines()
):
    summary["devkit_manifest_independent_of_tck"] = False
    summary_path.write_text(json.dumps(summary, indent=2) + "\n")
    raise SystemExit("pinned a2a-lab-dev-kit manifest depends on a2a-lab-tck")
summary["devkit_manifest_independent_of_tck"] = True
summary["devkit_manifest_path"] = str(manifest_path)
summary_path.write_text(json.dumps(summary, indent=2) + "\n")
if summary.get("unexplained_differences"):
    raise SystemExit(json.dumps(summary["unexplained_differences"], indent=2))
if summary["devkit_revision"] != devkit_revision:
    raise SystemExit("summary dev-kit revision mismatch")
if summary.get("profile_version") != "1.3.0":
    raise SystemExit("summary profile mismatch")
print(json.dumps(summary, indent=2))
PY
