use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use tracing::warn;

/// Per-library background jobs (scans, thumbnails): one run at a time per library,
/// and a request during a run queues exactly one more run.
#[derive(Clone, Default)]
pub struct Jobs {
    name: &'static str,
    running: Arc<Mutex<HashMap<i64, bool>>>,
}

impl Jobs {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            running: Arc::default(),
        }
    }

    pub fn is_running(&self, id: i64) -> bool {
        self.running.lock().is_ok_and(|r| r.contains_key(&id))
    }

    pub fn spawn<F, Fut>(&self, id: i64, job: F)
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send,
    {
        {
            let Ok(mut running) = self.running.lock() else {
                return;
            };
            if let Some(rerun) = running.get_mut(&id) {
                *rerun = true;
                return;
            }
            running.insert(id, false);
        }
        let jobs = self.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = job().await {
                    warn!("{} for library {id} failed: {e}", jobs.name);
                }
                let Ok(mut running) = jobs.running.lock() else {
                    return;
                };
                if running.get(&id) == Some(&true) {
                    running.insert(id, false);
                } else {
                    running.remove(&id);
                    return;
                }
            }
        });
    }
}
