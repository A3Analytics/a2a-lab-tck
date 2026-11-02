use a2a_lab_tck::compatibility::{
    DEVKIT_REVISION, PROFILE_VERSION, contract_report, full_suite_includes_agent_message,
    profile_has_video_cases, standard_fixtures,
};

#[test]
fn compatibility_fixture_matches_profile_version() {
    let fixtures = standard_fixtures();
    assert_eq!(fixtures.suite_version, PROFILE_VERSION);
    assert!(profile_has_video_cases());
    assert!(full_suite_includes_agent_message());
}

#[test]
fn contract_report_normalizes_timestamps_in_diagnostics() {
    let fixtures = standard_fixtures();
    let report = a2a_lab_tck::ComplianceResult {
        suite_version: fixtures.suite_version,
        selected_suite: a2a_lab_tck::ComplianceSuite::Basic,
        enabled_checks: [a2a_lab_tck::ComplianceCheck::BasicOperations]
            .into_iter()
            .collect(),
        implementation: a2a_lab_tck::ImplementationIdentity {
            name: "fixture".to_owned(),
            version: "1".to_owned(),
        },
        tested_interfaces: [a2a_lab_tck::ComplianceInterface::A2a]
            .into_iter()
            .collect(),
        cases: vec![a2a_lab_tck::ComplianceCaseResult {
            case_id: "logs.list.success".to_owned(),
            required: true,
            interfaces: [
                a2a_lab_tck::ComplianceInterface::A2a,
                a2a_lab_tck::ComplianceInterface::Mcp,
            ]
            .into_iter()
            .collect(),
            outcome: a2a_lab_tck::ComplianceOutcome::Pass,
            diagnostics: vec!["seen 2026-01-01T00:00:00Z".to_owned()],
        }],
        scenarios: Vec::new(),
        compliant: true,
    };
    let contract = contract_report(&report);
    assert!(
        contract["cases"][0]["diagnostics"][0]
            .as_str()
            .is_some_and(|value| value.contains("<timestamp>"))
    );
}

#[test]
fn unexplained_differences_ignore_explained_mismatch_rows() {
    use std::collections::BTreeSet;

    use a2a_lab_tck::compatibility::{
        is_explained_mismatch_matrix_row, unexplained_interface_differences,
    };
    use a2a_lab_tck::{
        ComplianceInterface, ComplianceOutcome, ComplianceResult, ComplianceSuite,
        ImplementationIdentity,
    };

    let report = ComplianceResult {
        suite_version: PROFILE_VERSION.to_owned(),
        selected_suite: ComplianceSuite::Full,
        enabled_checks: BTreeSet::default(),
        implementation: ImplementationIdentity {
            name: "fixture".to_owned(),
            version: "1".to_owned(),
        },
        tested_interfaces: [ComplianceInterface::A2a, ComplianceInterface::Mcp]
            .into_iter()
            .collect(),
        cases: vec![a2a_lab_tck::ComplianceCaseResult {
            case_id: "metrics.query.success".to_owned(),
            required: true,
            interfaces: [ComplianceInterface::A2a, ComplianceInterface::Mcp]
                .into_iter()
                .collect(),
            outcome: ComplianceOutcome::Fail,
            diagnostics: vec![
                "metrics.query.success: normalized interface difference: a2a={\"outcome\":\"error\",\"code\":\"not_found\"} mcp={\"outcome\":\"result\",\"value\":{}}".to_owned(),
            ],
        }],
        scenarios: Vec::new(),
        compliant: false,
    };
    assert!(is_explained_mismatch_matrix_row("full-noncompliant"));
    assert!(unexplained_interface_differences("full-noncompliant", &report).is_empty());
    assert_eq!(
        unexplained_interface_differences("basic-compliant", &report).len(),
        1
    );
}

#[test]
fn devkit_revision_constant_matches_manifest() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains(DEVKIT_REVISION));
}
