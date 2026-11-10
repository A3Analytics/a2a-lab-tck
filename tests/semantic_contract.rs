#![cfg(feature = "semantic-contracts")]

//! Additive library compatibility checks for semantic indexing and frame decisions.
//! Profile 1.3.0 compliance cases remain unchanged when semantic providers are absent.

use std::collections::BTreeSet;

use a2a_lab_dev_kit::{
    A2A_LAB_CONFIG_SCHEMA_VERSION, A2A_LAB_CONFIG_SCHEMA_VERSION_V2, A2aLabConfig,
    AppendFrameDecisionRequest, ChatProvider, DEFAULT_BEDROCK_DESCRIPTION_MODEL,
    DEFAULT_JEV_MODEL_REVISION, DEFAULT_SEMANTIC_DESCRIPTION_BACKEND, DecisionEvaluationCoverage,
    DecisionEvidenceItem, DecisionIdempotencyKey, DecisionOutcome, DecisionSubject,
    DecisionUnknownReason, DvrStoreNamespace, DvrTrackId, EmbeddingMetric, ForeignObjectSubject,
    FrameDecisionRecord, FrameDecisionSchemaRefs, FrameDescriptionProvenance, FrameEmbeddingVector,
    FrameRoiTransform, FrameVectorIndexAttachment, JevEvaluationId, JevEvaluationProvenance,
    JevProducerIdentity, JevStructuredOutput, LabPredicate, ModelSpaceIdentity,
    ProvenanceGeneration, SavedFrameRef, SemanticFrameHit, SemanticFrameIndexState,
    SemanticIndexingCoverage, SemanticIndexingFreshness, SemanticIndexingStateCounts,
    SemanticRetrievalScore, SemanticSearchSchemaRefs, SemanticSearchScope, Sha256Digest,
    TextEmbeddingProvenance, TimeRange, UtcTimestamp, VectorNormalization, VideoSourceId,
};
use schemars::{JsonSchema, schema_for};
use serde::de::DeserializeOwned;

pub const SEMANTIC_SCHEMA_FILES: &[&str] = &[
    "saved-frame-ref-1.0.0.json",
    "frame-description-provenance-1.0.0.json",
    "text-embedding-provenance-1.0.0.json",
    "search-semantic-frames-request-1.0.0.json",
    "semantic-frame-search-result-1.0.0.json",
    "semantic-indexing-coverage-1.0.0.json",
    "frame-decision-record-1.0.0.json",
    "append-frame-decision-request-1.0.0.json",
    "append-frame-decision-result-1.0.0.json",
    "read-frame-decision-history-request-1.0.0.json",
    "frame-decision-history-result-1.0.0.json",
    "a2a-lab-config-v2-semantic-example.json",
];

const VALID_V2_CONFIG: &str = include_str!("fixtures/a2a-lab-v2-semantic-valid.json");

fn chupacabra_frame(payload: &[u8], capture: &str) -> SavedFrameRef {
    SavedFrameRef::from_payload_bytes(
        DvrStoreNamespace::new("chupacabra.dvr").unwrap(),
        VideoSourceId::new("deck-left").unwrap(),
        DvrTrackId::new("mjpeg-0").unwrap(),
        UtcTimestamp::parse(capture).unwrap(),
        payload,
        None,
    )
    .unwrap()
}

fn model_space(generation: u64) -> ModelSpaceIdentity {
    ModelSpaceIdentity::new(
        "operator-embedding",
        "operator-text-embed-v1",
        384,
        VectorNormalization::L2Unit,
        EmbeddingMetric::Cosine,
        ProvenanceGeneration::new(generation).unwrap(),
    )
    .unwrap()
}

fn description(text: &[u8], generation: u64) -> FrameDescriptionProvenance {
    FrameDescriptionProvenance::new(
        "bedrock",
        DEFAULT_BEDROCK_DESCRIPTION_MODEL,
        Sha256Digest::hash_bytes(b"luna-prompt-v1"),
        Sha256Digest::hash_bytes(b"preprocess-v1"),
        Some(FrameRoiTransform::new([0.0, 0.0, 1.0, 1.0]).unwrap()),
        Sha256Digest::hash_bytes(text),
        ProvenanceGeneration::new(generation).unwrap(),
    )
    .unwrap()
}

