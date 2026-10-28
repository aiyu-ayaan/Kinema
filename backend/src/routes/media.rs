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
use kinema_db::profiles;
use kinema_media::playback::{self, ClientCaps, Plan, Track};
use kinema_media::subtitles;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio_util::io::ReaderStream;
use tower::ServiceExt;
use tower_http::services::ServeFile;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/files/:id/plan", get(plan))
        .route("/stream/:id", get(stream))
        .route("/hls/:id/index.m3u8", get(hls_playlist))
        .route("/hls/:id/:segment", get(hls_segment))
        .route("/subtitles/:stream_id", get(subtitle))
        .route("/images/:size/:name", get(image))
        .route("/items/:id/thumb", get(thumb))
}

/// Default track languages for one viewer: the profile's own where set, else the server's
/// Languages settings. A profile value of "" means "the file's default track".
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Langs {
    audio: String,
    subtitle: String,
    mode: String,
}

impl Langs {
    pub(crate) fn server(state: &AppState) -> Self {
        let s = state.settings();
        Self {
            audio: s.preferred_audio_language,
            subtitle: s.preferred_subtitle_language,
            mode: s.subtitle_mode,
        }
    }

    pub(crate) fn with_prefs(self, p: &profiles::Prefs) -> Self {
        Self {
            audio: p.audio_language.clone().unwrap_or(self.audio),
            subtitle: p.subtitle_language.clone().unwrap_or(self.subtitle),
            mode: p.subtitle_mode.clone().unwrap_or(self.mode),
        }
    }

    pub(crate) async fn for_user(state: &AppState, user_id: i64) -> ApiResult<Self> {
        let prefs = profiles::get(&state.db, user_id).await?;
        Ok(Self::server(state).with_prefs(&prefs))
    }
}

#[derive(Deserialize)]
pub(crate) struct PlayQuery {
    /// Audio stream index; the file's default when absent.
    audio: Option<i64>,
    #[serde(default)]
    start: f64,
    #[serde(default)]
    hevc: bool,
    #[serde(default)]
    av1: bool,
    /// The file as it is, whatever it holds: for a player that decodes anything (mpv in the desktop app).
    #[serde(default)]
    raw: bool,
}

struct Chosen {
    file: MediaFile,
    streams: Vec<Stream>,
    video: Option<usize>,
    audio: Option<usize>,
    plan: Plan,
    series: items::SeriesTracks,
}

fn track(s: &Stream) -> Track<'_> {
    Track {
        index: s.idx,
        codec: s.codec.as_deref(),
        profile: s.profile.as_deref(),
    }
}

async fn choose(
    state: &AppState,
    user_id: i64,
    langs: &Langs,
    file_id: i64,
    q: &PlayQuery,
) -> ApiResult<Chosen> {
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
    let preferred = &langs.audio;
    // The profile's own choice (made in any player) wins over the show's admin choice.
    let mine = items::file_track_choice(&state.db, user_id, file_id).await?;
    let show = items::file_series_tracks(&state.db, file_id).await?;
    let series = items::SeriesTracks {
        audio: mine.audio.or(show.audio),
        subtitle: mine.subtitle.or(show.subtitle),
    };
    let audio = q
        .audio
        .and_then(|idx| audios.iter().copied().find(|&i| streams[i].idx == idx))
        .or_else(|| {
            let n = usize::try_from(series.audio?).ok()?;
            audios.get(n).copied()
        })
        .or_else(|| {
            audios
                .iter()
                .copied()
                .find(|&i| lang_matches(streams[i].language.as_deref(), preferred))
        })
        .or_else(|| audios.iter().copied().find(|&i| streams[i].is_default))
        .or_else(|| audios.first().copied());
    let caps = ClientCaps {
        hevc: q.hevc,
        av1: q.av1,
    };
    // Unprobed files (no ffprobe, or probe failed) go out raw: never hide a file.
    let plan = if file.probe_status != "ok" || q.raw {
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
        series,
    })
}

#[derive(Serialize)]
pub(crate) struct PlanResponse {
    mode: &'static str,
    duration_s: Option<f64>,
    audio: Option<i64>,
    /// Where the stream for the requested start really begins; the player's time offset.
    start: f64,
    /// Subtitle stream id the Languages settings ask for; the player applies it on first load.
    subtitle: Option<i64>,
}

