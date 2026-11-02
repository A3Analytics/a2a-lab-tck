//! In-process lab API matching the compliance runner fixture contract.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use a2a_lab_dev_kit::{
    A2aLabApi, A2aLabCommand, A2aLabError, A2aLabFuture, A2aLabOutcome, A2aLabResult,
    GetTaskStatusRequest, GetVideoFrameAtResult, Image, ImageDescriptor, ImageSource, JsonObject,
    LogSource, MJPEG_MEDIA_TYPE, MetricDescriptor, Page, TaskDefinition, TaskRun, TaskSnapshot,
    TaskState, TimeRange, UtcTimestamp, VideoClip, VideoCodec, VideoFollow,
    VideoPlayback8xMetadata, VideoSource, VideoTimeline,
};
use a2a_lab_dev_kit::{PackagedMediaAsset, ResolvedMediaAsset, VideoClipStream, VideoFollowId};
use a2a_lab_dev_kit::{RunId, VideoClipId, VideoLiveStream};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureVariant {
    Standard,
    McpMetricError,
    McpEmptyLogSources,
}

struct Inner {
    tasks: HashMap<String, StoredTask>,
    status_reads: HashMap<String, usize>,
}

#[derive(Clone)]
struct StoredTask {
    context_id: String,
    state: TaskState,
    result: A2aLabResult,
}

pub struct FixtureLab {
    inner: Mutex<Inner>,
    ids: AtomicU64,
}

impl FixtureLab {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                tasks: HashMap::new(),
                status_reads: HashMap::new(),
            }),
            ids: AtomicU64::new(0),
        }
    }

    #[must_use]
    pub fn share(self) -> Arc<dyn A2aLabApi> {
        Arc::new(self)
    }

    fn allocate_task_id(&self) -> String {
        let number = self.ids.fetch_add(1, Ordering::Relaxed) + 1;
        format!("task-{number}")
    }

    fn snapshot(&self, result: A2aLabResult) -> TaskSnapshot {
        let id = self.allocate_task_id();
        let context_id = format!("ctx-{id}");
        let snapshot = TaskSnapshot {
            id: id.clone(),
            context_id: context_id.clone(),
            state: TaskState::Completed,
            result: result.clone(),
        };
        self.inner.lock().unwrap().tasks.insert(
            id,
            StoredTask {
                context_id,
                state: TaskState::Completed,
                result,
            },
        );
        snapshot
    }
}

impl Default for FixtureLab {
    fn default() -> Self {
        Self::new()
    }
}

impl FixtureLab {
    fn dispatch(&self, command: A2aLabCommand) -> Result<A2aLabResult, A2aLabError> {
        let mut inner = self.inner.lock().unwrap();
        if let A2aLabCommand::GetTaskStatus(request) = &command
            && request.id.as_str() == "runs.started"
        {
            let reads = inner
                .status_reads
                .entry(request.id.to_string())
                .or_insert(0);
            *reads += 1;
            let state = if *reads == 1 {
                TaskState::Working
            } else {
                TaskState::Completed
            };
            return Ok(A2aLabResult::GetTaskStatus(task_run(
                request.id.clone(),
                parse_id("tasks.discovered"),
                JsonObject::empty(),
                state,
            )));
        }
        drop(inner);
        fixture_result(command)
    }
}

impl A2aLabApi for FixtureLab {
    fn execute(
        &self,
        command: A2aLabCommand,
    ) -> A2aLabFuture<'_, Result<A2aLabOutcome, A2aLabError>> {
        Box::pin(async move {
            let result = self.dispatch(command)?;
            Ok(A2aLabOutcome {
                task: self.snapshot(result),
            })
        })
    }

    fn task<'a>(&'a self, task_id: &str) -> A2aLabFuture<'a, Result<TaskSnapshot, A2aLabError>> {
        let task_id = task_id.to_owned();
        Box::pin(async move {
            self.inner
                .lock()
                .unwrap()
                .tasks
                .get(&task_id)
                .cloned()
                .map(|stored| TaskSnapshot {
                    id: task_id.clone(),
                    context_id: stored.context_id,
                    state: stored.state,
                    result: stored.result,
                })
                .ok_or_else(|| A2aLabError::not_found("task", task_id))
        })
    }

    fn cancel(
        &self,
        request: GetTaskStatusRequest,
    ) -> A2aLabFuture<'_, Result<TaskSnapshot, A2aLabError>> {
        let id = request.id.clone();
        Box::pin(async move { Err(A2aLabError::invalid("cancel", id.to_string())) })
    }

    fn open_video_clip(
        &self,
        _id: &VideoClipId,
    ) -> A2aLabFuture<'_, Result<VideoClipStream, A2aLabError>> {
        Box::pin(async move { Err(A2aLabError::not_found("video clip", "fixture")) })
    }

    fn export_finite_video_clip(
        &self,
        _id: &VideoClipId,
    ) -> A2aLabFuture<'_, Result<PackagedMediaAsset, A2aLabError>> {
        Box::pin(async move { Err(A2aLabError::not_found("video clip", "fixture")) })
    }
}