fn embedding(text: &[u8], generation: u64) -> TextEmbeddingProvenance {
    TextEmbeddingProvenance::new(
        "operator-embedding",
        "operator-text-embed-v1",
        384,
        VectorNormalization::L2Unit,
        EmbeddingMetric::Cosine,
        ProvenanceGeneration::new(generation).unwrap(),
        Sha256Digest::hash_bytes(text),
    )
    .unwrap()
}

fn attachment(text: &[u8], generation: u64) -> FrameVectorIndexAttachment {
    let vector = FrameEmbeddingVector::new(vec![0.1; 384], 384).unwrap();
    FrameVectorIndexAttachment::new(
        model_space(generation),
        description(text, generation),
        embedding(text, generation),
        vector,
    )
    .unwrap()
}

fn jev_provenance(
    captured: &str,
    evaluated: &str,
    coverage: DecisionEvaluationCoverage,
) -> JevEvaluationProvenance {
    JevEvaluationProvenance::new(
        UtcTimestamp::parse(captured).unwrap(),
        UtcTimestamp::parse(evaluated).unwrap(),
        JevProducerIdentity::default_production(),
        JevEvaluationId::new("jev-req-chupacabra-unevaluated").unwrap(),
        Sha256Digest::hash_bytes(b"config-v1"),
        Sha256Digest::hash_bytes(b"prompt-v1"),
        Sha256Digest::hash_bytes(b"criteria-v1"),
        Sha256Digest::hash_bytes(b"threshold-v1"),
        FrameRoiTransform::new([0.0, 0.0, 1.0, 1.0]).unwrap(),
        Sha256Digest::hash_bytes(b"jev-response-bytes"),
        coverage,
        Option::<&str>::None,
    )
    .unwrap()
}

#[test]
fn checked_in_semantic_schemas_match_generated_types() {
    for file in SEMANTIC_SCHEMA_FILES {
        if file.ends_with("-example.json") {
            continue;
        }
        let path = format!("schemas/{file}");
        let checked_in: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|_| {
                panic!("missing schema artifact {path}; run `mise run schemas`")
            }))
            .unwrap();
        assert_eq!(
            checked_in["$id"],
            format!("https://a2a-lab.dev/schemas/{file}")
        );
    }

    assert_schema_matches::<SavedFrameRef>("saved-frame-ref-1.0.0.json");
    assert_schema_matches::<FrameDescriptionProvenance>("frame-description-provenance-1.0.0.json");
    assert_schema_matches::<TextEmbeddingProvenance>("text-embedding-provenance-1.0.0.json");
    assert_schema_matches::<a2a_lab_dev_kit::SearchSemanticFramesRequest>(
        "search-semantic-frames-request-1.0.0.json",
    );
    assert_schema_matches::<a2a_lab_dev_kit::SemanticFrameSearchResult>(
        "semantic-frame-search-result-1.0.0.json",
    );
    assert_schema_matches::<SemanticIndexingCoverage>("semantic-indexing-coverage-1.0.0.json");
    assert_schema_matches::<FrameDecisionRecord>("frame-decision-record-1.0.0.json");
    assert_schema_matches::<AppendFrameDecisionRequest>("append-frame-decision-request-1.0.0.json");
    assert_schema_matches::<a2a_lab_dev_kit::AppendFrameDecisionResult>(
        "append-frame-decision-result-1.0.0.json",
    );
    assert_schema_matches::<a2a_lab_dev_kit::ReadFrameDecisionHistoryRequest>(
        "read-frame-decision-history-request-1.0.0.json",
    );
    assert_schema_matches::<a2a_lab_dev_kit::FrameDecisionHistoryResult>(
        "frame-decision-history-result-1.0.0.json",
    );
}