/// A text track, or a picture track already read by OCR: either has a WebVTT to show.
fn has_vtt(s: &Stream) -> bool {
    subtitles::is_text_subtitle(s.codec.as_deref()) || s.ocr
}

/// The default subtitle per the viewer's (else the server's) subtitle mode and language.
fn default_subtitle(langs: &Langs, c: &Chosen) -> Option<i64> {
    // The show's own choice wins; -1 is off. Picture subtitles without OCR cannot be shown, so they mean off too.
    if let Some(n) = c.series.subtitle {
        let n = usize::try_from(n).ok()?;
        return c
            .streams
            .iter()
            .filter(|st| st.kind == "subtitle")
            .nth(n)
            .filter(|st| has_vtt(st))
            .map(|st| st.id);
    }
    let audio_lang = c.audio.and_then(|i| c.streams[i].language.as_deref());
    let wanted = match langs.mode.as_str() {
        "always" => true,
        "foreign" => !langs.audio.is_empty() && !lang_matches(audio_lang, &langs.audio),
        _ => false,
    };
    if !wanted {
        return None;
    }
    let text: Vec<&Stream> = c
        .streams
        .iter()
        .filter(|st| st.kind == "subtitle" && has_vtt(st))
        .collect();
    text.iter()
        .find(|st| lang_matches(st.language.as_deref(), &langs.subtitle))
        .or_else(|| text.first().filter(|_| langs.subtitle.is_empty()))
        .map(|st| st.id)
}

async fn plan(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    Query(q): Query<PlayQuery>,
) -> ApiResult<Json<PlanResponse>> {
    access::check_file(&state, &user, id).await?;
    let langs = Langs::for_user(&state, user.id).await?;
    plan_for(&state, user.id, &langs, id, &q).await.map(Json)
}

/// The plan for a file the caller already checked access to. `user_id` 0 (a share link)
/// has no per-title track choices.
pub(crate) async fn plan_for(
    state: &AppState,
    user_id: i64,
    langs: &Langs,
    id: i64,
    q: &PlayQuery,
) -> ApiResult<PlanResponse> {
    let c = choose(state, user_id, langs, id, q).await?;
    let start = match (c.plan, &state.bins) {
        (Plan::Direct, _) => 0.0,
        (
            Plan::Stream {
                copy_video: true, ..
            },
            Some(bins),
        ) => {
            // The stored keyframe index answers at once; without one (not built yet, or no cheap
            // source), ask ffprobe.
            match crate::keyframes::load(state, id).await {
                Some(index) => index.landing(q.start),
                None => {
                    let mapped: Vec<i64> = [c.video, c.audio]
                        .into_iter()
                        .flatten()
                        .map(|i| c.streams[i].idx)
                        .collect();
                    playback::seek_landing(
                        bins,
                        std::path::Path::new(&c.file.path),
                        q.start,
                        &mapped,
                    )
                    .await
                }
            }
        }
        // Re-encoding decodes up to `start` and cuts there exactly.
        _ => q.start.max(0.0),
    };
    Ok(PlanResponse {
        mode: c.plan.name(),
        duration_s: c.file.duration_s,
        audio: c.audio.map(|i| c.streams[i].idx),
        start,
        subtitle: default_subtitle(langs, &c),
    })
}

async fn stream(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    Query(q): Query<PlayQuery>,
    req: Request,
) -> ApiResult<Response> {
    access::check_file(&state, &user, id).await?;
    let langs = Langs::for_user(&state, user.id).await?;
    stream_for(&state, user.id, &langs, id, &q, req, None).await
}

/// Re-encodes one share link may run at once.
pub(crate) const SHARE_TRANSCODES_PER_LINK: usize = 1;

