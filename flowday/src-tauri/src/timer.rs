use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::sync::{watch, Mutex};
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimerStatus {
    Idle,
    Running,
    Paused,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub block_id: Option<String>,
    pub status: TimerStatus,
    pub duration_secs: u64,
    pub remaining_secs: u64,
    pub elapsed_secs: u64,
}

impl TimerState {
    fn idle() -> Self {
        Self {
            block_id: None,
            status: TimerStatus::Idle,
            duration_secs: 0,
            remaining_secs: 0,
            elapsed_secs: 0,
        }
    }
}

/// Payload emitted on every tick via the 'timer-tick' Tauri event.
#[derive(Clone, Serialize)]
struct TickPayload {
    remaining_secs: u64,
    elapsed_secs: u64,
    status: TimerStatus,
    block_id: Option<String>,
}

/// Shared timer state protected by a mutex.
/// The `cancel_tx` is used to signal the background tick task to stop.
struct TimerInner {
    state: TimerState,
    /// Sends `true` to tell the background task to stop.
    cancel_tx: Option<watch::Sender<bool>>,
    /// Wall-clock instant when the timer last (re)started ticking,
    /// used to keep sub-second accuracy across pause/resume cycles.
    tick_origin: Option<Instant>,
}

pub struct Timer {
    inner: Arc<Mutex<TimerInner>>,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(TimerInner {
                state: TimerState::idle(),
                cancel_tx: None,
                tick_origin: None,
            })),
        }
    }

    /// Start a new timer for `block_id` with the given duration.
    /// If a timer is already running it will be stopped first.
    pub async fn start(&self, app: AppHandle, block_id: String, duration_secs: u64) -> TimerState {
        // Stop any existing timer task
        self.cancel_background().await;

        let (cancel_tx, cancel_rx) = watch::channel(false);

        {
            let mut inner = self.inner.lock().await;
            inner.state = TimerState {
                block_id: Some(block_id),
                status: TimerStatus::Running,
                duration_secs,
                remaining_secs: duration_secs,
                elapsed_secs: 0,
            };
            inner.cancel_tx = Some(cancel_tx);
            inner.tick_origin = Some(Instant::now());
        }

        self.spawn_tick_task(app, cancel_rx);
        self.inner.lock().await.state.clone()
    }

    pub async fn pause(&self) -> TimerState {
        self.cancel_background().await;

        let mut inner = self.inner.lock().await;
        if inner.state.status == TimerStatus::Running {
            // Reconcile any fractional second before pausing
            if let Some(origin) = inner.tick_origin.take() {
                let sub_elapsed = origin.elapsed().as_secs();
                if sub_elapsed > 0 && inner.state.remaining_secs >= sub_elapsed {
                    inner.state.remaining_secs -= sub_elapsed;
                    inner.state.elapsed_secs += sub_elapsed;
                }
            }
            inner.state.status = TimerStatus::Paused;
        }
        inner.state.clone()
    }

    pub async fn resume(&self, app: AppHandle) -> TimerState {
        let should_spawn = {
            let mut inner = self.inner.lock().await;
            if inner.state.status == TimerStatus::Paused {
                inner.state.status = TimerStatus::Running;
                inner.tick_origin = Some(Instant::now());
                let (cancel_tx, _) = watch::channel(false);
                inner.cancel_tx = Some(cancel_tx);
                true
            } else {
                false
            }
        };

        if should_spawn {
            let cancel_rx = {
                let inner = self.inner.lock().await;
                inner.cancel_tx.as_ref().unwrap().subscribe()
            };
            self.spawn_tick_task(app, cancel_rx);
        }

        self.inner.lock().await.state.clone()
    }

    pub async fn stop(&self) -> TimerState {
        self.cancel_background().await;

        let mut inner = self.inner.lock().await;
        inner.state.status = TimerStatus::Idle;
        inner.state.remaining_secs = 0;
        inner.state.elapsed_secs = 0;
        inner.state.duration_secs = 0;
        inner.state.block_id = None;
        inner.tick_origin = None;
        inner.state.clone()
    }

    pub async fn extend(&self, extra_secs: u64) -> TimerState {
        let mut inner = self.inner.lock().await;
        inner.state.duration_secs += extra_secs;
        inner.state.remaining_secs += extra_secs;
        inner.state.clone()
    }

    pub async fn get_state(&self) -> TimerState {
        self.inner.lock().await.state.clone()
    }

    // ── internal helpers ──

    async fn cancel_background(&self) {
        let mut inner = self.inner.lock().await;
        if let Some(tx) = inner.cancel_tx.take() {
            let _ = tx.send(true);
        }
    }

    fn spawn_tick_task(&self, app: AppHandle, mut cancel_rx: watch::Receiver<bool>) {
        let inner = Arc::clone(&self.inner);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(1));
            // consume the initial immediate tick
            ticker.tick().await;

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let mut guard = inner.lock().await;
                        if guard.state.status != TimerStatus::Running {
                            break;
                        }

                        if guard.state.remaining_secs == 0 {
                            guard.state.status = TimerStatus::Completed;
                            let payload = TickPayload {
                                remaining_secs: 0,
                                elapsed_secs: guard.state.elapsed_secs,
                                status: TimerStatus::Completed,
                                block_id: guard.state.block_id.clone(),
                            };
                            let _ = app.emit("timer-tick", &payload);
                            break;
                        }

                        guard.state.remaining_secs -= 1;
                        guard.state.elapsed_secs += 1;
                        guard.tick_origin = Some(Instant::now());

                        let payload = TickPayload {
                            remaining_secs: guard.state.remaining_secs,
                            elapsed_secs: guard.state.elapsed_secs,
                            status: guard.state.status,
                            block_id: guard.state.block_id.clone(),
                        };
                        let _ = app.emit("timer-tick", &payload);
                    }
                    _ = cancel_rx.changed() => {
                        break;
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_state() {
        let state = TimerState::idle();
        assert_eq!(state.status, TimerStatus::Idle);
        assert_eq!(state.remaining_secs, 0);
        assert_eq!(state.elapsed_secs, 0);
        assert!(state.block_id.is_none());
    }

    #[tokio::test]
    async fn test_timer_start_sets_state() {
        // We can test the Timer struct without a real Tauri app
        // by only testing state transitions that don't need emit.
        let timer = Timer::new();

        // Verify initial state
        let state = timer.get_state().await;
        assert_eq!(state.status, TimerStatus::Idle);

        // We can't call start/resume without AppHandle in unit tests,
        // but we can test pause, stop, extend, get_state on manually set state.
        {
            let mut inner = timer.inner.lock().await;
            inner.state = TimerState {
                block_id: Some("test-block".into()),
                status: TimerStatus::Running,
                duration_secs: 300,
                remaining_secs: 300,
                elapsed_secs: 0,
            };
        }

        let state = timer.get_state().await;
        assert_eq!(state.status, TimerStatus::Running);
        assert_eq!(state.remaining_secs, 300);
        assert_eq!(state.block_id.as_deref(), Some("test-block"));
    }

    #[tokio::test]
    async fn test_timer_pause() {
        let timer = Timer::new();
        {
            let mut inner = timer.inner.lock().await;
            inner.state = TimerState {
                block_id: Some("b1".into()),
                status: TimerStatus::Running,
                duration_secs: 60,
                remaining_secs: 45,
                elapsed_secs: 15,
            };
        }

        let state = timer.pause().await;
        assert_eq!(state.status, TimerStatus::Paused);
    }

    #[tokio::test]
    async fn test_timer_stop() {
        let timer = Timer::new();
        {
            let mut inner = timer.inner.lock().await;
            inner.state = TimerState {
                block_id: Some("b1".into()),
                status: TimerStatus::Running,
                duration_secs: 60,
                remaining_secs: 30,
                elapsed_secs: 30,
            };
        }

        let state = timer.stop().await;
        assert_eq!(state.status, TimerStatus::Idle);
        assert_eq!(state.remaining_secs, 0);
        assert!(state.block_id.is_none());
    }

    #[tokio::test]
    async fn test_timer_extend() {
        let timer = Timer::new();
        {
            let mut inner = timer.inner.lock().await;
            inner.state = TimerState {
                block_id: Some("b1".into()),
                status: TimerStatus::Running,
                duration_secs: 60,
                remaining_secs: 10,
                elapsed_secs: 50,
            };
        }

        let state = timer.extend(30).await;
        assert_eq!(state.duration_secs, 90);
        assert_eq!(state.remaining_secs, 40);
    }

    #[tokio::test]
    async fn test_pause_only_when_running() {
        let timer = Timer::new();
        // Pausing an idle timer should remain idle
        let state = timer.pause().await;
        assert_eq!(state.status, TimerStatus::Idle);
    }
}