fn assert_schema_matches<T: JsonSchema>(file: &str) {
    let path = format!("schemas/{file}");
    let checked_in: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut generated = schema_for!(T);
    generated.insert(
        "$id".to_owned(),
        format!("https://a2a-lab.dev/schemas/{file}").into(),
    );
    assert_eq!(
        checked_in,
        serde_json::to_value(generated).unwrap(),
        "{file}"
    );
}

#[test]
fn provenance_round_trips_without_credentials() {
    let text = b"tip visible on channel one";
    let desc = description(text, 1);
    let embed = embedding(text, 1);
    let encoded_desc = serde_json::to_string(&desc).unwrap();
    let encoded_embed = serde_json::to_string(&embed).unwrap();
    assert_eq!(
        serde_json::from_str::<FrameDescriptionProvenance>(&encoded_desc).unwrap(),
        desc
    );
    assert_eq!(
        serde_json::from_str::<TextEmbeddingProvenance>(&encoded_embed).unwrap(),
        embed
    );
    assert!(encoded_desc.contains(DEFAULT_BEDROCK_DESCRIPTION_MODEL));
    assert!(!encoded_desc.contains("claude-chat-only"));
}

#[test]
fn luna_defaults_and_override_stay_independent_from_chat_and_jev() {
    let config = A2aLabConfig::parse(VALID_V2_CONFIG).unwrap();
    assert_eq!(config.schema_version, A2A_LAB_CONFIG_SCHEMA_VERSION_V2);
    assert_eq!(
        config.chat.as_ref().unwrap().provider,
        Some(ChatProvider::Anthropic)
    );
    let semantic = config.semantic_indexing.as_ref().unwrap();
    assert_eq!(
        semantic.description.backend(),
        DEFAULT_SEMANTIC_DESCRIPTION_BACKEND
    );
    assert_eq!(
        semantic.description.model(),
        DEFAULT_BEDROCK_DESCRIPTION_MODEL
    );
    assert_eq!(semantic.decisions.provider(), "jev");
    assert_eq!(semantic.decisions.model(), DEFAULT_JEV_MODEL_REVISION);

    let openai_override = r#"{"schema_version":2,"semantic_indexing":{
        "description":{"provider":"openai"},
        "embeddings":{"provider":"e","model":"m","space":{"dimensions":4,"normalization":"l2_unit","metric":"cosine","generation":1}},
        "limits":{
            "worker":{"max_batch_items":1,"max_concurrency":1,"max_queue_backlog":1,"max_retry_attempts":1,"job_deadline":"30s","retry_backoff":"5s"},
            "search":{"top_k":1,"candidate_limit":1,"max_neighbor_frames":1,"query_window":"1h"},
            "roi":{"min_edge_pixels":280,"max_edge_pixels":1568,"max_rois_per_frame":1},
            "storage":{"vector_index_path":"./semantic","max_disk_bytes":2048},
            "decisions":{"max_batch_items":1,"max_concurrency":1,"job_deadline":"45s","coverage_policy":"sampled"}
        }
    }}"#;
    let openai = A2aLabConfig::parse(openai_override).unwrap();
    assert_eq!(
        openai.semantic_indexing.unwrap().description.model(),
        a2a_lab_dev_kit::DEFAULT_OPENAI_DESCRIPTION_MODEL
    );
    assert_eq!(A2A_LAB_CONFIG_SCHEMA_VERSION, 1);
}

