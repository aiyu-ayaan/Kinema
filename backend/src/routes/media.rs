use crate::access;
use crate::auth::AuthUser;
use crate::error::{ApiError, ApiResult};
use crate::server_settings::lang_matches;
use crate::state::{AppState, Counted};
use crate::thumbs;
use axum::body::Body;
use axum::extract::{Path, Query, Request, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use futures_util::StreamExt;
use kinema_db::items::{self, MediaFile, Stream};
use kinema_media::playback::{self, ClientCaps, Plan, Track};
use kinema_media::subtitles;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use tokio_util::io::ReaderStream;
use tower::ServiceExt;
use tower_http::services::ServeFile;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/files/:id/plan", get(plan))
        .route("/stream/:id", get(stream))
        .route("/subtitles/:stream_id", get(subtitle))
        .route("/images/:size/:name", get(image))
        .route("/items/:id/thumb", get(thumb))
}

#[derive(Deserialize)]
struct PlayQuery {
    /// Audio stream index; the file's default when absent.
    audio: Option<i64>,
    #[serde(default)]
    start: f64,
    #[serde(default)]
    hevc: bool,
    #[serde(default)]
    av1: bool,
}

struct Chosen {
    file: MediaFile,
    streams: Vec<Stream>,
    video: Option<usize>,
    audio: Option<usize>,
    plan: Plan,
}

fn track(s: &Stream) -> Track<'_> {
    Track {
        index: s.idx,
        codec: s.codec.as_deref(),
        profile: s.profile.as_deref(),
    }
}

