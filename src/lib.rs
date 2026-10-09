//! A3 Analytics' Technology Compatibility Kit for the A2A-LAB profile.
//!
//! This crate is distinct from the upstream A2A protocol TCK.

pub mod compliance;
mod compliance_runner;

pub use compliance::{
    COMPLIANCE_PROFILE_VERSION, COMPLIANCE_RESULT_SCHEMA_ID, ComplianceCase, ComplianceCaseKind,
    ComplianceCaseResult, ComplianceCheck, ComplianceFixtures, ComplianceInterface,
    ComplianceOperation, ComplianceOutcome, ComplianceProfile, ComplianceResult,
    ComplianceScenario, ComplianceScenarioAssertion, ComplianceScenarioCapability,
    ComplianceScenarioResult, ComplianceSuite, ComplianceSuiteContract, ComplianceValidationError,
    ComplianceValidationErrors, FixtureCapability, ImageFixtures, ImplementationIdentity,
    LogFixtures, MetricFixtures, OperationContract, TaskFixtures, cases_by_operation,
    compliance_profile, compliance_result_schema, compliance_suite,
};
pub use compliance_runner::{
    ComplianceEndpoint, ComplianceRunError, ComplianceRunner, ComplianceRunnerConfig,
    run_compliance,
};

#[cfg(test)]
mod tests {
    #[test]
    fn package_has_canonical_name() {
        assert_eq!(env!("CARGO_PKG_NAME"), "a2a-lab-tck");
    }
}
