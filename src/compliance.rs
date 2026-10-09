//! Versioned behavioral contract shared by A2A-LAB TCK runners.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use a2a_lab_dev_kit::{
    ImageId, ImageSourceId, JsonObject, MetricId, RunId, SourceId, TaskId, TimeRange,
};
use schemars::{JsonSchema, Schema, schema_for};
use serde::{Deserialize, Serialize};

/// Current reusable A2A-LAB compliance profile.
pub const COMPLIANCE_PROFILE_VERSION: &str = "1.1.0";

/// Stable identifier of the JSON result schema.
pub const COMPLIANCE_RESULT_SCHEMA_ID: &str =
    "https://a2a-lab.dev/schemas/compliance-result-1.1.0.json";

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceSuite {
    #[default]
    Basic,
    Full,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceCheck {
    BasicOperations,
    LinkedScenarios,
    A2aAgentMessage,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceInterface {
    A2a,
    Mcp,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceOperation {
    ListLogSources,
    QueryLogs,
    ListMetrics,
    QueryMetric,
    ListTasks,
    StartTask,
    GetTaskStatus,
    ListImageSources,
    ListImages,
    SearchImages,
    GetImage,
    GetCurrentImage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceCaseKind {
    Success,
    Boundary,
    Invalid,
    NotFound,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComplianceCase {
    pub id: &'static str,
    pub operation: ComplianceOperation,
    pub kind: ComplianceCaseKind,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceScenarioCapability {
    Logs,
    Metrics,
    Tasks,
    Images,
    A2aAgentMessage,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceScenarioAssertion {
    MeaningfulLinkage,
    CompatibleIdentifier,
    SuccessfulAvailability,
    TaskTerminalProgress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComplianceScenario {
    pub id: &'static str,
    pub capability: ComplianceScenarioCapability,
    pub producer: ComplianceInterface,
    pub consumer: ComplianceInterface,
    pub steps: &'static [&'static str],
    pub assertions: &'static [ComplianceScenarioAssertion],
}

#[derive(Debug, Clone, Copy)]
pub struct ComplianceSuiteContract {
    pub name: ComplianceSuite,
    pub cases: &'static [ComplianceCase],
    pub scenarios: &'static [ComplianceScenario],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationContract {
    pub operation: ComplianceOperation,
    pub interfaces: &'static [ComplianceInterface],
    pub observable_fields: &'static [&'static str],
    pub unavailable_error_code: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ComplianceProfile {
    pub version: &'static str,
    pub operations: &'static [OperationContract],
    pub cases: &'static [ComplianceCase],
    pub suites: &'static [ComplianceSuiteContract],
    pub intentional_envelope_differences: &'static [&'static str],
}

#[must_use]
pub const fn compliance_profile() -> ComplianceProfile {
    ComplianceProfile {
        version: COMPLIANCE_PROFILE_VERSION,
        operations: OPERATIONS,
        cases: CASES,
        suites: SUITES,
        intentional_envelope_differences: INTENTIONAL_ENVELOPE_DIFFERENCES,
    }
}

#[must_use]
pub const fn compliance_suite(suite: ComplianceSuite) -> ComplianceSuiteContract {
    match suite {
        ComplianceSuite::Basic => BASIC_SUITE,
        ComplianceSuite::Full => FULL_SUITE,
    }
}

const BOTH: &[ComplianceInterface] = &[ComplianceInterface::A2a, ComplianceInterface::Mcp];
const PAGE_FIELDS: &[&str] = &["items", "next_cursor"];
const RUN_FIELDS: &[&str] = &[
    "id",
    "task_id",
    "state",
    "input",
    "message",
    "result",
    "progress",
    "error_kind",
    "error_identifier",
];
const IMAGE_FIELDS: &[&str] = &["descriptor", "data"];

const fn operation(
    operation: ComplianceOperation,
    observable_fields: &'static [&'static str],
) -> OperationContract {
    OperationContract {
        operation,
        interfaces: BOTH,
        observable_fields,
        unavailable_error_code: "unavailable",
    }
}

const OPERATIONS: &[OperationContract] = &[
    operation(ComplianceOperation::ListLogSources, PAGE_FIELDS),
    operation(ComplianceOperation::QueryLogs, PAGE_FIELDS),
    operation(ComplianceOperation::ListMetrics, PAGE_FIELDS),
    operation(ComplianceOperation::QueryMetric, PAGE_FIELDS),
    operation(ComplianceOperation::ListTasks, PAGE_FIELDS),
    operation(ComplianceOperation::StartTask, RUN_FIELDS),
    operation(ComplianceOperation::GetTaskStatus, RUN_FIELDS),
    operation(ComplianceOperation::ListImageSources, PAGE_FIELDS),
    operation(ComplianceOperation::ListImages, PAGE_FIELDS),
    operation(ComplianceOperation::SearchImages, PAGE_FIELDS),
    operation(ComplianceOperation::GetImage, IMAGE_FIELDS),
    operation(ComplianceOperation::GetCurrentImage, IMAGE_FIELDS),
];

const INTENTIONAL_ENVELOPE_DIFFERENCES: &[&str] = &[
    "A2A returns a task artifact; MCP returns structured tool content",
    "A2A and MCP use their native error envelopes and status transport",
    "MCP may repeat image base64 in text and structured content",
    "protocol task and context identifiers are not A2A-LAB result fields",
];

macro_rules! cases {
    ($(($id:literal, $operation:ident, $kind:ident)),+ $(,)?) => {
        &[$(ComplianceCase {
            id: $id,
            operation: ComplianceOperation::$operation,
            kind: ComplianceCaseKind::$kind,
        }),+]
    };
}

const CASES: &[ComplianceCase] = cases![
    ("logs.list.success", ListLogSources, Success),
    ("logs.list.pagination", ListLogSources, Boundary),
    ("logs.list.invalid-limit", ListLogSources, Invalid),
    ("logs.list.unavailable", ListLogSources, Unavailable),
    ("logs.query.success", QueryLogs, Success),
    ("logs.query.half-open-range", QueryLogs, Boundary),
    ("logs.query.missing-source", QueryLogs, NotFound),
    ("logs.query.invalid-range", QueryLogs, Invalid),
    ("logs.query.unavailable", QueryLogs, Unavailable),
    ("metrics.list.success", ListMetrics, Success),
    ("metrics.list.pagination", ListMetrics, Boundary),
    ("metrics.list.invalid-limit", ListMetrics, Invalid),
    ("metrics.list.unavailable", ListMetrics, Unavailable),
    ("metrics.query.success", QueryMetric, Success),
    ("metrics.query.half-open-range", QueryMetric, Boundary),
    ("metrics.query.missing-metric", QueryMetric, NotFound),
    ("metrics.query.invalid-range", QueryMetric, Invalid),
    ("metrics.query.unavailable", QueryMetric, Unavailable),
    ("tasks.list.success", ListTasks, Success),
    ("tasks.list.pagination", ListTasks, Boundary),
    ("tasks.list.invalid-limit", ListTasks, Invalid),
    ("tasks.list.unavailable", ListTasks, Unavailable),
    ("tasks.start.success", StartTask, Success),
    ("tasks.start.missing-task", StartTask, NotFound),
    ("tasks.start.unavailable", StartTask, Unavailable),
    ("tasks.status.success", GetTaskStatus, Success),
    ("tasks.status.missing-run", GetTaskStatus, NotFound),
    ("tasks.status.unavailable", GetTaskStatus, Unavailable),
    ("images.sources.success", ListImageSources, Success),
    ("images.sources.pagination", ListImageSources, Boundary),
    ("images.sources.invalid-limit", ListImageSources, Invalid),
    ("images.sources.unavailable", ListImageSources, Unavailable),
    ("images.list.success", ListImages, Success),
    ("images.list.pagination", ListImages, Boundary),
    ("images.list.missing-source", ListImages, NotFound),
    ("images.list.invalid-limit", ListImages, Invalid),
    ("images.list.unavailable", ListImages, Unavailable),
    ("images.search.success", SearchImages, Success),
    ("images.search.half-open-range", SearchImages, Boundary),
    ("images.search.invalid-criteria", SearchImages, Invalid),
    ("images.search.unavailable", SearchImages, Unavailable),
    ("images.get.success", GetImage, Success),
    ("images.get.missing-image", GetImage, NotFound),
    ("images.get.unavailable", GetImage, Unavailable),
    ("images.current.success", GetCurrentImage, Success),
    ("images.current.missing-source", GetCurrentImage, NotFound),
    ("images.current.unavailable", GetCurrentImage, Unavailable),
];

const LINK_ASSERTIONS: &[ComplianceScenarioAssertion] = &[
    ComplianceScenarioAssertion::MeaningfulLinkage,
    ComplianceScenarioAssertion::CompatibleIdentifier,
    ComplianceScenarioAssertion::SuccessfulAvailability,
];
const TASK_ASSERTIONS: &[ComplianceScenarioAssertion] = &[
    ComplianceScenarioAssertion::MeaningfulLinkage,
    ComplianceScenarioAssertion::CompatibleIdentifier,
    ComplianceScenarioAssertion::SuccessfulAvailability,
    ComplianceScenarioAssertion::TaskTerminalProgress,
];
const AGENT_ASSERTIONS: &[ComplianceScenarioAssertion] =
    &[ComplianceScenarioAssertion::SuccessfulAvailability];
const LOG_STEPS: &[&str] = &["list_log_sources", "query_logs"];
const METRIC_STEPS: &[&str] = &["list_metrics", "query_metric"];
const TASK_STEPS: &[&str] = &["list_tasks", "start_task", "get_task_status"];
const IMAGE_STEPS: &[&str] = &[
    "list_image_sources",
    "list_or_search_images",
    "get_image",
    "get_current_image",
];
const AGENT_STEPS: &[&str] = &["a2a_agent_message"];

macro_rules! scenario {
    ($id:literal, $cap:ident, $producer:ident, $consumer:ident, $steps:ident, $assertions:ident) => {
        ComplianceScenario {
            id: $id,
            capability: ComplianceScenarioCapability::$cap,
            producer: ComplianceInterface::$producer,
            consumer: ComplianceInterface::$consumer,
            steps: $steps,
            assertions: $assertions,
        }
    };
}

const FULL_SCENARIOS: &[ComplianceScenario] = &[
    scenario!(
        "logs.link.a2a-a2a",
        Logs,
        A2a,
        A2a,
        LOG_STEPS,
        LINK_ASSERTIONS
    ),
    scenario!(
        "logs.link.mcp-mcp",
        Logs,
        Mcp,
        Mcp,
        LOG_STEPS,
        LINK_ASSERTIONS
    ),
    scenario!(
        "logs.link.a2a-mcp",
        Logs,
        A2a,
        Mcp,
        LOG_STEPS,
        LINK_ASSERTIONS
    ),
    scenario!(
        "logs.link.mcp-a2a",
        Logs,
        Mcp,
        A2a,
        LOG_STEPS,
        LINK_ASSERTIONS
    ),
    scenario!(
        "metrics.link.a2a-a2a",
        Metrics,
        A2a,
        A2a,
        METRIC_STEPS,
        LINK_ASSERTIONS
    ),
    scenario!(
        "metrics.link.mcp-mcp",
        Metrics,
        Mcp,
        Mcp,
        METRIC_STEPS,
        LINK_ASSERTIONS
    ),
    scenario!(
        "metrics.link.a2a-mcp",
        Metrics,
        A2a,
        Mcp,
        METRIC_STEPS,
        LINK_ASSERTIONS
    ),
    scenario!(
        "metrics.link.mcp-a2a",
        Metrics,
        Mcp,
        A2a,
        METRIC_STEPS,
        LINK_ASSERTIONS
    ),
    scenario!(
        "tasks.link.a2a-a2a",
        Tasks,
        A2a,
        A2a,
        TASK_STEPS,
        TASK_ASSERTIONS
    ),
    scenario!(
        "tasks.link.mcp-mcp",
        Tasks,
        Mcp,
        Mcp,
        TASK_STEPS,
        TASK_ASSERTIONS
    ),
    scenario!(
        "tasks.link.a2a-mcp",
        Tasks,
        A2a,
        Mcp,
        TASK_STEPS,
        TASK_ASSERTIONS
    ),
    scenario!(
        "tasks.link.mcp-a2a",
        Tasks,
        Mcp,
        A2a,
        TASK_STEPS,
        TASK_ASSERTIONS
    ),
    scenario!(
        "images.link.a2a-a2a",
        Images,
        A2a,
        A2a,
        IMAGE_STEPS,
        LINK_ASSERTIONS
    ),
    scenario!(
        "images.link.mcp-mcp",
        Images,
        Mcp,
        Mcp,
        IMAGE_STEPS,
        LINK_ASSERTIONS
    ),
    scenario!(
        "images.link.a2a-mcp",
        Images,
        A2a,
        Mcp,
        IMAGE_STEPS,
        LINK_ASSERTIONS
    ),
    scenario!(
        "images.link.mcp-a2a",
        Images,
        Mcp,
        A2a,
        IMAGE_STEPS,
        LINK_ASSERTIONS
    ),
    scenario!(
        "agent-message.a2a",
        A2aAgentMessage,
        A2a,
        A2a,
        AGENT_STEPS,
        AGENT_ASSERTIONS
    ),
];

const BASIC_SUITE: ComplianceSuiteContract = ComplianceSuiteContract {
    name: ComplianceSuite::Basic,
    cases: CASES,
    scenarios: &[],
};
const FULL_SUITE: ComplianceSuiteContract = ComplianceSuiteContract {
    name: ComplianceSuite::Full,
    cases: CASES,
    scenarios: FULL_SCENARIOS,
};
const SUITES: &[ComplianceSuiteContract] = &[BASIC_SUITE, FULL_SUITE];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "availability", content = "fixtures", rename_all = "snake_case")]
pub enum FixtureCapability<T> {
    Required(T),
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LogFixtures {
    pub source_id: SourceId,
    pub alternate_source_id: SourceId,
    pub missing_source_id: SourceId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MetricFixtures {
    pub metric_id: MetricId,
    pub alternate_metric_id: MetricId,
    pub missing_metric_id: MetricId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskFixtures {
    pub task_id: TaskId,
    pub alternate_task_id: TaskId,
    pub run_id: RunId,
    pub missing_task_id: TaskId,
    pub missing_run_id: RunId,
    pub input: JsonObject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageFixtures {
    pub source_id: ImageSourceId,
    pub alternate_source_id: ImageSourceId,
    pub image_id: ImageId,
    pub alternate_image_id: ImageId,
    pub missing_source_id: ImageSourceId,
    pub missing_image_id: ImageId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComplianceFixtures {
    pub suite_version: String,
    pub range: TimeRange,
    pub logs: FixtureCapability<LogFixtures>,
    pub metrics: FixtureCapability<MetricFixtures>,
    pub tasks: FixtureCapability<TaskFixtures>,
    pub images: FixtureCapability<ImageFixtures>,
}

impl ComplianceFixtures {
    pub fn validate(&self) -> Result<(), ComplianceValidationErrors> {
        let mut errors = Vec::new();
        check_version(&self.suite_version, &mut errors);
        if let Err(error) = self.range.check() {
            errors.push(field_error("range", error));
        }
        if let FixtureCapability::Required(value) = &self.logs {
            unique(
                "logs",
                [
                    &value.source_id,
                    &value.alternate_source_id,
                    &value.missing_source_id,
                ],
                &mut errors,
            );
        }
        if let FixtureCapability::Required(value) = &self.metrics {
            unique(
                "metrics",
                [
                    &value.metric_id,
                    &value.alternate_metric_id,
                    &value.missing_metric_id,
                ],
                &mut errors,
            );
        }
        if let FixtureCapability::Required(value) = &self.tasks {
            unique(
                "tasks.definitions",
                [
                    &value.task_id,
                    &value.alternate_task_id,
                    &value.missing_task_id,
                ],
                &mut errors,
            );
            unique(
                "tasks.runs",
                [&value.run_id, &value.missing_run_id],
                &mut errors,
            );
        }
        if let FixtureCapability::Required(value) = &self.images {
            unique(
                "images.sources",
                [
                    &value.source_id,
                    &value.alternate_source_id,
                    &value.missing_source_id,
                ],
                &mut errors,
            );
            unique(
                "images.images",
                [
                    &value.image_id,
                    &value.alternate_image_id,
                    &value.missing_image_id,
                ],
                &mut errors,
            );
        }
        finish(errors)
    }

    pub fn from_json(json: &str) -> Result<Self, ComplianceValidationErrors> {
        let fixtures: Self = parse_json(json)?;
        fixtures.validate()?;
        Ok(fixtures)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImplementationIdentity {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceOutcome {
    Pass,
    Fail,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComplianceCaseResult {
    pub case_id: String,
    pub required: bool,
    pub interfaces: BTreeSet<ComplianceInterface>,
    pub outcome: ComplianceOutcome,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComplianceScenarioResult {
    pub scenario_id: String,
    pub required: bool,
    pub interfaces: BTreeSet<ComplianceInterface>,
    pub handoff_path: Vec<ComplianceInterface>,
    pub outcome: ComplianceOutcome,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComplianceResult {
    pub suite_version: String,
    pub selected_suite: ComplianceSuite,
    pub enabled_checks: BTreeSet<ComplianceCheck>,
    pub implementation: ImplementationIdentity,
    pub tested_interfaces: BTreeSet<ComplianceInterface>,
    pub cases: Vec<ComplianceCaseResult>,
    pub scenarios: Vec<ComplianceScenarioResult>,
    pub compliant: bool,
}

impl ComplianceResult {
    pub fn validate(&self) -> Result<(), ComplianceValidationErrors> {
        let mut errors = Vec::new();
        check_version(&self.suite_version, &mut errors);
        check_nonblank(
            "implementation.name",
            &self.implementation.name,
            &mut errors,
        );
        check_nonblank(
            "implementation.version",
            &self.implementation.version,
            &mut errors,
        );
        self.validate_enabled_checks(&mut errors);
        let both = both_interfaces();
        if self.tested_interfaces != both {
            errors.push(error("tested_interfaces", "must contain both a2a and mcp"));
        }
        let known: BTreeSet<_> = CASES.iter().map(|case| case.id).collect();
        let mut seen = BTreeSet::new();
        for (index, case) in self.cases.iter().enumerate() {
            let field = format!("cases[{index}].case_id");
            if !known.contains(case.case_id.as_str()) {
                errors.push(error(field, "is not a profile case"));
            } else if !seen.insert(case.case_id.as_str()) {
                errors.push(error(field, "is duplicated"));
            }
            if case.interfaces != both {
                errors.push(error(
                    format!("cases[{index}].interfaces"),
                    "must contain both a2a and mcp",
                ));
            }
            validate_outcome(
                index,
                "case",
                case.required,
                case.outcome,
                &case.diagnostics,
                self.compliant,
                &mut errors,
            );
        }
        for missing in known.difference(&seen) {
            errors.push(error("cases", format!("missing profile case `{missing}`")));
        }
        if self.compliant && self.cases.iter().all(|case| !case.required) {
            errors.push(error("compliant", "requires at least one required case"));
        }
        self.validate_scenarios(&mut errors);
        finish(errors)
    }

    fn validate_enabled_checks(&self, errors: &mut Vec<ComplianceValidationError>) {
        if !self
            .enabled_checks
            .contains(&ComplianceCheck::BasicOperations)
        {
            errors.push(error("enabled_checks", "must enable basic_operations"));
        }
        let linked = self
            .enabled_checks
            .contains(&ComplianceCheck::LinkedScenarios);
        let agent = self
            .enabled_checks
            .contains(&ComplianceCheck::A2aAgentMessage);
        match self.selected_suite {
            ComplianceSuite::Basic if linked || agent => errors.push(error(
                "enabled_checks",
                "basic suite cannot enable full-suite scenario checks",
            )),
            ComplianceSuite::Full if !linked => errors.push(error(
                "enabled_checks",
                "full suite must enable linked_scenarios",
            )),
            ComplianceSuite::Basic | ComplianceSuite::Full => {}
        }
    }

    fn validate_scenarios(&self, errors: &mut Vec<ComplianceValidationError>) {
        if self.selected_suite == ComplianceSuite::Basic {
            if !self.scenarios.is_empty() {
                errors.push(error(
                    "scenarios",
                    "basic suite cannot contain scenario outcomes",
                ));
            }
            return;
        }
        let definitions: BTreeMap<_, _> = FULL_SCENARIOS
            .iter()
            .map(|scenario| (scenario.id, scenario))
            .collect();
        let agent_enabled = self
            .enabled_checks
            .contains(&ComplianceCheck::A2aAgentMessage);
        let mut seen = BTreeSet::new();
        for (index, scenario) in self.scenarios.iter().enumerate() {
            let field = format!("scenarios[{index}].scenario_id");
            let Some(definition) = definitions.get(scenario.scenario_id.as_str()) else {
                errors.push(error(field, "is not a full-suite scenario"));
                continue;
            };
            if !seen.insert(scenario.scenario_id.as_str()) {
                errors.push(error(field, "is duplicated"));
            }
            let is_agent = definition.capability == ComplianceScenarioCapability::A2aAgentMessage;
            if is_agent && !agent_enabled {
                errors.push(error(
                    format!("scenarios[{index}]"),
                    "agent-message outcome requires a2a_agent_message to be enabled",
                ));
            }
            let path = [definition.producer, definition.consumer];
            if scenario.handoff_path != path {
                errors.push(error(
                    format!("scenarios[{index}].handoff_path"),
                    "must match the defined producer-to-consumer path",
                ));
            }
            if scenario.interfaces != path.into_iter().collect() {
                errors.push(error(
                    format!("scenarios[{index}].interfaces"),
                    "must match the defined handoff interfaces",
                ));
            }
            validate_outcome(
                index,
                "scenario",
                scenario.required,
                scenario.outcome,
                &scenario.diagnostics,
                self.compliant,
                errors,
            );
        }
        for definition in FULL_SCENARIOS {
            let required = definition.capability != ComplianceScenarioCapability::A2aAgentMessage
                || agent_enabled;
            if required && !seen.contains(definition.id) {
                errors.push(error(
                    "scenarios",
                    format!("missing enabled scenario `{}`", definition.id),
                ));
            }
        }
    }

    pub fn from_json(json: &str) -> Result<Self, ComplianceValidationErrors> {
        let result: Self = parse_json(json)?;
        result.validate()?;
        Ok(result)
    }
}

#[must_use]
pub fn compliance_result_schema() -> Schema {
    let mut schema = schema_for!(ComplianceResult);
    schema.insert(
        "$id".to_owned(),
        serde_json::Value::String(COMPLIANCE_RESULT_SCHEMA_ID.to_owned()),
    );
    schema
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ComplianceValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplianceValidationErrors {
    errors: Vec<ComplianceValidationError>,
}

impl ComplianceValidationErrors {
    #[must_use]
    pub fn errors(&self) -> &[ComplianceValidationError] {
        &self.errors
    }
}

impl fmt::Display for ComplianceValidationErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, item) in self.errors.iter().enumerate() {
            if index > 0 {
                formatter.write_str("; ")?;
            }
            write!(formatter, "{}: {}", item.field, item.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for ComplianceValidationErrors {}

fn validate_outcome(
    index: usize,
    item: &str,
    required: bool,
    outcome: ComplianceOutcome,
    diagnostics: &[String],
    compliant: bool,
    errors: &mut Vec<ComplianceValidationError>,
) {
    let collection = if item == "case" { "cases" } else { "scenarios" };
    if outcome != ComplianceOutcome::Pass && diagnostics.is_empty() {
        errors.push(error(
            format!("{collection}[{index}].diagnostics"),
            format!("must explain a failed or skipped {item}"),
        ));
    }
    if compliant && required && outcome != ComplianceOutcome::Pass {
        errors.push(error(
            format!("{collection}[{index}].outcome"),
            format!("required {item}s must pass when compliant is true"),
        ));
    }
}

fn both_interfaces() -> BTreeSet<ComplianceInterface> {
    BOTH.iter().copied().collect()
}

fn check_version(version: &str, errors: &mut Vec<ComplianceValidationError>) {
    if version != COMPLIANCE_PROFILE_VERSION {
        errors.push(error(
            "suite_version",
            format!("must be {COMPLIANCE_PROFILE_VERSION}"),
        ));
    }
}

fn check_nonblank(field: &str, value: &str, errors: &mut Vec<ComplianceValidationError>) {
    if value.trim().is_empty() {
        errors.push(error(field, "must be nonblank"));
    }
}

fn unique<T: AsRef<str>>(
    field: &str,
    values: impl IntoIterator<Item = T>,
    errors: &mut Vec<ComplianceValidationError>,
) {
    let mut seen = BTreeSet::new();
    if values
        .into_iter()
        .any(|value| !seen.insert(value.as_ref().to_owned()))
    {
        errors.push(error(field, "fixture identifiers must be distinct"));
    }
}

fn field_error(field: &str, source: impl fmt::Display) -> ComplianceValidationError {
    error(field, source.to_string())
}

fn error(field: impl Into<String>, message: impl Into<String>) -> ComplianceValidationError {
    ComplianceValidationError {
        field: field.into(),
        message: message.into(),
    }
}

fn finish(errors: Vec<ComplianceValidationError>) -> Result<(), ComplianceValidationErrors> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ComplianceValidationErrors { errors })
    }
}

fn parse_json<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, ComplianceValidationErrors> {
    serde_json::from_str(json).map_err(|source| ComplianceValidationErrors {
        errors: vec![error(json_error_field(&source), source.to_string())],
    })
}

fn json_error_field(source: &serde_json::Error) -> String {
    let message = source.to_string();
    let Some(start) = message.find('`') else {
        return "$".to_owned();
    };
    let remaining = &message[start + 1..];
    remaining
        .find('`')
        .map_or_else(|| "$".to_owned(), |end| remaining[..end].to_owned())
}

#[must_use]
pub fn cases_by_operation() -> BTreeMap<ComplianceOperation, Vec<&'static ComplianceCase>> {
    let mut grouped = BTreeMap::new();
    for case in CASES {
        grouped
            .entry(case.operation)
            .or_insert_with(Vec::new)
            .push(case);
    }
    grouped
}
