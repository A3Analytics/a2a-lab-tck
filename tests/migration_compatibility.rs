const VALIDATION: &str = include_str!("../.mise/scripts/compatibility.sh");
const MATRIX: &str = include_str!("../src/compatibility/matrix.rs");
const SUMMARY: &str = include_str!("../src/bin/a2a-lab-compatibility-matrix.rs");
const MANIFEST: &str = include_str!("../Cargo.toml");
const PROVENANCE: &str = include_str!("../PROVENANCE");

use std::collections::BTreeSet;

use a2a_lab_tck::compliance_profile;

const HISTORICAL_EXTRACTION: &str = "9d5327868d96b3e800fd89f6debf434bcc12709d";
const DEVKIT_REVISION: &str = "64ea97d3edcac8f51ba802dbed7fc4a595aec763";
const PROFILE_VERSION: &str = "1.3.0";
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
const PROFILE_1_2_0_CASE_IDS: &[&str] = &[
    "videos.sources.success",
    "videos.sources.pagination",
    "videos.sources.invalid-limit",
    "videos.sources.unavailable",
    "videos.frame.success",
    "videos.frame.nearest",
    "videos.frame.empty-coverage",
    "videos.frame.outside-coverage",
    "videos.frame.gap",
    "videos.frame.unavailable",
    "videos.clip.success",
    "videos.clip.half-open-range",
    "videos.clip.reversed-range",
    "videos.clip.over-limit",
    "videos.clip.empty-coverage",
    "videos.clip.outside-coverage",
    "videos.clip.gap",
    "videos.clip.unavailable-endpoint",
    "videos.clip.unavailable",
];

#[test]
fn migration_matrix_is_bounded_and_records_revisions() {
    for case in [
        "basic-compliant",
        "full-default-llm",
        "full-llm-opt-out",
        "full-noncompliant",
        "a2a-mcp-mismatch",
        "unavailable-capabilities",
        "timeout-or-endpoint-failure",
        "a2a-lab-compatibility-matrix",
        "a2a-lab-dev-kit-fixture",
    ] {
        assert!(
            VALIDATION.contains(case) || MATRIX.contains(case),
            "missing matrix case: {case}"
        );
    }
    assert!(VALIDATION.contains(DEVKIT_REVISION));
    assert!(VALIDATION.contains(HISTORICAL_EXTRACTION));
    assert!(VALIDATION.contains(PROFILE_VERSION));
    assert!(!VALIDATION.contains("--profile 1.1.0"));
    assert!(!VALIDATION.contains("a2a-lab-compliance --manifest-path"));
    assert!(!VALIDATION.contains("a2a-lab-ot2"));
}

#[test]
fn completed_runs_compare_the_public_report_contract() {
    for field in [
        "unexplained_interface_differences",
        "interface_differences",
        "contract_report",
        "normalized interface difference",
        "suite_version",
        "selected_suite",
        "enabled_checks",
        "compliant",
        "unexplained_differences",
    ] {
        assert!(
            VALIDATION.contains(field)
                || include_str!("../src/compatibility/report.rs").contains(field),
            "missing compared report field: {field}"
        );
    }
}

#[test]
fn provenance_and_manifest_record_revision_roles() {
    assert!(PROVENANCE.contains(HISTORICAL_EXTRACTION));
    assert!(PROVENANCE.contains(DEVKIT_REVISION));
    assert!(PROVENANCE.contains("Current dev-kit compatibility revision"));
    assert!(!PROVENANCE.contains("928eae822318f650f8fe2f187e99091d4b46647b"));
    assert!(MANIFEST.contains(DEVKIT_REVISION));
    assert!(!MANIFEST.contains("928eae822318f650f8fe2f187e99091d4b46647b"));
}

#[test]
fn validation_is_local_and_retains_machine_readable_evidence() {
    assert!(VALIDATION.contains("cargo build --offline --locked"));
    assert!(!VALIDATION.contains("cargo test --test compliance_runner"));
    assert!(VALIDATION.contains("cargo metadata"));
    assert!(
        SUMMARY.contains("network_model_services") || VALIDATION.contains("network_model_services")
    );
    assert!(VALIDATION.contains("target/compatibility"));
    assert!(VALIDATION.contains("summary.json"));
    assert!(
        VALIDATION.contains("standalone_worktree_dirty")
            || SUMMARY.contains("standalone_worktree_dirty")
    );
    assert!(VALIDATION.contains("devkit_manifest_independent_of_tck"));
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
            "1.3.0 dropped 1.1.0 case `{case_id}`"
        );
    }
}

#[test]
fn profile_1_3_0_does_not_rewrite_or_drop_1_2_0_case_identifiers() {
    let current: BTreeSet<_> = compliance_profile()
        .cases
        .iter()
        .map(|case| case.id)
        .collect();
    for case_id in PROFILE_1_2_0_CASE_IDS {
        if *case_id == "videos.clip.over-limit" {
            assert!(
                current.contains("videos.clip.empty-range"),
                "1.3.0 must replace videos.clip.over-limit with videos.clip.empty-range"
            );
            continue;
        }
        assert!(
            current.contains(case_id),
            "1.3.0 dropped 1.2.0 case `{case_id}`"
        );
    }
    assert!(current.contains("videos.clip.long-range"));
    for case_id in [
        "videos.timeline.success",
        "videos.timeline.empty-retention",
        "videos.timeline.missing-source",
        "videos.timeline.unavailable",
        "videos.follow.live-edge",
        "videos.follow.anchored",
        "videos.follow.missing-source",
        "videos.follow.live-unavailable",
        "videos.follow.unavailable",
    ] {
        assert!(
            current.contains(case_id),
            "1.3.0 missing new case `{case_id}`"
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
}

#[test]
fn profile_1_2_0_schema_files_remain() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/compliance-fixtures-1.2.0.json")).unwrap();
    let result: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/compliance-result-1.2.0.json")).unwrap();
    assert_eq!(
        fixtures["$id"],
        "https://a2a-lab.dev/schemas/compliance-fixtures-1.2.0.json"
    );
    assert_eq!(
        result["$id"],
        "https://a2a-lab.dev/schemas/compliance-result-1.2.0.json"
    );
}