pub struct VariantLab {
    inner: Arc<dyn A2aLabApi>,
    variant: FixtureVariant,
}

impl VariantLab {
    #[must_use]
    pub fn wrap(inner: Arc<dyn A2aLabApi>, variant: FixtureVariant) -> Arc<dyn A2aLabApi> {
        if variant == FixtureVariant::Standard {
            return inner;
        }
        Arc::new(Self { inner, variant })
    }
}

impl A2aLabApi for VariantLab {
    fn execute(
        &self,
        command: A2aLabCommand,
    ) -> A2aLabFuture<'_, Result<A2aLabOutcome, A2aLabError>> {
        match self.variant {
            FixtureVariant::McpMetricError if matches!(command, A2aLabCommand::QueryMetric(_)) => {
                Box::pin(async move {
                    Err(A2aLabError::not_found(
                        "metric",
                        "controlled-compliance-mismatch",
                    ))
                })
            }
            FixtureVariant::McpEmptyLogSources
                if matches!(command, A2aLabCommand::ListLogSources(_)) =>
            {
                let inner = Arc::clone(&self.inner);
                Box::pin(async move {
                    let mut outcome = inner.execute(command).await?;
                    outcome.task.result = A2aLabResult::ListLogSources(Page::new(Vec::new(), None));
                    Ok(outcome)
                })
            }
            _ => self.inner.execute(command),
        }
    }

    fn task<'a>(&'a self, task_id: &str) -> A2aLabFuture<'a, Result<TaskSnapshot, A2aLabError>> {
        self.inner.task(task_id)
    }

    fn cancel(
        &self,
        request: GetTaskStatusRequest,
    ) -> A2aLabFuture<'_, Result<TaskSnapshot, A2aLabError>> {
        self.inner.cancel(request)
    }

    fn open_video_clip(
        &self,
        id: &VideoClipId,
    ) -> A2aLabFuture<'_, Result<VideoClipStream, A2aLabError>> {
        self.inner.open_video_clip(id)
    }

    fn export_finite_video_clip(
        &self,
        id: &VideoClipId,
    ) -> A2aLabFuture<'_, Result<PackagedMediaAsset, A2aLabError>> {
        self.inner.export_finite_video_clip(id)
    }

    fn open_video_follow(
        &self,
        id: &VideoFollowId,
    ) -> A2aLabFuture<'_, Result<VideoLiveStream, A2aLabError>> {
        self.inner.open_video_follow(id)
    }

    fn resolve_live_playback_asset(
        &self,
        id: &VideoFollowId,
        relative_path: Option<&str>,
    ) -> A2aLabFuture<'_, Result<ResolvedMediaAsset, A2aLabError>> {
        self.inner.resolve_live_playback_asset(id, relative_path)
    }

    fn rebase_video_follow(
        &self,
        id: &VideoFollowId,
        anchor: UtcTimestamp,
    ) -> A2aLabFuture<'_, Result<VideoFollow, A2aLabError>> {
        self.inner.rebase_video_follow(id, anchor)
    }
}

/// Lab API that always reports provider unavailability on both interfaces.
pub struct UnavailableLab;

impl UnavailableLab {
    #[must_use]
    pub fn share() -> Arc<dyn A2aLabApi> {
        Arc::new(Self)
    }
}