#[test]
fn v1_rejects_semantic_indexing_key() {
    let error = A2aLabConfig::parse(r#"{"schema_version":1,"semantic_indexing":{}}"#).unwrap_err();
    assert!(error.to_string().contains("semantic_indexing"));
}

#[test]
fn rejects_non_jev_and_fallback_settings_before_startup() {
    let limits = r#","limits":{"worker":{"max_batch_items":1,"max_concurrency":1,"max_queue_backlog":1,"max_retry_attempts":1,"job_deadline":"30s","retry_backoff":"5s"},"search":{"top_k":1,"candidate_limit":1,"max_neighbor_frames":1,"query_window":"1h"},"roi":{"min_edge_pixels":280,"max_edge_pixels":1568,"max_rois_per_frame":1},"storage":{"vector_index_path":"./x","max_disk_bytes":1024},"decisions":{"max_batch_items":1,"max_concurrency":1,"job_deadline":"45s","coverage_policy":"on_demand"}}"#;
    let openai_decisions = format!(
        r#"{{"schema_version":2,"semantic_indexing":{{"decisions":{{"provider":"openai","model":"{DEFAULT_JEV_MODEL_REVISION}"}},"embeddings":{{"provider":"p","model":"m","space":{{"dimensions":4,"normalization":"l2_unit","metric":"cosine","generation":1}}}}{limits}}}}}"#
    );
    assert!(A2aLabConfig::parse(&openai_decisions).is_err());

    let fallback = format!(
        r#"{{"schema_version":2,"semantic_indexing":{{"fallback":true,"embeddings":{{"provider":"p","model":"m","space":{{"dimensions":4,"normalization":"l2_unit","metric":"cosine","generation":1}}}}{limits}}}}}"#
    );
    let err = A2aLabConfig::parse(&fallback).unwrap_err();
    assert!(err.to_string().contains("semantic_indexing.fallback"));
}

#[test]
fn missing_descriptions_surface_explicit_index_state() {
    let counts = SemanticIndexingStateCounts::new(5, 0, 0, 0);
    assert_eq!(counts.pending(), 5);
    assert_eq!(counts.ready(), 0);

    let saved = chupacabra_frame(b"chupacabra-pilot", "2024-06-01T12:00:10Z");
    let attach = attachment(b"pending description job", 1);
    let retrieval = SemanticRetrievalScore::new(0.42, EmbeddingMetric::Cosine).unwrap();
    let start = saved.captured_at();
    let end = start.saturating_add_micros(5_000_000);
    let neighbors = a2a_lab_dev_kit::SemanticFramePlaybackNeighbors::new(
        TimeRange::new(start, end).unwrap(),
        vec![saved.clone()],
        5,
    )
    .unwrap();
    let hit = SemanticFrameHit::new(
        saved,
        "description pending",
        &attach,
        retrieval,
        SemanticFrameIndexState::Pending,
        a2a_lab_dev_kit::SemanticFrameMediaAvailability::Pending,
        neighbors,
    )
    .unwrap();
    assert_eq!(hit.index(), SemanticFrameIndexState::Pending);
}

#[test]
fn uncertain_outcome_differs_from_unevaluated_coverage() {
    let saved = chupacabra_frame(b"chupacabra-uncertain", "2024-06-01T12:00:11Z");
    let uncertain_subject = DecisionSubject::ForeignObject(
        ForeignObjectSubject::new(
            "deck-slot-3",
            FrameRoiTransform::new([0.1, 0.1, 0.9, 0.9]).unwrap(),
            Some(Sha256Digest::hash_bytes(b"expected-scene-v1")),
            Some(DecisionUnknownReason::UncertainExpectation),
        )
        .unwrap(),
    );
    let uncertain = FrameDecisionRecord::new(
        DecisionIdempotencyKey::new(
            "jev-req-chupacabra-uncertain:lab.foreign_object_present:deck-slot-3",
        )
        .unwrap(),
        saved.clone(),
        LabPredicate::foreign_object_present_v1(),
        uncertain_subject,
        vec![
            DecisionEvidenceItem::saved_frame_revision(
                saved.frame_id().clone(),
                FrameRoiTransform::new([0.0, 0.0, 1.0, 1.0]).unwrap(),
            )
            .unwrap(),
        ],
        DecisionOutcome::Unknown,
        JevStructuredOutput::empty(),
        jev_provenance(
            "2024-06-01T12:00:11Z",
            "2024-06-01T12:00:13Z",
            DecisionEvaluationCoverage::Evaluated,
        ),
        None,
    )
    .unwrap();
    assert_eq!(uncertain.outcome(), DecisionOutcome::Unknown);
    assert_eq!(
        uncertain.provenance().coverage(),
        DecisionEvaluationCoverage::Evaluated
    );

    let unevaluated = jev_provenance(
        "2024-06-01T12:00:11Z",
        "2024-06-01T12:00:11Z",
        DecisionEvaluationCoverage::NotEvaluated,
    );
    assert_ne!(uncertain.provenance().coverage(), unevaluated.coverage());
}

#[test]
fn attachment_provenance_binds_chupacabra_revision() {
    let saved = chupacabra_frame(b"chupacabra-pilot", "2024-06-01T12:00:00Z");
    let attach = attachment(b"left pipette present", 1);
    let encoded = serde_json::to_string(&attach).unwrap();
    let decoded: FrameVectorIndexAttachment = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        decoded.description().resolved_model(),
        DEFAULT_BEDROCK_DESCRIPTION_MODEL
    );
    assert_eq!(saved.store().as_str(), "chupacabra.dvr");
}

