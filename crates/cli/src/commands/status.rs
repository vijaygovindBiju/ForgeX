use chrono::{Local, Utc};
use colored::Colorize;
use serde_json::json;

use forgex_core::{CommitmentStatus, Result};
use crate::app::AppContext;
use crate::ui::formatting::*;

pub fn execute_status(ctx: &AppContext, as_json: bool) -> Result<()> {
    let all_commitments = ctx.db.list_commitments()?;
    let all_activities = ctx.db.list_activities()?;
    let active_penalties = ctx.db.get_active_penalties()?;
    let available_save_days = ctx.db.get_available_save_days()?;
    let overdue_commitments = ctx.check_overdue_commitments()?;

    let today_local = Local::now().date_naive();

    // Filter commitments for today
    let today_commitments: Vec<_> = all_commitments
        .iter()
        .filter(|c| c.scheduled_start.with_timezone(&Local).date_naive() == today_local)
        .collect();

    // Entertainment logged today
    let today_entertainment_mins: u32 = all_activities
        .iter()
        .filter(|a| {
            a.started_at.with_timezone(&Local).date_naive() == today_local
                && a.category.is_entertainment()
        })
        .map(|a| a.duration_mins)
        .sum();

    // Highest current skip streak
    let max_consecutive_skips = all_commitments
        .iter()
        .map(|c| c.consecutive_skips)
        .max()
        .unwrap_or(0);

    // Latest or average avoidance score
    let recent_penalties = ctx.db.list_penalties()?;
    let latest_avoidance_score = recent_penalties
        .first()
        .map(|p| p.avoidance_score)
        .unwrap_or(0);

    // Total active penalty
    let total_remaining_penalty_mins: u32 = active_penalties
        .iter()
        .map(|p| p.remaining_penalty_mins)
        .sum();
    let max_restricted_days: u32 = active_penalties
        .iter()
        .map(|p| p.restricted_days)
        .max()
        .unwrap_or(0);

    // Weekly consistency (last 7 days completed commitments)
    let completed_this_week = all_commitments
        .iter()
        .filter(|c| {
            let days_ago = (Utc::now() - c.scheduled_start).num_days();
            (0..=7).contains(&days_ago) && c.status == CommitmentStatus::Completed
        })
        .count();
    let scheduled_this_week = all_commitments
        .iter()
        .filter(|c| {
            let days_ago = (Utc::now() - c.scheduled_start).num_days();
            (0..=7).contains(&days_ago)
        })
        .count();

    let consistency_ratio = if scheduled_this_week > 0 {
        (completed_this_week as f32 / scheduled_this_week as f32).min(1.0)
    } else {
        1.0
    };
    let consistency_days_approx = (consistency_ratio * 7.0).round() as u32;

    if as_json {
        let output = json!({
            "today_commitments": today_commitments,
            "skip_streak": max_consecutive_skips,
            "today_entertainment_mins": today_entertainment_mins,
            "avoidance_score": latest_avoidance_score,
            "penalty": {
                "remaining_mins": total_remaining_penalty_mins,
                "restricted_days": max_restricted_days,
            },
            "save_days": {
                "available": available_save_days.len(),
                "weekly_consistency": format!("{}/7", consistency_days_approx),
            },
            "overdue_count": overdue_commitments.len()
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return Ok(());
    }

    // Display Warnings if any tasks are overdue
    if !overdue_commitments.is_empty() {
        println!(
            "\n{} {} commitment(s) are past due! Run {} to review.",
            "⚠️".yellow().bold(),
            overdue_commitments.len().to_string().bold().red(),
            "forgex evaluate".bold().white()
        );
    }

    // Header & Today's tasks
    println!("{}", header("TODAY"));
    if today_commitments.is_empty() {
        println!("{}", "No commitments scheduled for today.".dimmed());
        println!("Add one with: {}", "forgex task add --title <TITLE> --duration <MINS>".dimmed());
    } else {
        for c in today_commitments {
            let icon = match c.status {
                CommitmentStatus::Completed => success_icon(),
                CommitmentStatus::Missed => missed_icon(),
                CommitmentStatus::Active => active_icon(),
                CommitmentStatus::Planned => planned_icon(),
                CommitmentStatus::Excused => excused_icon(),
            };

            let id_short = &c.id.to_string()[..8];
            let title_padded = format!("{:<28}", c.title);
            let duration_str = format_duration(c.scheduled_duration_mins);

            let status_note = match c.status {
                CommitmentStatus::Active => " (Active)".yellow().bold().to_string(),
                CommitmentStatus::Missed => {
                    if c.consecutive_skips > 1 {
                        format!(" (Skipped x{})", c.consecutive_skips).red().to_string()
                    } else {
                        " (Missed)".red().to_string()
                    }
                }
                _ => "".to_string(),
            };

            println!(
                "{} {:<28} {:>6}  [{}] {}",
                icon,
                title_padded,
                duration_str.bold(),
                id_short.dimmed(),
                status_note
            );
        }
    }

    // Behavior section
    println!("{}", header("BEHAVIOR"));
    let skip_display = if max_consecutive_skips > 0 {
        max_consecutive_skips.to_string().red().bold().to_string()
    } else {
        "0".green().to_string()
    };
    println!("{:<24} {}", "Skip streak:", skip_display);
    println!(
        "{:<24} {}",
        "Entertainment:",
        format_duration(today_entertainment_mins).yellow().bold()
    );
    let avoidance_color = if latest_avoidance_score > 50 {
        latest_avoidance_score.to_string().red().bold()
    } else if latest_avoidance_score > 0 {
        latest_avoidance_score.to_string().yellow()
    } else {
        "0".green()
    };
    println!("{:<24} {}", "Avoidance score:", avoidance_color);

    // Penalty section
    println!("{}", header("PENALTY"));
    if total_remaining_penalty_mins > 0 || max_restricted_days > 0 {
        println!(
            "{:<24} {}",
            "Remaining:",
            format_duration(total_remaining_penalty_mins).red().bold()
        );
        println!(
            "{:<24} {}",
            "Restricted days:",
            max_restricted_days.to_string().red().bold()
        );
        println!(
            "{}",
            "Tip: Complete commitments to earn recovery reductions!".italic().dimmed()
        );
    } else {
        println!("{:<24} {}", "Remaining:", "0m (No restrictions)".green());
        println!("{:<24} {}", "Restricted days:", "0".green());
    }

    // Save Days section
    println!("{}", header("SAVE DAYS"));
    let save_color = if available_save_days.is_empty() {
        "0".yellow()
    } else {
        available_save_days.len().to_string().green().bold()
    };
    println!("{:<24} {}", "Available:", save_color);
    println!(
        "{:<24} {}/7 {}",
        "Weekly consistency:",
        consistency_days_approx.to_string().bold(),
        progress_bar(consistency_days_approx, 7, 10)
    );

    println!();
    Ok(())
}