async fn choose(state: &AppState, file_id: i64, q: &PlayQuery) -> ApiResult<Chosen> {
    let file = items::file(&state.db, file_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let streams = items::streams(&state.db, file_id).await?;
    let video = streams.iter().position(|s| s.kind == "video");
    let audios: Vec<usize> = streams
        .iter()
        .enumerate()
        .filter(|(_, s)| s.kind == "audio")
        .map(|(i, _)| i)
        .collect();
    let preferred = state.settings().preferred_audio_language;
    let audio = q
        .audio
        .and_then(|idx| audios.iter().copied().find(|&i| streams[i].idx == idx))
        .or_else(|| {
            audios
                .iter()
                .copied()
                .find(|&i| lang_matches(streams[i].language.as_deref(), &preferred))
        })
        .or_else(|| audios.iter().copied().find(|&i| streams[i].is_default))
        .or_else(|| audios.first().copied());
    let caps = ClientCaps {
        hevc: q.hevc,
        av1: q.av1,
    };
    // Unprobed files (no ffprobe, or probe failed) go out raw: never hide a file.
    let plan = if file.probe_status != "ok" {
        Plan::Direct
    } else {
        playback::plan(
            std::path::Path::new(&file.path),
            video.map(|i| track(&streams[i])),
            audio.map(|i| track(&streams[i])),
            audio == audios.first().copied(),
            caps,
        )
    };
    Ok(Chosen {
        file,
        streams,
        video,
        audio,
        plan,
    })
}

#[derive(Serialize)]
struct PlanResponse {
    mode: &'static str,
    duration_s: Option<f64>,
    audio: Option<i64>,
    /// Where the stream for the requested start really begins; the player's time offset.
    start: f64,
    /// Subtitle stream id the Languages settings ask for; the player applies it on first load.
    subtitle: Option<i64>,
}

/// The default subtitle per the admin's subtitle mode and language.
fn default_subtitle(state: &AppState, c: &Chosen) -> Option<i64> {
    let s = state.settings();
    let audio_lang = c.audio.and_then(|i| c.streams[i].language.as_deref());
    let wanted = match s.subtitle_mode.as_str() {
        "always" => true,
        "foreign" => {
            !s.preferred_audio_language.is_empty()
                && !lang_matches(audio_lang, &s.preferred_audio_language)
        }
        _ => false,
    };
    if !wanted {
        return None;
    }
    let text: Vec<&Stream> = c
        .streams
        .iter()
        .filter(|st| st.kind == "subtitle" && subtitles::is_text_subtitle(st.codec.as_deref()))
        .collect();
    text.iter()
        .find(|st| lang_matches(st.language.as_deref(), &s.preferred_subtitle_language))
        .or_else(|| {
            text.first()
                .filter(|_| s.preferred_subtitle_language.is_empty())
        })
        .map(|st| st.id)
}

async fn plan(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    Query(q): Query<PlayQuery>,
) -> ApiResult<Json<PlanResponse>> {
    access::check_file(&state, &user, id).await?;
    let c = choose(&state, id, &q).await?;
    let start = match (c.plan, &state.bins) {
        (Plan::Direct, _) => 0.0,
        (
            Plan::Stream {
                copy_video: true, ..
            },
            Some(bins),
        ) => {
            let mapped: Vec<i64> = [c.video, c.audio]
                .into_iter()
                .flatten()
                .map(|i| c.streams[i].idx)
                .collect();
            playback::seek_landing(bins, std::path::Path::new(&c.file.path), q.start, &mapped).await
        }
        // Re-encoding decodes up to `start` and cuts there exactly.
        _ => q.start.max(0.0),
    };
    Ok(Json(PlanResponse {
        mode: c.plan.name(),
        duration_s: c.file.duration_s,
        audio: c.audio.map(|i| c.streams[i].idx),
        start,
        subtitle: default_subtitle(&state, &c),
    }))
}

async fn stream(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    Query(q): Query<PlayQuery>,
    req: Request,
) -> ApiResult<Response> {
    access::check_file(&state, &user, id).await?;
    let c = choose(&state, id, &q).await?;
    let path = PathBuf::from(&c.file.path);
    let Plan::Stream {
        copy_video,
        copy_audio,
    } = c.plan
    else {
        // ServeFile handles Range and streams from disk without loading the file.
        let res = ServeFile::new(&path)
            .oneshot(req)
            .await
            .map_err(|_| ApiError::internal())?;
        return Ok(res.into_response());
    };
    let bins = state
        .bins
        .clone()
        .ok_or_else(|| ApiError::bad_request("media.ffmpeg_missing"))?;
    let settings = state.settings();
    // Re-encoding is the only CPU-heavy path; cap it so one server cannot be swamped.
    let transcode_slot = if copy_video {
        None
    } else {
        if state.active_transcodes.load(Ordering::SeqCst) >= settings.max_transcodes as usize {
            return Err(ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "media.transcode_busy",
            ));
        }
        Some(Counted::new(&state.active_transcodes))
    };
    let stream_slot = Counted::new(&state.active_streams);
    let mut child = playback::stream_command(
        &bins,
        &path,
        q.start.max(0.0),
        c.video.map(|i| track(&c.streams[i])),
        c.audio.map(|i| track(&c.streams[i])),
        copy_video,
        copy_audio,
        playback::Encode {
            preset: &settings.transcode_preset,
            crf: settings.transcode_crf,
            max_height: settings.transcode_max_height,
        },
    )
    .spawn()?;
    let stdout = child.stdout.take().ok_or_else(ApiError::internal)?;
    // The stream owns the child: when the client goes away the body drops and
    // kill_on_drop stops ffmpeg, so abandoned sessions never keep encoding.
    // The counters live as long as the stream, so the dashboard sees real activity.
    let body = ReaderStream::with_capacity(stdout, 256 * 1024).map(move |chunk| {
        let _owner = (&child, &stream_slot, &transcode_slot);
        chunk
    });
    Ok((
        [
            (CONTENT_TYPE, HeaderValue::from_static("video/mp4")),
            (CACHE_CONTROL, HeaderValue::from_static("no-store")),
        ],
        Body::from_stream(body),
    )
        .into_response())
}

