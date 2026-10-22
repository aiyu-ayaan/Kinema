//! Live HLS sessions: one per profile and file, holding the segments cut so far and at most one
//! ffmpeg run. A seek inside what is cut is a file read; a seek just ahead waits for the run; only a
//! seek elsewhere restarts ffmpeg. Idle sessions live [hls::IDLE_TIMEOUT] so a quick seek back or a
//! paused player does not pay the startup again. Decisions are in `kinema_media::hls`.

use crate::error::{ApiError, ApiResult};
use crate::state::{AppState, Counted};
use kinema_media::hls::{self, Fetch, Produced};
use kinema_media::playback::{Encode, Track};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::process::Child;
use tracing::warn;

/// A segment not ready after this fails the request; the player retries it.
const SEGMENT_WAIT: Duration = Duration::from_secs(45);
/// How often the live runs are checked for throttling and idle sessions reaped.
const TICK: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub struct OwnedTrack {
    pub index: i64,
    pub codec: Option<String>,
    pub profile: Option<String>,
}

impl OwnedTrack {
    fn track(&self) -> Track<'_> {
        Track {
            index: self.index,
            codec: self.codec.as_deref(),
            profile: self.profile.as_deref(),
        }
    }
}

/// Everything a run needs, fixed for the session's life.
#[derive(Debug, Clone)]
pub struct Spec {
    pub path: PathBuf,
    pub video: Option<OwnedTrack>,
    pub audio: Option<OwnedTrack>,
    pub copy_video: bool,
    pub copy_audio: bool,
    pub preset: String,
    pub crf: u32,
    pub max_height: u32,
    pub threads: u32,
    pub hardware: bool,
    pub starts: Vec<f64>,
}

pub struct Session {
    user_id: i64,
    file_id: i64,
    dir: PathBuf,
    spec: Spec,
    produced: Produced,
    run: u32,
    child: Option<Child>,
    paused: bool,
    requested: usize,
    last_access: Instant,
    /// Stream and (for a re-encode) transcode counters, held while a run exists.
    slots: Option<(Counted, Option<Counted>)>,
}

impl Session {
    fn new(user_id: i64, file_id: i64, dir: PathBuf, spec: Spec, now: Instant) -> Self {
        Self {
            user_id,
            file_id,
            dir,
            produced: Produced::new(spec.starts.len()),
            spec,
            run: 0,
            child: None,
            paused: false,
            requested: 0,
            last_access: now,
            slots: None,
        }
    }

    fn reencode(&self) -> bool {
        !self.spec.copy_video && self.spec.video.is_some()
    }