impl A2aLabApi for UnavailableLab {
    fn execute(
        &self,
        _command: A2aLabCommand,
    ) -> A2aLabFuture<'_, Result<A2aLabOutcome, A2aLabError>> {
        Box::pin(async move { Err(A2aLabError::unavailable("fixture unavailable")) })
    }

    fn task<'a>(&'a self, task_id: &str) -> A2aLabFuture<'a, Result<TaskSnapshot, A2aLabError>> {
        let task_id = task_id.to_owned();
        Box::pin(async move { Err(A2aLabError::not_found("task", task_id)) })
    }

    fn cancel(
        &self,
        request: GetTaskStatusRequest,
    ) -> A2aLabFuture<'_, Result<TaskSnapshot, A2aLabError>> {
        let id = request.id.to_string();
        Box::pin(async move { Err(A2aLabError::invalid("cancel", id)) })
    }

    fn open_video_clip(
        &self,
        _id: &VideoClipId,
    ) -> A2aLabFuture<'_, Result<VideoClipStream, A2aLabError>> {
        Box::pin(async move { Err(A2aLabError::unavailable("fixture unavailable")) })
    }

    fn export_finite_video_clip(
        &self,
        _id: &VideoClipId,
    ) -> A2aLabFuture<'_, Result<PackagedMediaAsset, A2aLabError>> {
        Box::pin(async move { Err(A2aLabError::unavailable("fixture unavailable")) })
    }
}

fn fixture_result(command: A2aLabCommand) -> Result<A2aLabResult, A2aLabError> {
    match command {
        A2aLabCommand::ListLogSources(request) => {
            request.page.check()?;
            Ok(A2aLabResult::ListLogSources(item_page(
                LogSource {
                    id: parse_id("logs.discovered"),
                    name: "Discovered logs".to_owned(),
                    description: "Linked scenario fixture".to_owned(),
                    asset_id: None,
                    semantic_id: None,
                },
                &request.page,
            )))
        }
        A2aLabCommand::QueryLogs(request) => {
            request.range.check()?;
            missing(request.source_id.as_str())?;
            Ok(A2aLabResult::QueryLogs(Page::new(Vec::new(), None)))
        }
        A2aLabCommand::ListMetrics(request) => {
            request.page.check()?;
            Ok(A2aLabResult::ListMetrics(item_page(
                MetricDescriptor {
                    id: parse_id("metrics.discovered"),
                    name: "Discovered metric".to_owned(),
                    description: "Linked scenario fixture".to_owned(),
                    unit: "count".to_owned(),
                    asset_id: None,
                    semantic_id: None,
                },
                &request.page,
            )))
        }
        A2aLabCommand::QueryMetric(request) => {
            request.range.check()?;
            missing(request.metric_id.as_str())?;
            Ok(A2aLabResult::QueryMetric(Page::new(Vec::new(), None)))
        }
        A2aLabCommand::ListTasks(request) => {
            request.page.check()?;
            Ok(A2aLabResult::ListTasks(item_page(
                TaskDefinition {
                    id: parse_id("tasks.discovered"),
                    name: "Discovered task".to_owned(),
                    description: "Linked scenario fixture".to_owned(),
                    asset_id: None,
                    semantic_id: None,
                    input_schema: None,
                    output_schema: None,
                },
                &request.page,
            )))
        }
        A2aLabCommand::StartTask(request) => {
            missing(request.task_id.as_str())?;
            Ok(A2aLabResult::StartTask(task_run(
                RunId::new("runs.started").unwrap(),
                request.task_id,
                request.input,
                TaskState::Submitted,
            )))
        }
        A2aLabCommand::GetTaskStatus(request) => {
            missing(request.id.as_str())?;
            let task_id = if request.id.as_str() == "runs.started" {
                parse_id("tasks.discovered")
            } else {
                parse_id("tasks.primary")
            };
            Ok(A2aLabResult::GetTaskStatus(task_run(
                request.id,
                task_id,
                JsonObject::empty(),
                TaskState::Completed,
            )))
        }
        command @ (A2aLabCommand::ListImageSources(_)
        | A2aLabCommand::ListImages(_)
        | A2aLabCommand::SearchImages(_)
        | A2aLabCommand::GetImage(_)
        | A2aLabCommand::GetCurrentImage(_)) => image_fixture_result(command),
        command @ (A2aLabCommand::ListVideoSources(_)
        | A2aLabCommand::GetVideoFrameAt(_)
        | A2aLabCommand::GetVideoClip(_)
        | A2aLabCommand::GetVideoTimeline(_)
        | A2aLabCommand::FollowVideo(_)) => video_fixture_result(command),
    }
}

