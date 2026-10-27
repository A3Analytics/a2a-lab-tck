const VALIDATION: &str = include_str!("../.mise/scripts/compatibility.sh");
const MANIFEST: &str = include_str!("../Cargo.toml");
const PROVENANCE: &str = include_str!("../PROVENANCE");

use std::collections::BTreeSet;

use a2a_lab_tck::compliance_profile;

const DEVKIT_REVISION: &str = "9d5327868d96b3e800fd89f6debf434bcc12709d";
const SEAM_REVISION: &str = "928eae822318f650f8fe2f187e99091d4b46647b";
const PROFILE_1_1_0_CASE_IDS: &[&str] = &[
    "logs.list.success",
    "logs.list.pagination",
    "logs.list.invalid-limit",
    "logs.list.unavailable",
    "logs.query.success",
    "logs.query.half-open-range",
    "logs.query.missing-source",
    "logs.query.invalid-range",
    "logs.query.unavailable",
    "metrics.list.success",
    "metrics.list.pagination",
    "metrics.list.invalid-limit",
    "metrics.list.unavailable",
    "metrics.query.success",
    "metrics.query.half-open-range",
    "metrics.query.missing-metric",
    "metrics.query.invalid-range",
    "metrics.query.unavailable",
    "tasks.list.success",
    "tasks.list.pagination",
    "tasks.list.invalid-limit",
    "tasks.list.unavailable",
    "tasks.start.success",
    "tasks.start.missing-task",
    "tasks.start.unavailable",
    "tasks.status.success",
    "tasks.status.missing-run",
    "tasks.status.unavailable",
    "images.sources.success",
    "images.sources.pagination",
    "images.sources.invalid-limit",
    "images.sources.unavailable",
    "images.list.success",
    "images.list.pagination",
    "images.list.missing-source",
    "images.list.invalid-limit",
    "images.list.unavailable",
    "images.search.success",
    "images.search.half-open-range",
    "images.search.invalid-criteria",
    "images.search.unavailable",
    "images.get.success",
    "images.get.missing-image",
    "images.get.unavailable",
    "images.current.success",
    "images.current.missing-source",
    "images.current.unavailable",
];

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

#[test]
fn profile_1_2_0_does_not_rewrite_or_drop_1_1_0_case_identifiers() {
    let current: BTreeSet<_> = compliance_profile()
        .cases
        .iter()
        .map(|case| case.id)
        .collect();
    for case_id in PROFILE_1_1_0_CASE_IDS {
        assert!(
            current.contains(case_id),
            "1.2.0 dropped 1.1.0 case `{case_id}`"
        );
    }
}

#[test]
fn profile_1_1_0_schema_files_remain() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/compliance-fixtures-1.1.0.json")).unwrap();
    let result: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/compliance-result-1.1.0.json")).unwrap();
    assert_eq!(
        fixtures["$id"],
        "https://a2a-lab.dev/schemas/compliance-fixtures-1.1.0.json"
    );
    assert_eq!(
        result["$id"],
        "https://a2a-lab.dev/schemas/compliance-result-1.1.0.json"
    );
    for field in ["suite_version", "logs", "metrics", "tasks", "images"] {
        assert!(
            fixtures["properties"]
                .as_object()
                .is_some_and(|properties| properties.contains_key(field)),
            "1.1.0 fixtures schema missing {field}"
        );
    }
    for field in ["suite_version", "cases", "scenarios", "compliant"] {
        assert!(
            result["properties"]
                .as_object()
                .is_some_and(|properties| properties.contains_key(field)),
            "1.1.0 result schema missing {field}"
        );
    }
    assert!(VALIDATION.contains("--profile 1.1.0"));
}
