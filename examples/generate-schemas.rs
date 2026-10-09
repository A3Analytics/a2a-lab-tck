use std::fs;

use a2a_lab_tck::{ComplianceFixtures, compliance_result_schema};
use schemars::schema_for;

const FIXTURE_SCHEMA_ID: &str = "https://a2a-lab.dev/schemas/compliance-fixtures-1.1.0.json";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("schemas")?;

    let mut fixtures = schema_for!(ComplianceFixtures);
    fixtures.insert("$id".to_owned(), FIXTURE_SCHEMA_ID.into());
    write_schema(
        "schemas/compliance-fixtures-1.1.0.json",
        &serde_json::to_value(fixtures)?,
    )?;
    write_schema(
        "schemas/compliance-result-1.1.0.json",
        &serde_json::to_value(compliance_result_schema())?,
    )
}

fn write_schema(path: &str, schema: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let mut encoded = serde_json::to_string_pretty(&schema)?;
    encoded.push('\n');
    fs::write(path, encoded)?;
    Ok(())
}
