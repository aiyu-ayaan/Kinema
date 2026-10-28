//! Keyframe index job: after each scan, indexes every probed video once per file version, so seeks
//! and HLS cuts never run ffprobe on the play path. Shares the one background media slot.

use crate::state::AppState;
use kinema_db::keyframes;
use kinema_media::keyframes::KeyframeIndex;
use tracing::{debug, warn};

pub fn spawn(state: AppState, library_id: i64) {
    if state.bins.is_none() {
        return;
    }
    let jobs = state.keyframe_jobs.clone();
    jobs.spawn(library_id, move || run(state.clone(), library_id));
}

async fn run(state: AppState, library_id: i64) -> anyhow::Result<()> {
    let Some(bins) = state.bins.clone() else {
        return Ok(());
    };
    for (id, path, size) in keyframes::missing(&state.db, library_id).await? {
        let _permit = state.background_slot(&state.thumb_slots).await?;
        let built = kinema_media::keyframes::build(
            &bins,
            std::path::Path::new(&path),
            u64::try_from(size).unwrap_or(u64::MAX),
        )
        .await;
        let saved = match built {
            Ok(Some(index)) => {
                debug!(
                    "{} keyframes from {} in {path}",
                    index.times.len(),
                    index.source
                );
                keyframes::save(
                    &state.db,
                    id,
                    index.source,
                    index.has_b_frames,
                    &index.encode_times(),
                )
                .await
            }
            Ok(None) => keyframes::save(&state.db, id, "none", false, &[]).await,
            Err(e) => {
                // Unreadable now (unmounted share): left missing, retried after the next scan.
                warn!("keyframe index of {path}: {e}");
                continue;
            }
        };
        if let Err(e) = saved {
            warn!("saving keyframe index of {path}: {e}");
        }
    }
    Ok(())
}

/// The stored index of a file's current version, if there is one.
pub async fn load(state: &AppState, file_id: i64) -> Option<KeyframeIndex> {
    let (has_b_frames, blob) = keyframes::get(&state.db, file_id).await.ok()??;
    Some(KeyframeIndex {
        source: "db",
        has_b_frames,
        times: KeyframeIndex::decode_times(&blob),
    })
}