/// The stream for a file the caller already checked access to. `link_transcodes` is the running re-encode
/// count of the share link asking, if any.
pub(crate) async fn stream_for(
    state: &AppState,
    user_id: i64,
    langs: &Langs,
    id: i64,
    q: &PlayQuery,
    req: Request,
    link_transcodes: Option<&Arc<AtomicUsize>>,
) -> ApiResult<Response> {
    let c = choose(state, user_id, langs, id, q).await?;
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
    // A share link gets its own small share of the re-encodes (see `shares.rs`).
    let link_slot = match link_transcodes {
        Some(count) if !copy_video => {
            if count.load(Ordering::SeqCst) >= SHARE_TRANSCODES_PER_LINK {
                return Err(ApiError::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "shares.transcode_busy",
                ));
            }
            Some(Counted::new(count))
        }
        _ => None,
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
            threads: playback::transcode_threads(
                settings.transcode_threads,
                settings.max_transcodes,
            ),
            hardware: settings.transcode_hardware,
        },
    )
    .spawn()?;
    let started = std::time::Instant::now();
    let stdout = child.stdout.take().ok_or_else(ApiError::internal)?;
    let progress = child.stderr.take().map(playback::Progress::watch);
    // The stream owns the child: when the client goes away the body drops and
    // kill_on_drop stops ffmpeg, so abandoned sessions never keep encoding.
    // The counters live as long as the stream, so the dashboard sees real activity.
    let body = ReaderStream::with_capacity(stdout, 256 * 1024)
        .then(move |chunk| {
            let progress = progress.clone();
            async move {
                if let Some(p) = progress {
                    p.pace(started).await;
                }
                chunk
            }
        })
        .map(move |chunk| {
            let _owner = (&child, &stream_slot, &transcode_slot, &link_slot);
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

/// (copy video, copy audio) for HLS. MPEG-TS segments carry AAC and MP3 as they are; other audio is
/// re-encoded. A file the browser could direct-play is remuxed whole.
fn hls_mode(c: &Chosen) -> ApiResult<(bool, bool)> {
    if c.file.probe_status != "ok" {
        return Err(ApiError::new(StatusCode::CONFLICT, "media.hls_unavailable"));
    }
    let (copy_video, copy_audio) = match c.plan {
        Plan::Direct => (c.video.is_some(), true),
        Plan::Stream {
            copy_video,
            copy_audio,
        } => (copy_video, copy_audio),
    };
    let ts_audio = c
        .audio
        .is_none_or(|i| matches!(c.streams[i].codec.as_deref(), Some("aac" | "mp3")));
    Ok((copy_video, copy_audio && ts_audio))
}

/// Segment start times: keyframes for copied video (needs the stored index), a fixed grid otherwise.
async fn hls_starts(state: &AppState, c: &Chosen, copy_video: bool) -> ApiResult<Vec<f64>> {
    let duration = c
        .file
        .duration_s
        .ok_or_else(|| ApiError::new(StatusCode::CONFLICT, "media.hls_unavailable"))?;
    if !copy_video {
        return Ok(kinema_media::hls::grid_segments(duration));
    }
    let index = crate::keyframes::load(state, c.file.id)
        .await
        .ok_or_else(|| ApiError::new(StatusCode::CONFLICT, "media.hls_unavailable"))?;
    Ok(kinema_media::hls::copy_segments(&index.times, duration))
}

/// The chosen audio pinned in the segment URLs, so every segment reaches the same session.
fn hls_query(c: &Chosen, q: &PlayQuery) -> String {
    let audio = c
        .audio
        .map(|i| format!("audio={}&", c.streams[i].idx))
        .unwrap_or_default();
    format!("{audio}hevc={}&av1={}", q.hevc, q.av1)
}

async fn hls_playlist(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    Query(q): Query<PlayQuery>,
) -> ApiResult<Response> {
    access::check_file(&state, &user, id).await?;
    let langs = Langs::for_user(&state, user.id).await?;
    let c = choose(&state, user.id, &langs, id, &q).await?;
    let (copy_video, _) = hls_mode(&c)?;
    let starts = hls_starts(&state, &c, copy_video).await?;
    let body = kinema_media::hls::playlist(
        &starts,
        c.file.duration_s.unwrap_or_default(),
        &hls_query(&c, &q),
    );
    Ok((
        [
            (
                CONTENT_TYPE,
                HeaderValue::from_static("application/vnd.apple.mpegurl"),
            ),
            (CACHE_CONTROL, HeaderValue::from_static("no-store")),
        ],
        body,
    )
        .into_response())
}

async fn hls_segment(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id, segment)): Path<(i64, String)>,
    Query(q): Query<PlayQuery>,
    req: Request,
) -> ApiResult<Response> {
    let n: usize = segment
        .strip_suffix(".ts")
        .and_then(|s| s.parse().ok())
        .ok_or_else(ApiError::not_found)?;
    access::check_file(&state, &user, id).await?;
    let langs = Langs::for_user(&state, user.id).await?;
    let c = choose(&state, user.id, &langs, id, &q).await?;
    let (copy_video, copy_audio) = hls_mode(&c)?;
    let audio_idx = c.audio.map_or(-1, |i| c.streams[i].idx);
    let key = format!(
        "{}-{id}-{audio_idx}-{}-{}",
        user.id, copy_video as u8, copy_audio as u8
    );
    let owned = |i: usize| crate::hls::OwnedTrack {
        index: c.streams[i].idx,
        codec: c.streams[i].codec.clone(),
        profile: c.streams[i].profile.clone(),
    };
    let session = state
        .hls
        .session(&state, key, user.id, id, async || {
            let settings = state.settings();
            Ok(crate::hls::Spec {
                path: PathBuf::from(&c.file.path),
                video: c.video.map(owned),
                audio: c.audio.map(owned),
                copy_video,
                copy_audio,
                preset: settings.transcode_preset.clone(),
                crf: settings.transcode_crf,
                max_height: settings.transcode_max_height,
                threads: playback::transcode_threads(
                    settings.transcode_threads,
                    settings.max_transcodes,
                ),
                hardware: settings.transcode_hardware,
                starts: hls_starts(&state, &c, copy_video).await?,
            })
        })
        .await?;
    let path = state.hls.segment(&state, &session, n).await?;
    let mut res = ServeFile::new(&path)
        .oneshot(req)
        .await
        .map_err(|_| ApiError::internal())?
        .into_response();
    let headers = res.headers_mut();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("video/mp2t"));
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=3600"),
    );
    Ok(res)
}

