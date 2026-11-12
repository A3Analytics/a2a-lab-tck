use std::fs;

use a2a_lab_tck::{ComplianceFixtures, compliance_result_schema};
use schemars::schema_for;
use serde_json::Value;

#[cfg(feature = "semantic-contracts")]
use a2a_lab_dev_kit::{
    AppendFrameDecisionRequest, AppendFrameDecisionResult, FrameDecisionHistoryResult,
    FrameDecisionRecord, FrameDescriptionProvenance, ReadFrameDecisionHistoryRequest,
    SavedFrameRef, SearchSemanticFramesRequest, SemanticFrameSearchResult,
    SemanticIndexingCoverage, SemanticSearchCapabilities, TextEmbeddingProvenance,
};
#[cfg(feature = "semantic-contracts")]
use schemars::JsonSchema;

const FIXTURE_SCHEMA_ID: &str = "https://a2a-lab.dev/schemas/compliance-fixtures-1.3.0.json";
#[cfg(feature = "semantic-contracts")]
const SCHEMA_BASE: &str = "https://a2a-lab.dev/schemas";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("schemas")?;

    let mut fixtures = schema_for!(ComplianceFixtures);
    fixtures.insert("$id".to_owned(), FIXTURE_SCHEMA_ID.into());
    write_schema(
        "schemas/compliance-fixtures-1.3.0.json",
        &serde_json::to_value(fixtures)?,
    )?;
    write_schema(
        "schemas/compliance-result-1.3.0.json",
        &serde_json::to_value(compliance_result_schema())?,
    )?;

    #[cfg(feature = "semantic-contracts")]
    write_semantic_schemas()?;

    Ok(())
}

#[cfg(feature = "semantic-contracts")]
fn write_semantic_schemas() -> Result<(), Box<dyn std::error::Error>> {
    write_typed_schema::<SavedFrameRef>("saved-frame-ref-1.0.0.json")?;
    write_typed_schema::<FrameDescriptionProvenance>("frame-description-provenance-1.0.0.json")?;
    write_typed_schema::<TextEmbeddingProvenance>("text-embedding-provenance-1.0.0.json")?;
    write_typed_schema::<SearchSemanticFramesRequest>("search-semantic-frames-request-1.0.0.json")?;
    write_typed_schema::<SemanticFrameSearchResult>("semantic-frame-search-result-1.0.0.json")?;
    write_typed_schema::<SemanticIndexingCoverage>("semantic-indexing-coverage-1.0.0.json")?;
    write_typed_schema::<SemanticSearchCapabilities>("semantic-search-capabilities-1.0.0.json")?;
    write_typed_schema::<FrameDecisionRecord>("frame-decision-record-1.0.0.json")?;
    write_typed_schema::<AppendFrameDecisionRequest>("append-frame-decision-request-1.0.0.json")?;
    write_typed_schema::<AppendFrameDecisionResult>("append-frame-decision-result-1.0.0.json")?;
    write_typed_schema::<ReadFrameDecisionHistoryRequest>(
        "read-frame-decision-history-request-1.0.0.json",
    )?;
    write_typed_schema::<FrameDecisionHistoryResult>("frame-decision-history-result-1.0.0.json")?;
    Ok(())
}

#[cfg(feature = "semantic-contracts")]
fn write_typed_schema<T: JsonSchema>(file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let id = format!("{SCHEMA_BASE}/{file_name}");
    let mut schema = schema_for!(T);
    schema.insert("$id".to_owned(), id.into());
    write_schema(
        &format!("schemas/{file_name}"),
        &serde_json::to_value(schema)?,
    )
}

fn write_schema(path: &str, schema: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let mut encoded = serde_json::to_string_pretty(schema)?;
    encoded.push('\n');
    fs::write(path, encoded)?;
    Ok(())
}