/// WebVTT for a subtitle stream, extracted once and cached by file mtime.
async fn subtitle(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(stream_id): Path<i64>,
) -> ApiResult<Response> {
    let s = items::stream(&state.db, stream_id)
        .await?
        .filter(|s| s.kind == "subtitle")
        .ok_or_else(ApiError::not_found)?;
    access::check_file(&state, &user, s.file_id).await?;
    if !subtitles::is_text_subtitle(s.codec.as_deref()) {
        return Err(ApiError::bad_request("media.subtitle_unsupported"));
    }
    let file = items::file(&state.db, s.file_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let dir = state.cache_dir("subtitles");
    let out = dir.join(format!("{}-{}-{}.vtt", file.id, file.mtime, s.idx));
    if !out.exists() {
        let bins = state
            .bins
            .clone()
            .ok_or_else(|| ApiError::bad_request("media.ffmpeg_missing"))?;
        tokio::fs::create_dir_all(&dir).await?;
        match &s.external_path {
            Some(ext) => {
                subtitles::extract_vtt(&bins, std::path::Path::new(ext), None, &out).await?
            }
            None => {
                subtitles::extract_vtt(&bins, std::path::Path::new(&file.path), Some(s.idx), &out)
                    .await?
            }
        }
    }
    let body = tokio::fs::read(&out).await?;
    Ok((
        [
            (
                CONTENT_TYPE,
                HeaderValue::from_static("text/vtt; charset=utf-8"),
            ),
            (
                CACHE_CONTROL,
                HeaderValue::from_static("private, max-age=3600"),
            ),
        ],
        body,
    )
        .into_response())
}

fn valid_image_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() < 128
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
        && !name.starts_with('.')
        && (name.ends_with(".jpg") || name.ends_with(".png"))
}

/// TMDB or TVmaze artwork, fetched on first view and kept on disk afterwards.
async fn image(
    State(state): State<AppState>,
    _: AuthUser,
    Path((size, name)): Path<(String, String)>,
    req: Request,
) -> ApiResult<Response> {
    let tvmaze = size == kinema_meta::tvmaze::IMAGE_PREFIX;
    if !(tvmaze || kinema_meta::IMAGE_SIZES.contains(&size.as_str())) || !valid_image_name(&name) {
        return Err(ApiError::bad_request("media.image_invalid"));
    }
    let dir = state.cache_dir("images").join(&size);
    let path = dir.join(&name);
    if !path.exists() {
        let bytes = if tvmaze {
            kinema_meta::tvmaze::fetch_image(&state.http, &name).await
        } else {
            kinema_meta::fetch_image(&state.http, &size, &name).await
        }
        .map_err(|_| ApiError::not_found())?;
        tokio::fs::create_dir_all(&dir).await?;
        let tmp = dir.join(format!("{name}.tmp"));
        tokio::fs::write(&tmp, &bytes).await?;
        tokio::fs::rename(&tmp, &path).await?;
    }
    let mut res = ServeFile::new(&path)
        .oneshot(req)
        .await
        .map_err(|_| ApiError::internal())?
        .into_response();
    // Keys are content-addressed by TMDB and TVmaze, so the bytes behind a key never change.
    res.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=31536000, immutable"),
    );
    if res.status() == StatusCode::NOT_FOUND {
        return Err(ApiError::not_found());
    }
    Ok(res)
}

/// A video frame for items without artwork. Cache only: frames are made by the
/// background job (`thumbs.rs`), so this never runs ffmpeg. A miss nudges the job.
async fn thumb(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    req: Request,
) -> ApiResult<Response> {
    access::check_item(&state, &user, id).await?;
    let file = items::thumb_file(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let (out, failed) = thumbs::paths(&state, &file);
    if !out.exists() {
        if !failed.exists()
            && let Some(card) = items::card(&state.db, 0, id).await?
        {
            thumbs::spawn(state.clone(), card.library_id);
        }
        return Err(ApiError::not_found());
    }
    let mut res = ServeFile::new(&out)
        .oneshot(req)
        .await
        .map_err(|_| ApiError::internal())?
        .into_response();
    // Short cache: "Regenerate thumbnails" should show up on the next visit.
    res.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=600"),
    );
    Ok(res)
}
