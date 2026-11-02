//! Runs the standalone compliance CLI against local dev-kit-backed endpoints.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use serde::Serialize;

use crate::{COMPLIANCE_PROFILE_VERSION, ComplianceResult, ComplianceSuite};

use super::lab::FixtureVariant;
use super::report::{contract_report, interface_differences, unexplained_interface_differences};
use super::server::{FixtureMode, FixtureServer};

pub const DEVKIT_REVISION: &str = "64ea97d3edcac8f51ba802dbed7fc4a595aec763";
pub const PROFILE_VERSION: &str = COMPLIANCE_PROFILE_VERSION;

#[derive(Debug, Clone, Copy)]
pub struct MatrixCase {
    pub name: &'static str,
    pub mode: MatrixMode,
    pub suite: ComplianceSuite,
    pub llm_check: bool,
    pub expect_compliant: bool,
    pub timeout_milliseconds: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixMode {
    Variant(FixtureVariant),
    UnavailableCapabilities,
    EndpointFailure,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatrixOutcome {
    pub case: &'static str,
    pub variant: &'static str,
    pub suite: &'static str,
    pub llm_check: bool,
    pub cli_exit: i32,
    pub compliant: bool,
    pub contract: serde_json::Value,
    pub interface_differences: Vec<String>,
    pub unexplained_interface_differences: Vec<String>,
}

pub async fn run_matrix(
    evidence: &Path,
    compliance_bin: &Path,
) -> Result<Vec<MatrixOutcome>, String> {
    std::fs::create_dir_all(evidence.join("reports"))
        .map_err(|error| format!("create reports directory: {error}"))?;
    let mut outcomes = Vec::new();
    for case in MATRIX {
        let outcome = run_case(evidence, compliance_bin, *case).await?;
        outcomes.push(outcome);
    }
    Ok(outcomes)
}

const MATRIX: &[MatrixCase] = &[
    MatrixCase {
        name: "basic-compliant",
        mode: MatrixMode::Variant(FixtureVariant::Standard),
        suite: ComplianceSuite::Basic,
        llm_check: false,
        expect_compliant: true,
        timeout_milliseconds: 5_000,
    },
    MatrixCase {
        name: "full-default-llm",
        mode: MatrixMode::Variant(FixtureVariant::Standard),
        suite: ComplianceSuite::Full,
        llm_check: true,
        expect_compliant: true,
        timeout_milliseconds: 5_000,
    },
    MatrixCase {
        name: "full-llm-opt-out",
        mode: MatrixMode::Variant(FixtureVariant::Standard),
        suite: ComplianceSuite::Full,
        llm_check: false,
        expect_compliant: true,
        timeout_milliseconds: 5_000,
    },
    MatrixCase {
        name: "full-noncompliant",
        mode: MatrixMode::Variant(FixtureVariant::McpMetricError),
        suite: ComplianceSuite::Full,
        llm_check: true,
        expect_compliant: false,
        timeout_milliseconds: 5_000,
    },
    MatrixCase {
        name: "a2a-mcp-mismatch",
        mode: MatrixMode::Variant(FixtureVariant::McpEmptyLogSources),
        suite: ComplianceSuite::Full,
        llm_check: true,
        expect_compliant: false,
        timeout_milliseconds: 5_000,
    },
    MatrixCase {
        name: "unavailable-capabilities",
        mode: MatrixMode::UnavailableCapabilities,
        suite: ComplianceSuite::Basic,
        llm_check: false,
        expect_compliant: true,
        timeout_milliseconds: 5_000,
    },
    MatrixCase {
        name: "timeout-or-endpoint-failure",
        mode: MatrixMode::EndpointFailure,
        suite: ComplianceSuite::Basic,
        llm_check: false,
        expect_compliant: false,
        timeout_milliseconds: 500,
    },
];

async fn run_case(
    evidence: &Path,
    compliance_bin: &Path,
    case: MatrixCase,
) -> Result<MatrixOutcome, String> {
    let fixture_mode = match case.mode {
        MatrixMode::Variant(variant) => FixtureMode::Variant(variant),
        MatrixMode::UnavailableCapabilities => FixtureMode::UnavailableCapabilities,
        MatrixMode::EndpointFailure => FixtureMode::Variant(FixtureVariant::Standard),
    };
    let server = FixtureServer::start(fixture_mode)
        .await
        .map_err(|error| format!("{} fixture: {error}", case.name))?;
    let fixtures_path = evidence.join(format!("{}-fixtures.json", case.name));
    let report_path = evidence.join("reports").join(format!("{}.json", case.name));
    let fixture_json = serde_json::to_string_pretty(&server.fixtures)
        .map_err(|error| format!("{} fixtures encode: {error}", case.name))?;
    std::fs::write(&fixtures_path, format!("{fixture_json}\n"))
        .map_err(|error| format!("{} fixtures write: {error}", case.name))?;

    let mcp_url = if case.mode == MatrixMode::EndpointFailure {
        "http://127.0.0.1:1/mcp".to_owned()
    } else {
        server.mcp_url.clone()
    };

    let status = invoke_compliance(&ComplianceInvoke {
        binary: compliance_bin,
        a2a_url: &server.a2a_url,
        mcp_url: &mcp_url,
        fixtures: &fixtures_path,
        report: &report_path,
        suite: case.suite,
        llm_check: case.llm_check,
        timeout_milliseconds: case.timeout_milliseconds,
    })
    .map_err(|error| format!("{} invoke: {error}", case.name))?;
    server.shutdown().await;

    let report_json = std::fs::read_to_string(&report_path)
        .map_err(|error| format!("{} report read: {error}", case.name))?;
    let report = ComplianceResult::from_json(&report_json)
        .map_err(|error| format!("{} report parse: {error}", case.name))?;
    let differences = interface_differences(&report);
    let unexplained = unexplained_interface_differences(case.name, &report);
    let contract = contract_report(&report);
    let cli_exit = status.code().unwrap_or(-1);
    if report.compliant != case.expect_compliant {
        return Err(format!(
            "{} compliance mismatch: expected compliant={} got {} (exit {cli_exit})",
            case.name, case.expect_compliant, report.compliant
        ));
    }
    if case.mode == MatrixMode::EndpointFailure {
        let transport_or_timeout = report.cases.iter().any(|item| {
            item.diagnostics
                .iter()
                .any(|message| message.contains("transport") || message.contains("timeout"))
        });
        if !transport_or_timeout {
            return Err(format!(
                "{} must report a timeout or transport failure without aborting",
                case.name
            ));
        }
    }
    Ok(MatrixOutcome {
        case: case.name,
        variant: mode_name(case.mode),
        suite: suite_name(case.suite),
        llm_check: case.llm_check,
        cli_exit,
        compliant: report.compliant,
        contract,
        interface_differences: differences,
        unexplained_interface_differences: unexplained,
    })
}

struct ComplianceInvoke<'a> {
    binary: &'a Path,
    a2a_url: &'a str,
    mcp_url: &'a str,
    fixtures: &'a Path,
    report: &'a Path,
    suite: ComplianceSuite,
    llm_check: bool,
    timeout_milliseconds: u64,
}

fn invoke_compliance(invoke: &ComplianceInvoke<'_>) -> Result<ExitStatus, String> {
    let mut command = Command::new(invoke.binary);
    command.args([
        "--a2a-url",
        invoke.a2a_url,
        "--mcp-url",
        invoke.mcp_url,
        "--fixtures",
        invoke
            .fixtures
            .to_str()
            .ok_or("fixtures path is not UTF-8")?,
        "--report",
        invoke.report.to_str().ok_or("report path is not UTF-8")?,
        "--implementation-name",
        "a2a-lab-dev-kit-fixture",
        "--implementation-version",
        DEVKIT_REVISION,
        "--profile",
        PROFILE_VERSION,
        "--suite",
        suite_name(invoke.suite),
        "--timeout-milliseconds",
        &invoke.timeout_milliseconds.to_string(),
    ]);
    if !invoke.llm_check {
        command.arg("--no-llm-check");
    }
    command
        .status()
        .map_err(|error| format!("spawn compliance binary: {error}"))
}

pub fn run_invalid_configuration(compliance_bin: &Path) -> Result<i32, String> {
    let status = Command::new(compliance_bin)
        .arg("--suite")
        .arg("invalid")
        .status()
        .map_err(|error| format!("spawn invalid suite check: {error}"))?;
    Ok(status.code().unwrap_or(-1))
}

fn suite_name(suite: ComplianceSuite) -> &'static str {
    match suite {
        ComplianceSuite::Basic => "basic",
        ComplianceSuite::Full => "full",
    }
}

fn mode_name(mode: MatrixMode) -> &'static str {
    match mode {
        MatrixMode::Variant(FixtureVariant::Standard) => "standard",
        MatrixMode::Variant(FixtureVariant::McpMetricError) => "mcp-metric-error",
        MatrixMode::Variant(FixtureVariant::McpEmptyLogSources) => "mcp-empty-log-sources",
        MatrixMode::UnavailableCapabilities => "unavailable-capabilities",
        MatrixMode::EndpointFailure => "endpoint-failure",
    }
}

#[must_use]
pub fn evidence_dir(root: &Path) -> PathBuf {
    root.join("target/compatibility")
}
