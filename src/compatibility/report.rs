//! Normalized compliance report comparison for matrix validation.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::{ComplianceResult, ComplianceSuite, compliance_profile, compliance_suite};

#[must_use]
pub fn contract_report(report: &ComplianceResult) -> Value {
    serde_json::json!({
        "suite_version": report.suite_version,
        "selected_suite": report.selected_suite,
        "enabled_checks": report.enabled_checks,
        "compliant": report.compliant,
        "cases": report.cases.iter().map(|item| {
            serde_json::json!({
                "id": item.case_id,
                "required": item.required,
                "outcome": item.outcome,
                "diagnostics": item.diagnostics.iter().map(|value| normalize_diagnostics(value)).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
        "scenarios": report.scenarios.iter().map(|item| {
            serde_json::json!({
                "id": item.scenario_id,
                "required": item.required,
                "outcome": item.outcome,
                "diagnostics": item.diagnostics.iter().map(|value| normalize_diagnostics(value)).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
    })
}

fn normalize_diagnostics(value: &str) -> String {
    let mut normalized = value.to_owned();
    while let Some(start) = normalized.find("20") {
        let slice = &normalized[start..];
        if slice.len() >= 20 && slice.as_bytes().get(19) == Some(&b'Z') {
            normalized.replace_range(start..start + 20, "<timestamp>");
            continue;
        }
        if slice.len() >= 27
            && slice.as_bytes().get(19) == Some(&b'.')
            && slice.as_bytes().get(26) == Some(&b'Z')
        {
            normalized.replace_range(start..start + 27, "<timestamp>");
            continue;
        }
        break;
    }
    normalized
}

#[must_use]
pub fn interface_differences(report: &ComplianceResult) -> Vec<String> {
    let mut differences = Vec::new();
    for case in &report.cases {
        for diagnostic in &case.diagnostics {
            if diagnostic.contains("normalized interface difference") {
                differences.push(format!("{}: {diagnostic}", case.case_id));
            }
        }
    }
    for scenario in &report.scenarios {
        for diagnostic in &scenario.diagnostics {
            if diagnostic.contains("normalized interface difference") {
                differences.push(format!("{}: {diagnostic}", scenario.scenario_id));
            }
        }
    }
    differences
}

#[must_use]
pub fn is_explained_mismatch_matrix_row(case: &str) -> bool {
    matches!(
        case,
        "full-noncompliant" | "a2a-mcp-mismatch" | "timeout-or-endpoint-failure"
    )
}

#[must_use]
pub fn unexplained_interface_differences(case: &str, report: &ComplianceResult) -> Vec<String> {
    if is_explained_mismatch_matrix_row(case) {
        return Vec::new();
    }
    interface_differences(report)
}

pub fn video_case_ids() -> BTreeSet<&'static str> {
    BTreeSet::from([
        "videos.clip.long-range",
        "videos.timeline.success",
        "videos.follow.live-edge",
        "videos.clip.empty-range",
        "videos.timeline.missing-source",
        "videos.follow.missing-source",
        "videos.follow.live-unavailable",
        "videos.clip.unavailable",
    ])
}

#[must_use]
pub fn profile_has_video_cases() -> bool {
    let ids: BTreeSet<_> = compliance_profile()
        .cases
        .iter()
        .map(|case| case.id)
        .collect();
    video_case_ids().iter().all(|case_id| ids.contains(case_id))
}

#[must_use]
pub fn full_suite_includes_agent_message() -> bool {
    compliance_suite(ComplianceSuite::Full)
        .scenarios
        .iter()
        .any(|scenario| scenario.id == "agent-message.a2a")
}
