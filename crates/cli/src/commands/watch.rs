use std::time::Duration as StdDuration;
use chrono::{DateTime, Local, Utc};
use colored::Colorize;
use tokio::time::sleep;

use forgex_core::{Activity, ForgeEvent, ForgeEventPayload, Result};
use forgex_scoring::ActivityClassifier;
use crate::app::AppContext;
use crate::ui::formatting::*;
use crate::watcher::{ActiveWindow, WindowObserver};

pub async fn execute_watch(
    ctx: &AppContext,
    interval_secs: u64,
    flush_secs: u64,
    quiet: bool,
) -> Result<()> {
    println!("{}", header("FORGEX LINUX ACTIVITY WATCHER (MVP 2)"));
    println!("{} Observing active window in background...", "👁️".cyan());
    println!("Sampling interval: {}s | Auto-flush interval: {}s", interval_secs, flush_secs);
    println!("Compositor: Hyprland / Wayland active window monitor");
    println!("Database:   {}", ctx.db_path.display().to_string().cyan());
    println!();
    println!("{}", "Press Ctrl+C to stop watching and flush remaining activity.".dimmed());
    println!();

    let mut current_window: Option<ActiveWindow> = None;
    let mut current_start: DateTime<Utc> = Utc::now();
    let mut accumulated_secs: u64 = 0;

    let flush_activity = |ctx: &AppContext, win: &ActiveWindow, start: DateTime<Utc>, secs: u64, quiet: bool| -> Result<()> {
        let mins = ((secs as f32 / 60.0).round() as u32).max(1);

        let category = ActivityClassifier::classify(&win.app_class, &win.title);
        let detail = if win.title.is_empty() { None } else { Some(win.title.clone()) };

        let act = Activity::new(
            "linux-laptop",
            &win.app_class,
            detail,
            category,
            start,
            mins,
        )?;

        ctx.db.insert_activity(&act)?;
        ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::ActivityLogged {
            activity_id: act.id,
            application: act.application.clone(),
            duration_mins: act.duration_mins,
        }))?;

        if !quiet {
            let cat_colored = match act.category {
                forgex_core::ActivityCategory::Productive => act.category.to_string().green(),
                forgex_core::ActivityCategory::Essential => act.category.to_string().cyan(),
                forgex_core::ActivityCategory::MediumEntertainment => act.category.to_string().yellow(),
                forgex_core::ActivityCategory::HighEntertainment => act.category.to_string().red().bold(),
                forgex_core::ActivityCategory::Unknown => act.category.to_string().dimmed(),
            };

            let title_snip = if act.domain_or_detail.as_deref().unwrap_or("").len() > 30 {
                format!("{}...", &act.domain_or_detail.as_deref().unwrap()[..27])
            } else {
                act.domain_or_detail.clone().unwrap_or_default()
            };

            println!(
                "[{}] {} {} ({}) -> {} ({})",
                Local::now().format("%H:%M:%S").to_string().dimmed(),
                "✓ Logged:".green(),
                act.application.bold(),
                title_snip.dimmed(),
                cat_colored,
                format_duration(act.duration_mins).bold()
            );
        }

        Ok(())
    };

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\n{}", "Stopping ForgeX watcher...".yellow());
                if let Some(ref win) = current_window {
                    if accumulated_secs >= flush_secs.min(10) {
                        let _ = flush_activity(ctx, win, current_start, accumulated_secs, quiet);
                    }
                }
                println!("{}", "ForgeX Activity Watcher stopped gracefully.".green());
                return Ok(());
            }
            _ = sleep(StdDuration::from_secs(interval_secs)) => {
                let active = WindowObserver::get_active_window();

                match (&current_window, &active) {
                    (Some(prev), Some(now)) if prev.app_class == now.app_class && prev.title == now.title => {
                        accumulated_secs += interval_secs;

                        // Auto-flush every flush_secs
                        if accumulated_secs >= flush_secs {
                            flush_activity(ctx, prev, current_start, accumulated_secs, quiet)?;
                            current_start = Utc::now();
                            accumulated_secs = 0;
                        }
                    }
                    (Some(prev), _) => {
                        // Window switched
                        if accumulated_secs >= flush_secs.min(10) {
                            flush_activity(ctx, prev, current_start, accumulated_secs, quiet)?;
                        }
                        current_window = active;
                        current_start = Utc::now();
                        accumulated_secs = interval_secs;
                    }
                    (None, Some(_)) => {
                        current_window = active;
                        current_start = Utc::now();
                        accumulated_secs = interval_secs;
                    }
                    (None, None) => {}
                }
            }
        }
    }
}
