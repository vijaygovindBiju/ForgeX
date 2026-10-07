pub mod observer;

pub use observer::{ActiveWindow, WindowObserver};

use std::sync::{Arc, RwLock};
use std::time::Duration as StdDuration;
use chrono::{DateTime, Utc};
use forgex_core::{Activity, ActivityCategory, ForgeEvent, ForgeEventPayload};
use forgex_scoring::ActivityClassifier;
use forgex_storage::ForgeDb;

/// Live tracking state shared between the background watcher and the TUI presentation layer.
#[derive(Debug, Clone)]
pub struct LiveWatcherInfo {
    pub app_class: String,
    pub title: String,
    pub category: ActivityCategory,
    pub session_secs: u64,
}

/// Runs a lightweight live window watcher in the background while the ForgeX application is running.
/// It continuously tracks focused windows (Hyprland / Wayland / X11), classifies them,
/// updates `LiveWatcherInfo` in real time, and persists activity chunks to SQLite.
pub async fn run_live_watcher(
    db: ForgeDb,
    live_info: Arc<RwLock<Option<LiveWatcherInfo>>>,
    mut stop_rx: tokio::sync::oneshot::Receiver<()>,
) {
    let interval_secs: u64 = 2; // Sample every 2 seconds
    let flush_threshold_secs: u64 = 60; // Auto-flush every 60s of continuous activity

    let mut current_window: Option<ActiveWindow> = None;
    let mut current_start: DateTime<Utc> = Utc::now();
    let mut accumulated_secs: u64 = 0;

    let flush_activity = |db: &ForgeDb, win: &ActiveWindow, start: DateTime<Utc>, secs: u64| {
        let mins = ((secs as f32) / 60.0).round().max(1.0) as u32;
        let category = ActivityClassifier::classify(&win.app_class, &win.title);
        let detail = if win.title.is_empty() { None } else { Some(win.title.clone()) };

        if let Ok(act) = Activity::new(
            "linux-desktop",
            &win.app_class,
            detail,
            category,
            start,
            mins,
        ) {
            let _ = db.insert_activity(&act);
            let _ = db.log_event(&ForgeEvent::new(ForgeEventPayload::ActivityLogged {
                activity_id: act.id,
                application: act.application.clone(),
                duration_mins: act.duration_mins,
            }));
        }
    };

    loop {
        tokio::select! {
            _ = &mut stop_rx => {
                // Application exit: flush remaining activity if active for >= 10s
                if let Some(ref win) = current_window {
                    if accumulated_secs >= 10 {
                        flush_activity(&db, win, current_start, accumulated_secs);
                    }
                }
                break;
            }
            _ = tokio::time::sleep(StdDuration::from_secs(interval_secs)) => {
                let active = WindowObserver::get_active_window();

                match (&current_window, &active) {
                    (Some(prev), Some(now)) if prev.app_class == now.app_class && prev.title == now.title => {
                        accumulated_secs += interval_secs;

                        let cat = ActivityClassifier::classify(&now.app_class, &now.title);
                        if let Ok(mut info) = live_info.write() {
                            *info = Some(LiveWatcherInfo {
                                app_class: now.app_class.clone(),
                                title: now.title.clone(),
                                category: cat,
                                session_secs: accumulated_secs,
                            });
                        }

                        // Auto-flush every minute of continuous activity
                        if accumulated_secs >= flush_threshold_secs {
                            flush_activity(&db, prev, current_start, accumulated_secs);
                            current_start = Utc::now();
                            accumulated_secs = 0;
                        }
                    }
                    (Some(prev), _) => {
                        // Window switched or title changed
                        if accumulated_secs >= 10 {
                            flush_activity(&db, prev, current_start, accumulated_secs);
                        }

                        if let Some(ref now) = active {
                            let cat = ActivityClassifier::classify(&now.app_class, &now.title);
                            if let Ok(mut info) = live_info.write() {
                                *info = Some(LiveWatcherInfo {
                                    app_class: now.app_class.clone(),
                                    title: now.title.clone(),
                                    category: cat,
                                    session_secs: interval_secs,
                                });
                            }
                        } else if let Ok(mut info) = live_info.write() {
                            *info = None;
                        }

                        current_window = active;
                        current_start = Utc::now();
                        accumulated_secs = interval_secs;
                    }
                    (None, Some(now)) => {
                        let cat = ActivityClassifier::classify(&now.app_class, &now.title);
                        if let Ok(mut info) = live_info.write() {
                            *info = Some(LiveWatcherInfo {
                                app_class: now.app_class.clone(),
                                title: now.title.clone(),
                                category: cat,
                                session_secs: interval_secs,
                            });
                        }

                        current_window = active;
                        current_start = Utc::now();
                        accumulated_secs = interval_secs;
                    }
                    (None, None) => {
                        if let Ok(mut info) = live_info.write() {
                            *info = None;
                        }
                    }
                }
            }
        }
    }
}