    /// Picks up segments the run finished, and notices when it ended.
    async fn refresh(&mut self) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        let exited = !matches!(child.try_wait(), Ok(None));
        let list = self.dir.join(format!("list-{}.csv", self.run));
        if let Ok(csv) = tokio::fs::read_to_string(&list).await {
            for n in hls::listed_segments(&csv, self.run) {
                self.produced.complete(self.run, n);
            }
        }
        if exited {
            self.child = None;
            self.slots = None;
            self.paused = false;
            self.produced.cursor = None;
        }
    }

    /// Pauses the run once it is far enough ahead of the player, resumes it when the player nears.
    fn throttle(&mut self) {
        let want = self.child.is_some()
            && self
                .produced
                .should_pause(self.requested, &self.spec.starts);
        if want == self.paused {
            return;
        }
        if let Some(pid) = self.child.as_ref().and_then(Child::id)
            && kinema_core::platform::suspend_process(pid, want).is_ok()
        {
            self.paused = want;
        }
    }

    fn stop(&mut self) {
        // kill_on_drop: a paused (stopped) ffmpeg dies from SIGKILL too.
        self.child = None;
        self.slots = None;
        self.paused = false;
        self.produced.cursor = None;
    }

    async fn start_run(&mut self, state: &AppState, n: usize) -> ApiResult<()> {
        self.stop();
        let bins = state
            .bins
            .clone()
            .ok_or_else(|| ApiError::bad_request("media.ffmpeg_missing"))?;
        let transcode = if self.reencode() {
            let max = state.settings().max_transcodes as usize;
            if state
                .active_transcodes
                .load(std::sync::atomic::Ordering::SeqCst)
                >= max
            {
                return Err(ApiError::new(
                    axum::http::StatusCode::SERVICE_UNAVAILABLE,
                    "media.transcode_busy",
                ));
            }
            Some(Counted::new(&state.active_transcodes))
        } else {
            None
        };
        tokio::fs::create_dir_all(&self.dir).await?;
        self.run += 1;
        let spec = &self.spec;
        let child = hls::segment_command(
            &bins,
            &spec.path,
            &spec.starts,
            n,
            self.run,
            &self.dir,
            spec.video.as_ref().map(OwnedTrack::track),
            spec.audio.as_ref().map(OwnedTrack::track),
            spec.copy_video,
            spec.copy_audio,
            Encode {
                preset: &spec.preset,
                crf: spec.crf,
                max_height: spec.max_height,
                threads: spec.threads,
                hardware: spec.hardware,
            },
        )
        .spawn()?;
        self.child = Some(child);
        self.slots = Some((Counted::new(&state.active_streams), transcode));
        self.produced.cursor = Some(n);
        Ok(())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.child = None;
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

type Shared = Arc<tokio::sync::Mutex<Session>>;

#[derive(Default)]
struct Registry {
    sessions: HashMap<String, Shared>,
    ticking: bool,
}

#[derive(Clone, Default)]
pub struct HlsSessions(Arc<Mutex<Registry>>);

impl HlsSessions {
    /// The session for `key`, made from `spec` when new. Another session of the same profile and file
    /// (a different audio track) is dropped: the player moved on from it.
    pub async fn session(
        &self,
        state: &AppState,
        key: String,
        user_id: i64,
        file_id: i64,
        spec: impl AsyncFnOnce() -> ApiResult<Spec>,
    ) -> ApiResult<Shared> {
        if let Some(s) = self.get(&key) {
            return Ok(s);
        }
        let spec = spec().await?;
        let dir = state.cache_dir("hls").join(&key);
        let new = Arc::new(tokio::sync::Mutex::new(Session::new(
            user_id,
            file_id,
            dir,
            spec,
            Instant::now(),
        )));
        let start_ticker = {
            let Ok(mut reg) = self.0.lock() else {
                return Err(ApiError::internal());
            };
            if let Some(s) = reg.sessions.get(&key) {
                return Ok(s.clone());
            }
            reg.sessions.retain(|_, s| {
                s.try_lock()
                    .map_or(true, |s| s.user_id != user_id || s.file_id != file_id)
            });
            reg.sessions.insert(key, new.clone());
            !std::mem::replace(&mut reg.ticking, true)
        };
        if start_ticker {
            tokio::spawn(self.clone().tick());
        }
        Ok(new)
    }

    fn get(&self, key: &str) -> Option<Shared> {
        self.0.lock().ok()?.sessions.get(key).cloned()
    }

    /// Path of segment `n`, once on disk: from an earlier run, the live one, or a new run from `n`.
    pub async fn segment(
        &self,
        state: &AppState,
        session: &Shared,
        n: usize,
    ) -> ApiResult<PathBuf> {
        let deadline = Instant::now() + SEGMENT_WAIT;
        let mut restarted = false;
        loop {
            {
                let mut s = session.lock().await;
                if n >= s.spec.starts.len() {
                    return Err(ApiError::not_found());
                }
                s.last_access = Instant::now();
                s.requested = n;
                s.refresh().await;
                let starts = s.spec.starts.clone();
                for (i, run) in s.produced.prune(n, &starts) {
                    let _ = tokio::fs::remove_file(s.dir.join(format!("{run}-{i}.ts"))).await;
                }
                match s.produced.fetch(n, &starts) {
                    Fetch::Ready(run) => {
                        s.throttle();
                        return Ok(s.dir.join(format!("{run}-{n}.ts")));
                    }
                    Fetch::Wait => s.throttle(),
                    // A run started for this request that died without the segment will not do better.
                    Fetch::Restart if restarted => {
                        warn!("HLS run for file {} ended before segment {n}", s.file_id);
                        return Err(ApiError::internal());
                    }
                    Fetch::Restart => {
                        s.start_run(state, n).await?;
                        restarted = true;
                    }
                }
            }
            if Instant::now() >= deadline {
                return Err(ApiError::internal());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Removes sessions idle at `now`; they tear down (ffmpeg killed, files deleted) when dropped.
    /// A session in use right now is not idle.
    fn reap(&self, now: Instant) -> Vec<Shared> {
        let Ok(mut reg) = self.0.lock() else {
            return Vec::new();
        };
        let idle: Vec<String> = reg
            .sessions
            .iter()
            .filter(|(_, s)| s.try_lock().is_ok_and(|s| hls::is_idle(s.last_access, now)))
            .map(|(k, _)| k.clone())
            .collect();
        idle.iter().filter_map(|k| reg.sessions.remove(k)).collect()
    }

    /// Runs while any session exists: throttles live runs between requests, reaps idle sessions.
    async fn tick(self) {
        loop {
            tokio::time::sleep(TICK).await;
            drop(self.reap(Instant::now()));
            let live: Vec<Shared> = {
                let Ok(mut reg) = self.0.lock() else {
                    return;
                };
                if reg.sessions.is_empty() {
                    reg.ticking = false;
                    return;
                }
                reg.sessions.values().cloned().collect()
            };
            for s in live {
                let mut s = s.lock().await;
                s.refresh().await;
                s.throttle();
            }
        }
    }
}

/// Deletes segments left by a previous server process.
pub fn purge(state: &AppState) {
    let _ = std::fs::remove_dir_all(state.cache_dir("hls"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> Spec {
        Spec {
            path: "a.mkv".into(),
            video: None,
            audio: None,
            copy_video: true,
            copy_audio: true,
            preset: "veryfast".into(),
            crf: 23,
            max_height: 0,
            threads: 0,
            hardware: false,
            starts: vec![0.0, 4.0],
        }
    }

    #[test]
    fn idle_sessions_are_reaped_and_their_files_removed() {
        let sessions = HlsSessions::default();
        let t0 = Instant::now();
        let root = std::env::temp_dir().join(format!("kinema-hls-{}", std::process::id()));
        let add = |key: &str, last: Instant| {
            let dir = root.join(key);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("1-0.ts"), b"x").unwrap();
            let s = Session::new(1, 1, dir, spec(), last);
            if let Ok(mut reg) = sessions.0.lock() {
                reg.sessions
                    .insert(key.into(), Arc::new(tokio::sync::Mutex::new(s)));
            }
        };
        add("old", t0);
        add("fresh", t0 + Duration::from_secs(30));
        // 59 s after the old one's last request: both kept.
        assert!(sessions.reap(t0 + Duration::from_secs(59)).is_empty());
        let gone = sessions.reap(t0 + Duration::from_secs(60));
        assert_eq!(gone.len(), 1);
        drop(gone);
        assert!(!root.join("old").exists());
        assert!(root.join("fresh").exists());
        assert!(sessions.get("fresh").is_some());
        assert!(sessions.get("old").is_none());
        // A session busy serving a request is never reaped.
        let busy = sessions.get("fresh").unwrap();
        let guard = busy.try_lock().unwrap();
        assert!(sessions.reap(t0 + Duration::from_secs(600)).is_empty());
        drop(guard);
        drop(busy);
        assert_eq!(sessions.reap(t0 + Duration::from_secs(600)).len(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }
}
