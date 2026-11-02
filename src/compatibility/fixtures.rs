//! Deterministic profile 1.3.0 fixtures for compatibility endpoints.

use a2a_lab_dev_kit::{JsonObject, TimeRange, UtcTimestamp};

use crate::{
    COMPLIANCE_PROFILE_VERSION, ComplianceFixtures, FixtureCapability, ImageFixtures, LogFixtures,
    MetricFixtures, TaskFixtures, VideoFixtures,
};

/// Standard compliant fixture document shared by compatibility matrix cases.
#[must_use]
pub fn standard_fixtures() -> ComplianceFixtures {
    ComplianceFixtures {
        suite_version: COMPLIANCE_PROFILE_VERSION.to_owned(),
        range: TimeRange::new(
            timestamp("2026-01-01T00:00:00Z"),
            timestamp("2026-01-02T00:00:00Z"),
        )
        .unwrap(),
        logs: FixtureCapability::Required(LogFixtures {
            source_id: id("logs.primary"),
            alternate_source_id: id("logs.secondary"),
            missing_source_id: id("logs.missing"),
        }),
        metrics: FixtureCapability::Required(MetricFixtures {
            metric_id: id("metrics.primary"),
            alternate_metric_id: id("metrics.secondary"),
            missing_metric_id: id("metrics.missing"),
        }),
        tasks: FixtureCapability::Required(TaskFixtures {
            task_id: id("tasks.primary"),
            alternate_task_id: id("tasks.secondary"),
            run_id: id("runs.primary"),
            missing_task_id: id("tasks.missing"),
            missing_run_id: id("runs.missing"),
            input: JsonObject::parse(r#"{"duration":1}"#).unwrap(),
        }),
        images: FixtureCapability::Required(ImageFixtures {
            source_id: id("images.sources.primary"),
            alternate_source_id: id("images.sources.secondary"),
            image_id: id("images.primary"),
            alternate_image_id: id("images.secondary"),
            missing_source_id: id("images.sources.missing"),
            missing_image_id: id("images.missing"),
        }),
        videos: FixtureCapability::Required(VideoFixtures {
            source_id: id("videos.sources.primary"),
            alternate_source_id: id("videos.sources.secondary"),
            missing_source_id: id("videos.sources.missing"),
            empty_source_id: id("videos.sources.empty"),
            frame_at: timestamp("2026-01-01T00:00:00Z"),
            nearest_at: timestamp("2026-01-01T00:00:00.500000Z"),
            outside_at: timestamp("2026-01-02T00:00:00Z"),
            gap_at: timestamp("2026-01-01T12:00:00Z"),
            clip_range: TimeRange::new(
                timestamp("2026-01-01T00:00:00Z"),
                timestamp("2026-01-01T00:00:01Z"),
            )
            .unwrap(),
            outside_range: TimeRange::new(
                timestamp("2026-01-02T00:00:00Z"),
                timestamp("2026-01-02T00:00:01Z"),
            )
            .unwrap(),
            gap_range: TimeRange::new(
                timestamp("2026-01-01T12:00:00Z"),
                timestamp("2026-01-01T12:00:01Z"),
            )
            .unwrap(),
            long_clip_range: TimeRange::new(
                timestamp("2026-01-01T00:00:00Z"),
                timestamp("2026-01-01T00:01:01Z"),
            )
            .unwrap(),
            live_unavailable_source_id: id("videos.sources.live-unavailable"),
            follow_anchor_at: timestamp("2026-01-01T00:00:00Z"),
        }),
    }
}

/// Fixture document with every capability marked unavailable.
#[must_use]
pub fn unavailable_fixtures() -> ComplianceFixtures {
    ComplianceFixtures {
        logs: FixtureCapability::Unavailable,
        metrics: FixtureCapability::Unavailable,
        tasks: FixtureCapability::Unavailable,
        images: FixtureCapability::Unavailable,
        videos: FixtureCapability::Unavailable,
        ..standard_fixtures()
    }
}

fn id<T>(value: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(serde_json::Value::String(value.to_owned())).unwrap()
}

fn timestamp(value: &str) -> UtcTimestamp {
    UtcTimestamp::parse(value).unwrap_or_else(|error| panic!("{value}: {error}"))
}
