use std::collections::BTreeSet;

use a2a_lab_dev_kit::{JsonObject, TimeRange, UtcTimestamp};
use a2a_lab_tck::{
    COMPLIANCE_PROFILE_VERSION, COMPLIANCE_RESULT_SCHEMA_ID, ComplianceCaseResult, ComplianceCheck,
    ComplianceFixtures, ComplianceInterface, ComplianceOutcome, ComplianceResult,
    ComplianceScenarioCapability, ComplianceScenarioResult, ComplianceSuite, FixtureCapability,
    ImageFixtures, ImplementationIdentity, LogFixtures, MetricFixtures, TaskFixtures,
    compliance_profile, compliance_result_schema, compliance_suite,
};
use schemars::schema_for;
use serde_json::json;

const CASE_IDS: &[&str] = &[
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

const SCENARIO_IDS: &[&str] = &[
    "logs.link.a2a-a2a",
    "logs.link.mcp-mcp",
    "logs.link.a2a-mcp",
    "logs.link.mcp-a2a",
    "metrics.link.a2a-a2a",
    "metrics.link.mcp-mcp",
    "metrics.link.a2a-mcp",
    "metrics.link.mcp-a2a",
    "tasks.link.a2a-a2a",
    "tasks.link.mcp-mcp",
    "tasks.link.a2a-mcp",
    "tasks.link.mcp-a2a",
    "images.link.a2a-a2a",
    "images.link.mcp-mcp",
    "images.link.a2a-mcp",
    "images.link.mcp-a2a",
    "agent-message.a2a",
];

fn interfaces() -> BTreeSet<ComplianceInterface> {
    [ComplianceInterface::A2a, ComplianceInterface::Mcp]
        .into_iter()
        .collect()
}

fn fixtures() -> ComplianceFixtures {
    ComplianceFixtures {
        suite_version: COMPLIANCE_PROFILE_VERSION.to_owned(),
        range: TimeRange::new(
            UtcTimestamp::parse("2026-01-01T00:00:00Z").unwrap(),
            UtcTimestamp::parse("2026-01-02T00:00:00Z").unwrap(),
        )
        .unwrap(),
        logs: FixtureCapability::Required(LogFixtures {
            source_id: a2a_lab_dev_kit::SourceId::new("logs.primary").unwrap(),
            alternate_source_id: a2a_lab_dev_kit::SourceId::new("logs.secondary").unwrap(),
            missing_source_id: a2a_lab_dev_kit::SourceId::new("logs.missing").unwrap(),
        }),
        metrics: FixtureCapability::Required(MetricFixtures {
            metric_id: a2a_lab_dev_kit::MetricId::new("metrics.primary").unwrap(),
            alternate_metric_id: a2a_lab_dev_kit::MetricId::new("metrics.secondary").unwrap(),
            missing_metric_id: a2a_lab_dev_kit::MetricId::new("metrics.missing").unwrap(),
        }),
        tasks: FixtureCapability::Required(TaskFixtures {
            task_id: a2a_lab_dev_kit::TaskId::new("tasks.primary").unwrap(),
            alternate_task_id: a2a_lab_dev_kit::TaskId::new("tasks.secondary").unwrap(),
            run_id: a2a_lab_dev_kit::RunId::new("runs.primary").unwrap(),
            missing_task_id: a2a_lab_dev_kit::TaskId::new("tasks.missing").unwrap(),
            missing_run_id: a2a_lab_dev_kit::RunId::new("runs.missing").unwrap(),
            input: JsonObject::parse(r#"{"duration":1}"#).unwrap(),
        }),
        images: FixtureCapability::Required(ImageFixtures {
            source_id: a2a_lab_dev_kit::ImageSourceId::new("images.sources.primary").unwrap(),
            alternate_source_id: a2a_lab_dev_kit::ImageSourceId::new("images.sources.secondary")
                .unwrap(),
            image_id: a2a_lab_dev_kit::ImageId::new("images.primary").unwrap(),
            alternate_image_id: a2a_lab_dev_kit::ImageId::new("images.secondary").unwrap(),
            missing_source_id: a2a_lab_dev_kit::ImageSourceId::new("images.sources.missing")
                .unwrap(),
            missing_image_id: a2a_lab_dev_kit::ImageId::new("images.missing").unwrap(),
        }),
    }
}

fn result(suite: ComplianceSuite, agent_message: bool) -> ComplianceResult {
    let mut enabled_checks: BTreeSet<ComplianceCheck> =
        [ComplianceCheck::BasicOperations].into_iter().collect();
    if suite == ComplianceSuite::Full {
        enabled_checks.insert(ComplianceCheck::LinkedScenarios);
    }
    if agent_message {
        enabled_checks.insert(ComplianceCheck::A2aAgentMessage);
    }
    let scenarios = compliance_suite(suite)
        .scenarios
        .iter()
        .filter(|scenario| {
            scenario.capability != ComplianceScenarioCapability::A2aAgentMessage || agent_message
        })
        .map(|scenario| ComplianceScenarioResult {
            scenario_id: scenario.id.to_owned(),
            required: true,
            interfaces: [scenario.producer, scenario.consumer].into_iter().collect(),
            handoff_path: vec![scenario.producer, scenario.consumer],
            outcome: ComplianceOutcome::Pass,
            diagnostics: Vec::new(),
        })
        .collect();
    ComplianceResult {
        suite_version: COMPLIANCE_PROFILE_VERSION.to_owned(),
        selected_suite: suite,
        enabled_checks,
        implementation: ImplementationIdentity {
            name: "fixture-agent".to_owned(),
            version: "1.2.3".to_owned(),
        },
        tested_interfaces: interfaces(),
        cases: compliance_profile()
            .cases
            .iter()
            .map(|case| ComplianceCaseResult {
                case_id: case.id.to_owned(),
                required: true,
                interfaces: interfaces(),
                outcome: ComplianceOutcome::Pass,
                diagnostics: Vec::new(),
            })
            .collect(),
        scenarios,
        compliant: true,
    }
}

#[test]
fn published_profile_identifiers_and_composition_are_stable() {
    assert_eq!(COMPLIANCE_PROFILE_VERSION, "1.1.0");
    let basic = compliance_suite(ComplianceSuite::Basic);
    let full = compliance_suite(ComplianceSuite::Full);
    assert_eq!(
        basic.cases.iter().map(|case| case.id).collect::<Vec<_>>(),
        CASE_IDS
    );
    assert_eq!(basic.cases, full.cases);
    assert!(basic.scenarios.is_empty());
    assert_eq!(
        full.scenarios
            .iter()
            .map(|scenario| scenario.id)
            .collect::<Vec<_>>(),
        SCENARIO_IDS
    );
    assert_eq!(full.scenarios.len(), 16 + 1);
}

#[test]
fn fixture_and_report_json_round_trip_with_validation() {
    let fixtures = fixtures();
    fixtures.validate().unwrap();
    let encoded = serde_json::to_string(&fixtures).unwrap();
    assert_eq!(ComplianceFixtures::from_json(&encoded).unwrap(), fixtures);

    for report in [
        result(ComplianceSuite::Basic, false),
        result(ComplianceSuite::Full, false),
        result(ComplianceSuite::Full, true),
    ] {
        report.validate().unwrap();
        let encoded = serde_json::to_string(&report).unwrap();
        assert_eq!(ComplianceResult::from_json(&encoded).unwrap(), report);
    }
}

#[test]
fn required_skip_and_llm_evidence_rules_are_enforced() {
    let mut skipped = result(ComplianceSuite::Basic, false);
    skipped.cases[0].outcome = ComplianceOutcome::Skip;
    skipped.cases[0].diagnostics = vec!["fixture unavailable".to_owned()];
    assert!(
        skipped
            .validate()
            .unwrap_err()
            .errors()
            .iter()
            .any(|item| { item.field == "cases[0].outcome" })
    );

    let mut missing_llm = result(ComplianceSuite::Full, false);
    missing_llm
        .enabled_checks
        .insert(ComplianceCheck::A2aAgentMessage);
    assert!(
        missing_llm
            .validate()
            .unwrap_err()
            .errors()
            .iter()
            .any(|item| {
                item.field == "scenarios" && item.message.contains("agent-message.a2a")
            })
    );

    let with_llm = result(ComplianceSuite::Full, true);
    let evidence = with_llm.scenarios.last().unwrap();
    assert_eq!(evidence.scenario_id, "agent-message.a2a");
    assert_eq!(evidence.handoff_path, [ComplianceInterface::A2a; 2]);
    assert_eq!(
        evidence.interfaces,
        [ComplianceInterface::A2a].into_iter().collect()
    );
}

#[test]
fn malformed_reports_return_field_level_diagnostics() {
    let malformed = json!({
        "suite_version": "0.0.0",
        "selected_suite": "basic",
        "enabled_checks": ["basic_operations"],
        "implementation": {"name": "", "version": "1"},
        "tested_interfaces": ["a2a"],
        "cases": [],
        "scenarios": [],
        "compliant": true
    });
    let errors = ComplianceResult::from_json(&malformed.to_string()).unwrap_err();
    let fields: BTreeSet<_> = errors
        .errors()
        .iter()
        .map(|item| item.field.as_str())
        .collect();
    for expected in [
        "suite_version",
        "implementation.name",
        "tested_interfaces",
        "cases",
        "compliant",
    ] {
        assert!(fields.contains(expected));
    }
}

#[test]
fn checked_in_schemas_match_generated_public_types() {
    let fixture_checked_in: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/compliance-fixtures-1.1.0.json")).unwrap();
    let mut fixtures = schema_for!(ComplianceFixtures);
    fixtures.insert(
        "$id".to_owned(),
        "https://a2a-lab.dev/schemas/compliance-fixtures-1.1.0.json".into(),
    );
    assert_eq!(fixture_checked_in, serde_json::to_value(fixtures).unwrap());

    let result_checked_in: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/compliance-result-1.1.0.json")).unwrap();
    let generated = serde_json::to_value(compliance_result_schema()).unwrap();
    assert_eq!(generated["$id"], COMPLIANCE_RESULT_SCHEMA_ID);
    assert_eq!(result_checked_in, generated);
}
