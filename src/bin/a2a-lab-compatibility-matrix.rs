use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use a2a_lab_tck::compatibility::{
    DEVKIT_REVISION, PROFILE_VERSION, devkit_manifest_independent_of_tck, evidence_dir,
    pinned_devkit_manifest_path, run_invalid_configuration, run_matrix,
};

#[tokio::main]
async fn main() -> ExitCode {
    let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let evidence = evidence_dir(&root);
    let compliance_bin = env::var("A2ALAB_COMPLIANCE_BIN").map_or_else(
        |_| root.join("target/compatibility/build/a2a-lab-compliance"),
        PathBuf::from,
    );
    if let Err(error) = execute(&root, &evidence, &compliance_bin).await {
        eprintln!("{error}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

async fn execute(root: &Path, evidence: &Path, compliance_bin: &Path) -> Result<(), String> {
    verify_devkit_revision(root)?;
    let devkit_independent = devkit_manifest_independent_of_tck(root)?;
    if !devkit_independent {
        return Err(
            "pinned a2a-lab-dev-kit manifest depends on a2a-lab-tck; compatibility cannot pass"
                .to_owned(),
        );
    }
    let devkit_manifest_path = pinned_devkit_manifest_path(root)?.display().to_string();
    let matrix = run_matrix(evidence, compliance_bin).await?;
    let invalid_exit = run_invalid_configuration(compliance_bin)?;
    if invalid_exit != 2 {
        return Err(format!(
            "invalid configuration exit mismatch: {invalid_exit}"
        ));
    }
    let unexplained: Vec<String> = matrix
        .iter()
        .flat_map(|item| item.unexplained_interface_differences.iter().cloned())
        .collect();
    write_summary(
        root,
        evidence,
        &matrix,
        invalid_exit,
        devkit_independent,
        &devkit_manifest_path,
        &unexplained,
    )?;
    if !unexplained.is_empty() {
        return Err(format!(
            "unexplained normalized interface differences: {}",
            unexplained.join("; ")
        ));
    }
    Ok(())
}

fn verify_devkit_revision(root: &Path) -> Result<(), String> {
    let lock = std::fs::read_to_string(root.join("Cargo.lock"))
        .map_err(|error| format!("read Cargo.lock: {error}"))?;
    if !lock.contains(DEVKIT_REVISION) {
        return Err(format!(
            "Cargo.lock does not pin dev-kit revision {DEVKIT_REVISION}"
        ));
    }
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))
        .map_err(|error| format!("read Cargo.toml: {error}"))?;
    if !manifest.contains(DEVKIT_REVISION) {
        return Err("Cargo.toml does not pin the current dev-kit revision".to_owned());
    }
    if manifest
        .lines()
        .any(|line| line.starts_with("a2a-lab-tck ="))
    {
        return Err("Cargo.toml must not depend on a2a-lab-tck".to_owned());
    }
    Ok(())
}

fn write_summary(
    root: &Path,
    evidence: &Path,
    matrix: &[a2a_lab_tck::compatibility::MatrixOutcome],
    invalid_exit: i32,
    devkit_manifest_independent_of_tck: bool,
    devkit_manifest_path: &str,
    unexplained: &[String],
) -> Result<(), String> {
    let standalone_revision = git_head(root)?;
    let dirty = !git_porcelain(root)?.trim().is_empty();
    let summary = serde_json::json!({
        "devkit_revision": DEVKIT_REVISION,
        "profile_version": PROFILE_VERSION,
        "standalone_revision": standalone_revision,
        "standalone_worktree_dirty": dirty,
        "devkit_manifest_independent_of_tck": devkit_manifest_independent_of_tck,
        "devkit_manifest_path": devkit_manifest_path,
        "network_model_services": false,
        "docker": false,
        "hardware": false,
        "ot2_required": false,
        "invalid_configuration_exit": invalid_exit,
        "matrix": matrix,
        "unexplained_differences": unexplained,
        "evidence": "Model services, Docker, hardware, and OT-2 are not required for this compatibility run.",
        "normalization": "Repeated runs with the same revisions normalize RFC3339 timestamps in diagnostics to <timestamp> inside contract_report.",
    });
    std::fs::write(
        evidence.join("summary.json"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&summary).map_err(|error| error.to_string())?
        ),
    )
    .map_err(|error| format!("write summary.json: {error}"))?;
    println!("{}", serde_json::to_string_pretty(&summary).unwrap());
    Ok(())
}

fn git_head(root: &Path) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["-C", root.to_str().unwrap(), "rev-parse", "HEAD"])
        .output()
        .map_err(|error| format!("git rev-parse: {error}"))?;
    if !output.status.success() {
        return Err("git rev-parse failed".to_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn git_porcelain(root: &Path) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["-C", root.to_str().unwrap(), "status", "--porcelain"])
        .output()
        .map_err(|error| format!("git status: {error}"))?;
    if !output.status.success() {
        return Err("git status failed".to_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