fn image_fixture_result(command: A2aLabCommand) -> Result<A2aLabResult, A2aLabError> {
    match command {
        A2aLabCommand::ListImageSources(request) => {
            request.page().check()?;
            Ok(A2aLabResult::ListImageSources(item_page(
                ImageSource {
                    id: parse_id("images.sources.discovered"),
                    name: "Discovered camera".to_owned(),
                    description: "Linked scenario fixture".to_owned(),
                    asset_id: None,
                    semantic_id: None,
                },
                request.page(),
            )))
        }
        A2aLabCommand::ListImages(request) => {
            request.page().check()?;
            missing(request.source_id().as_str())?;
            let descriptor = if request.source_id().as_str() == "images.sources.discovered" {
                image_with("images.discovered", "images.sources.discovered", vec![2])
                    .descriptor()
                    .clone()
            } else {
                image().descriptor().clone()
            };
            Ok(A2aLabResult::ListImages(item_page(
                descriptor,
                request.page(),
            )))
        }
        A2aLabCommand::SearchImages(request) => {
            request.check()?;
            Ok(A2aLabResult::SearchImages(empty_page(request.page())))
        }
        A2aLabCommand::GetImage(request) => {
            missing(request.id().as_str())?;
            let image = if request.id().as_str() == "images.discovered" {
                image_with("images.discovered", "images.sources.discovered", vec![3])
            } else {
                image()
            };
            Ok(A2aLabResult::GetImage(image))
        }
        A2aLabCommand::GetCurrentImage(request) => {
            missing(request.source_id().as_str())?;
            let image = if request.source_id().as_str() == "images.sources.discovered" {
                image_with(
                    "images.current.different",
                    "images.sources.discovered",
                    vec![4],
                )
            } else {
                image()
            };
            Ok(A2aLabResult::GetCurrentImage(image))
        }
        _ => unreachable!("image fixture helper received a non-image command"),
    }
}

fn video_fixture_result(command: A2aLabCommand) -> Result<A2aLabResult, A2aLabError> {
    match command {
        A2aLabCommand::ListVideoSources(request) => {
            request.page().check()?;
            Ok(A2aLabResult::ListVideoSources(item_page(
                video_source("videos.sources.discovered"),
                request.page(),
            )))
        }
        A2aLabCommand::GetVideoFrameAt(request) => {
            video_coverage_missing(request.source_id().as_str(), request.at())?;
            Ok(A2aLabResult::GetVideoFrameAt(GetVideoFrameAtResult::new(
                request.at(),
                video_frame(request.source_id().as_str(), request.at()),
            )))
        }
        A2aLabCommand::GetVideoClip(request) => {
            request.check()?;
            video_clip_missing(request.source_id().as_str(), request.range())?;
            let clip = VideoClip::new(
                parse_id("videos.clips.primary"),
                request.source_id().clone(),
                request.range(),
                request.range(),
                VideoCodec::Mjpeg,
                MJPEG_MEDIA_TYPE,
            )
            .unwrap();
            Ok(A2aLabResult::GetVideoClip(clip_with_playback_8x(
                clip,
                request.source_id().as_str(),
                request.range(),
            )))
        }
        A2aLabCommand::GetVideoTimeline(request) => {
            video_timeline_missing(request.source_id().as_str())?;
            Ok(A2aLabResult::GetVideoTimeline(video_timeline(
                request.source_id().as_str(),
            )?))
        }
        A2aLabCommand::FollowVideo(request) => {
            video_follow_missing(request.source_id().as_str())?;
            Ok(A2aLabResult::FollowVideo(follow_with_playback_8x(
                request.source_id().as_str(),
                request.anchor_at(),
            )?))
        }
        _ => unreachable!("video fixture helper received a non-video command"),
    }
}

fn video_timeline(source_id: &str) -> Result<VideoTimeline, A2aLabError> {
    let source = parse_id::<a2a_lab_dev_kit::VideoSourceId>(source_id);
    if source_id.contains("empty") {
        return VideoTimeline::new(source, Vec::new(), None, false);
    }
    let first = TimeRange::new(
        timestamp("2026-01-01T00:00:00Z"),
        timestamp("2026-01-01T00:00:01Z"),
    )
    .unwrap();
    let second = TimeRange::new(
        timestamp("2026-01-01T01:00:00Z"),
        timestamp("2026-01-01T01:00:01Z"),
    )
    .unwrap();
    VideoTimeline::new(source, vec![first, second], Some(second.start()), true)
}

fn clip_with_playback_8x(clip: VideoClip, source_id: &str, range: TimeRange) -> VideoClip {
    if source_id == "videos.sources.primary"
        && range.start() == timestamp("2026-01-01T00:00:00Z")
        && range.end() == timestamp("2026-01-01T00:00:01Z")
    {
        clip.with_playback_8x(VideoPlayback8xMetadata::available(
            "sessions/primary/8x.m3u8",
            "sessions/primary/index.m3u8",
        ))
    } else {
        clip
    }
}