/// WebVTT for a subtitle stream. Normally made by the background job after a scan
/// (`crate::subtitles`); a text track it has not reached yet is extracted here once.
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
    subtitle_for(&state, s).await
}

/// WebVTT for a subtitle stream the caller already checked access to.
pub(crate) async fn subtitle_for(state: &AppState, s: Stream) -> ApiResult<Response> {
    let text = subtitles::is_text_subtitle(s.codec.as_deref());
    if !text && !s.ocr {
        return Err(ApiError::bad_request("media.subtitle_unsupported"));
    }
    let file = items::file(&state.db, s.file_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let dir = state.cache_dir("subtitles");
    let key = subtitles::CacheKey {
        file_id: file.id,
        size: file.size,
        mtime: file.mtime,
        idx: s.idx,
        sidecar: s
            .external_path
            .as_deref()
            .map(|p| (p, s.external_fingerprint.as_deref().unwrap_or("0"))),
    };
    let (out, failed) = key.paths(&dir);
    if !out.exists() {
        // OCR is far too slow for a request; only the background job reads picture tracks.
        if !text {
            return Err(ApiError::bad_request("media.subtitle_unsupported"));
        }
        if failed.exists() {
            return Err(ApiError::bad_request("media.subtitle_failed"));
        }
        let bins = state
            .bins
            .clone()
            .ok_or_else(|| ApiError::bad_request("media.ffmpeg_missing"))?;
        tokio::fs::create_dir_all(&dir).await?;
        let made = match &s.external_path {
            Some(ext) => subtitles::extract_vtt(&bins, std::path::Path::new(ext), None, &out).await,
            None => {
                subtitles::extract_vtt(&bins, std::path::Path::new(&file.path), Some(s.idx), &out)
                    .await
            }
        };
        if made.is_err() {
            let _ = tokio::fs::write(&failed, b"").await;
            return Err(ApiError::bad_request("media.subtitle_failed"));
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
    AuthUser(user): AuthUser,
    Path((size, name)): Path<(String, String)>,
    req: Request,
) -> ApiResult<Response> {
    access::check_image(&state, &user, &name).await?;
    image_for(&state, &size, &name, req).await
}

pub(crate) async fn image_for(
    state: &AppState,
    size: &str,
    name: &str,
    req: Request,
) -> ApiResult<Response> {
    let tvmaze = size == kinema_meta::tvmaze::IMAGE_PREFIX;
    if !(tvmaze || kinema_meta::IMAGE_SIZES.contains(&size)) || !valid_image_name(name) {
        return Err(ApiError::bad_request("media.image_invalid"));
    }
    let dir = state.cache_dir("images").join(size);
    let path = dir.join(name);
    if !path.exists() {
        let bytes = if tvmaze {
            kinema_meta::tvmaze::fetch_image(&state.http, name).await
        } else {
            kinema_meta::fetch_image(&state.http, size, name).await
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