#[test]
fn semantic_hit_schema_excludes_decision_fields() {
    fn property_names<T: JsonSchema + DeserializeOwned>() -> Vec<String> {
        let value = serde_json::to_value(schema_for!(T)).unwrap();
        value
            .pointer("/properties")
            .and_then(|properties| properties.as_object())
            .map(|properties| properties.keys().cloned().collect())
            .unwrap_or_default()
    }
    let names = property_names::<SemanticFrameHit>();
    for forbidden in ["outcome", "predicate", "jev", "decision", "confidence"] {
        assert!(
            !names.iter().any(|name| name.contains(forbidden)),
            "SemanticFrameHit must not expose decision-like field {forbidden}"
        );
    }
}

#[test]
fn coverage_reports_incomplete_indexing_without_implying_absence() {
    let scope = SemanticSearchScope::new(
        VideoSourceId::new("deck-left").unwrap(),
        Some(
            TimeRange::new(
                UtcTimestamp::parse("2024-06-01T12:00:00Z").unwrap(),
                UtcTimestamp::parse("2024-06-01T13:00:00Z").unwrap(),
            )
            .unwrap(),
        ),
    )
    .unwrap();
    let coverage = SemanticIndexingCoverage::new(
        scope,
        model_space(1),
        SemanticIndexingStateCounts::new(12, 8, 3, 1),
        SemanticIndexingFreshness::new(
            Some(UtcTimestamp::parse("2024-06-01T12:00:30Z").unwrap()),
            None,
            Some(UtcTimestamp::parse("2024-06-01T12:00:30Z").unwrap()),
        ),
        false,
    );
    assert!(!coverage.scope_complete());
    assert!(coverage.indexing_incomplete());
}

#[test]
fn capability_schema_refs_match_delivered_transport_names() {
    let search = SemanticSearchSchemaRefs::registered();
    assert_eq!(search.search_request(), "SearchSemanticFramesRequest");
    assert_eq!(search.coverage_result(), "SemanticIndexingCoverage");
    let decisions = FrameDecisionSchemaRefs::registered();
    assert_eq!(decisions.append_request(), "AppendFrameDecisionRequest");
    assert_eq!(decisions.read_result(), "FrameDecisionHistoryResult");
}

#[test]
fn v2_example_config_matches_fixture_and_parses() {
    let example: serde_json::Value = serde_json::from_str(include_str!(
        "../schemas/a2a-lab-config-v2-semantic-example.json"
    ))
    .unwrap();
    let fixture: serde_json::Value = serde_json::from_str(VALID_V2_CONFIG).unwrap();
    let mut example_body = example.clone();
    example_body.as_object_mut().unwrap().remove("$id");
    assert_eq!(example_body, fixture);
    A2aLabConfig::parse(VALID_V2_CONFIG).unwrap();
}

#[test]
fn semantic_schema_file_set_is_stable() {
    let mut expected: BTreeSet<_> = SEMANTIC_SCHEMA_FILES.iter().copied().collect();
    for legacy in [
        "compliance-fixtures-1.1.0.json",
        "compliance-fixtures-1.2.0.json",
        "compliance-fixtures-1.3.0.json",
        "compliance-result-1.1.0.json",
        "compliance-result-1.2.0.json",
        "compliance-result-1.3.0.json",
    ] {
        expected.remove(legacy);
    }
    assert_eq!(expected.len(), SEMANTIC_SCHEMA_FILES.len());
}