fn follow_with_playback_8x(
    source_id: &str,
    anchor_at: Option<UtcTimestamp>,
) -> Result<VideoFollow, A2aLabError> {
    let follow = VideoFollow::new_mjpeg(parse_id("videos.follow.primary"), parse_id(source_id))?;
    if source_id == "videos.sources.primary" && anchor_at.is_none() {
        Ok(
            follow.with_playback_8x(VideoPlayback8xMetadata::unavailable(
                "sessions/primary/live.m3u8",
                "trick_play_unavailable",
            )),
        )
    } else {
        Ok(follow)
    }
}

fn video_timeline_missing(source_id: &str) -> Result<(), A2aLabError> {
    missing(source_id)
}

fn video_follow_missing(source_id: &str) -> Result<(), A2aLabError> {
    missing(source_id)?;
    if source_id.contains("live-unavailable") {
        return Err(A2aLabError::live_unavailable(
            "fixture live follow unavailable",
        ));
    }
    Ok(())
}

fn video_source(source_id: &str) -> VideoSource {
    VideoSource::new(
        parse_id(source_id),
        "Discovered camera",
        "Linked scenario fixture",
        None,
        None,
        1,
        1,
        1,
        MJPEG_MEDIA_TYPE,
    )
    .unwrap()
}

fn video_frame(source_id: &str, captured_at: UtcTimestamp) -> Image {
    Image::new(
        ImageDescriptor::new(
            parse_id("videos.frames.primary"),
            parse_id(source_id),
            captured_at,
            "image/jpeg",
            1,
            1,
            None,
            JsonObject::empty(),
        )
        .unwrap(),
        vec![0xFF, 0xD8, 0xFF, 0xD9],
    )
    .unwrap()
}

fn video_coverage_missing(source_id: &str, at: UtcTimestamp) -> Result<(), A2aLabError> {
    missing(source_id)?;
    if source_id.contains("empty") {
        return Err(A2aLabError::not_found("video frame", source_id));
    }
    let stamp = at.to_rfc3339();
    if stamp.contains("T12:") || stamp.starts_with("2026-01-02") {
        return Err(A2aLabError::not_found("video frame", stamp));
    }
    Ok(())
}

fn video_clip_missing(source_id: &str, range: TimeRange) -> Result<(), A2aLabError> {
    missing(source_id)?;
    if source_id.contains("empty") {
        return Err(A2aLabError::not_found("video coverage", source_id));
    }
    let stamp = range.start().to_rfc3339();
    if stamp.contains("T12:") || stamp.starts_with("2026-01-02") {
        return Err(A2aLabError::not_found("video coverage", stamp));
    }
    Ok(())
}

fn empty_page<T>(request: &a2a_lab_dev_kit::PageRequest) -> Page<T> {
    Page::new(Vec::new(), (request.limit() == 1).then(|| "1".to_owned()))
}

fn item_page<T>(item: T, request: &a2a_lab_dev_kit::PageRequest) -> Page<T> {
    Page::new(vec![item], (request.limit() == 1).then(|| "1".to_owned()))
}

fn missing(value: &str) -> Result<(), A2aLabError> {
    if value.contains("missing") {
        Err(A2aLabError::not_found("fixture", value))
    } else {
        Ok(())
    }
}

fn task_run(
    id: a2a_lab_dev_kit::RunId,
    task_id: a2a_lab_dev_kit::TaskId,
    input: JsonObject,
    state: TaskState,
) -> TaskRun {
    TaskRun {
        id,
        task_id,
        state,
        input,
        message: None,
        result: None,
        progress: None,
        error_kind: None,
        error_identifier: None,
    }
}

fn image() -> Image {
    image_with("images.primary", "images.sources.primary", vec![1])
}

fn image_with(image_id: &str, source_id: &str, data: Vec<u8>) -> Image {
    Image::new(
        ImageDescriptor::new(
            parse_id(image_id),
            parse_id(source_id),
            timestamp("2026-01-01T00:00:00Z"),
            "image/png",
            1,
            1,
            None,
            JsonObject::empty(),
        )
        .unwrap(),
        data,
    )
    .unwrap()
}

fn timestamp(value: &str) -> UtcTimestamp {
    UtcTimestamp::parse(value).unwrap()
}

fn parse_id<T>(value: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(serde_json::Value::String(value.to_owned())).unwrap()
}
