//! Pinned dev-kit manifest checks for compatibility evidence.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::DEVKIT_REVISION;

pub fn devkit_manifest_independent_of_tck(root: &Path) -> Result<bool, String> {
    let manifest = read_pinned_devkit_manifest(root)?;
    Ok(!manifest.lines().any(|line| {
        let trimmed = line.split('#').next().unwrap_or(line).trim();
        trimmed.starts_with("a2a-lab-tck =") || trimmed.starts_with("a2a-lab-tck=")
    }))
}

pub fn pinned_devkit_manifest_path(root: &Path) -> Result<PathBuf, String> {
    read_pinned_devkit_manifest_path(root)
}

fn read_pinned_devkit_manifest(root: &Path) -> Result<String, String> {
    let path = read_pinned_devkit_manifest_path(root)?;
    std::fs::read_to_string(&path)
        .map_err(|error| format!("read dev-kit manifest `{}`: {error}", path.display()))
}

fn read_pinned_devkit_manifest_path(root: &Path) -> Result<PathBuf, String> {
    if let Ok(path) = devkit_manifest_path_from_metadata(root) {
        return Ok(path);
    }
    devkit_manifest_path_from_git_checkout(root)
}

fn devkit_manifest_path_from_metadata(root: &Path) -> Result<PathBuf, String> {
    let manifest_path = root.join("Cargo.toml");
    let manifest = manifest_path
        .to_str()
        .ok_or("Cargo.toml path is not UTF-8")?;
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--offline",
            "--locked",
            "--format-version",
            "1",
            "--manifest-path",
            manifest,
        ])
        .current_dir(root)
        .output()
        .map_err(|error| format!("cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("decode cargo metadata: {error}"))?;
    let packages = metadata
        .get("packages")
        .and_then(|value| value.as_array())
        .ok_or("cargo metadata missing packages")?;
    let package = packages
        .iter()
        .find(|package| {
            package.get("name").and_then(|value| value.as_str()) == Some("a2a-lab-dev-kit")
        })
        .ok_or("cargo metadata missing a2a-lab-dev-kit package")?;
    verify_devkit_source(package)?;
    let path = package
        .get("manifest_path")
        .and_then(|value| value.as_str())
        .ok_or("a2a-lab-dev-kit manifest_path missing from metadata")?;
    Ok(PathBuf::from(path))
}

fn verify_devkit_source(package: &serde_json::Value) -> Result<(), String> {
    let revision = package
        .get("source")
        .and_then(|value| value.as_str())
        .ok_or("a2a-lab-dev-kit source missing from metadata")?;
    if !revision.contains(DEVKIT_REVISION) {
        return Err(format!(
            "resolved a2a-lab-dev-kit revision `{revision}` does not match {DEVKIT_REVISION}"
        ));
    }
    Ok(())
}

fn devkit_manifest_path_from_git_checkout(root: &Path) -> Result<PathBuf, String> {
    let lock = std::fs::read_to_string(root.join("Cargo.lock"))
        .map_err(|error| format!("read Cargo.lock: {error}"))?;
    if !lock.contains(DEVKIT_REVISION) {
        return Err(format!(
            "Cargo.lock does not pin dev-kit revision {DEVKIT_REVISION}"
        ));
    }
    let short = &DEVKIT_REVISION[..7];
    let cargo_home = std::env::var("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::var("HOME")
                .map(|home| PathBuf::from(home).join(".cargo"))
                .map_err(|error| format!("CARGO_HOME and HOME unset: {error}"))
        })?;
    let checkouts = cargo_home.join("git/checkouts");
    let entries = std::fs::read_dir(&checkouts)
        .map_err(|error| format!("read `{}`: {error}", checkouts.display()))?;
    for repo in entries {
        let repo = repo.map_err(|error| format!("list git checkouts: {error}"))?;
        if !repo.file_type().is_ok_and(|kind| kind.is_dir()) {
            continue;
        }
        let candidate = repo.path().join(short).join("Cargo.toml");
        if !candidate.is_file() {
            continue;
        }
        let manifest = std::fs::read_to_string(&candidate)
            .map_err(|error| format!("read `{}`: {error}", candidate.display()))?;
        if manifest_contains_devkit_crate_name(&manifest) {
            return Ok(candidate);
        }
    }
    Err(format!(
        "could not locate dev-kit manifest for revision {DEVKIT_REVISION} under `{}`",
        checkouts.display()
    ))
}

fn manifest_contains_devkit_crate_name(manifest: &str) -> bool {
    manifest.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "name = \"a2a-lab-dev-kit\"" || trimmed == "name = 'a2a-lab-dev-kit'"
    })
}
