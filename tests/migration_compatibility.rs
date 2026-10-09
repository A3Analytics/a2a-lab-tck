const VALIDATION: &str = include_str!("../.mise/scripts/compatibility.sh");
const MANIFEST: &str = include_str!("../Cargo.toml");
const PROVENANCE: &str = include_str!("../PROVENANCE");

const DEVKIT_REVISION: &str = "9d5327868d96b3e800fd89f6debf434bcc12709d";
const SEAM_REVISION: &str = "928eae822318f650f8fe2f187e99091d4b46647b";

#[test]
fn migration_matrix_is_bounded_and_records_revisions() {
    for case in [
        "basic-compliant standard basic false",
        "full-default-llm standard full true",
        "full-llm-opt-out standard full false",
        "full-noncompliant mcp-metric-error full true",
        "a2a-mcp-mismatch mcp-empty-log-sources full true",
        "unavailable_capabilities_skip_only_nonrequired_cases",
        "timeout_and_endpoint_failure_are_reported_without_aborting",
        "--suite invalid",
    ] {
        assert!(VALIDATION.contains(case), "missing matrix case: {case}");
    }
    assert!(VALIDATION.contains(DEVKIT_REVISION));
    assert!(VALIDATION.contains("standalone_revision"));
    assert!(PROVENANCE.contains(DEVKIT_REVISION));
}

#[test]
fn completed_runs_compare_the_public_report_contract() {
    for field in [
        "CLI exit class",
        "suite_version",
        "selected_suite",
        "enabled_checks",
        "compliant",
        "case_id",
        "scenario_id",
        "required",
        "outcome",
        "diagnostics",
        "normalized compliance report",
    ] {
        assert!(
            VALIDATION.contains(field),
            "missing compared report field: {field}"
        );
    }
    assert!(VALIDATION.contains("unexplained_differences"));
    assert!(VALIDATION.contains("raise SystemExit(json.dumps(unexplained"));
}

#[test]
fn action_and_architecture_checks_allow_only_named_differences() {
    for action_value in [
        "source_action",
        "standalone_action",
        "source_runner",
        "standalone_runner",
        "Action input/output field mismatch",
        "Action default mismatch",
        "Action runner contract mismatch",
        "branding and provenance differences allowed",
    ] {
        assert!(
            VALIDATION.contains(action_value),
            "missing Action comparison: {action_value}"
        );
    }
    assert!(MANIFEST.contains(SEAM_REVISION));
    assert!(VALIDATION.contains("a2a-lab-tck -> a2a-lab-dev-kit"));
    assert!(VALIDATION.contains("devkit typed-seam revision depends"));
}

#[test]
fn validation_is_local_and_retains_machine_readable_evidence() {
    assert!(VALIDATION.contains("cargo build --offline --locked"));
    assert!(VALIDATION.contains("\"network_model_services\": False"));
    assert!(VALIDATION.contains("\"docker\": False"));
    assert!(VALIDATION.contains("\"hardware\": False"));
    assert!(VALIDATION.contains("target/compatibility"));
    assert!(VALIDATION.contains("summary.json"));
}
