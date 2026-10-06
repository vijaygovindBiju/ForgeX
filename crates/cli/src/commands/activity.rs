use chrono::{Duration, Local, Utc};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color, Table};

use forgex_core::{Activity, ActivityCategory, ForgeEvent, ForgeEventPayload, Result};
use crate::app::AppContext;
use crate::commands::task::parse_schedule_time;
use crate::ui::formatting::*;

pub fn execute_activity_log(
    ctx: &AppContext,
    application: &str,
    domain_or_detail: Option<&str>,
    duration_mins: u32,
    category_str: &str,
    started_at_str: Option<&str>,
) -> Result<()> {
    let category = category_str.parse::<ActivityCategory>()?;
    let started_at = match started_at_str {
        Some(s) => parse_schedule_time(s)?,
        None => Utc::now() - Duration::minutes(duration_mins as i64),
    };

    let activity = Activity::new(
        "linux-local",
        application,
        domain_or_detail.map(|s| s.to_string()),
        category,
        started_at,
        duration_mins,
    )?;

    ctx.db.insert_activity(&activity)?;
    ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::ActivityLogged {
        activity_id: activity.id,
        application: activity.application.clone(),
        duration_mins: activity.duration_mins,
    }))?;

    let cat_color = match activity.category {
        ActivityCategory::Productive => Color::Green,
        ActivityCategory::Essential => Color::Cyan,
        ActivityCategory::MediumEntertainment => Color::Yellow,
        ActivityCategory::HighEntertainment => Color::Red,
        ActivityCategory::Unknown => Color::DarkGrey,
    };

    println!(
        "{} Activity logged: {} ({}) for {}",
        success_icon(),
        activity.application.bold(),
        activity.category.to_string().color(format!("{:?}", cat_color).to_lowercase()),
        format_duration(activity.duration_mins).bold()
    );

    Ok(())
}

pub fn execute_activity_list(ctx: &AppContext) -> Result<()> {
    let activities = ctx.db.list_activities()?;
    if activities.is_empty() {
        println!("{}", "No activities logged yet.".dimmed());
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec![
            Cell::new("Time").fg(Color::Yellow),
            Cell::new("Application").fg(Color::White),
            Cell::new("Detail / Domain").fg(Color::Cyan),
            Cell::new("Category").fg(Color::Magenta),
            Cell::new("Duration").fg(Color::White),
        ]);

    for a in activities {
        let cat_color = match a.category {
            ActivityCategory::Productive => Color::Green,
            ActivityCategory::Essential => Color::Cyan,
            ActivityCategory::MediumEntertainment => Color::Yellow,
            ActivityCategory::HighEntertainment => Color::Red,
            ActivityCategory::Unknown => Color::DarkGrey,
        };

        table.add_row(vec![
            Cell::new(a.started_at.with_timezone(&Local).format("%m-%d %H:%M").to_string()),
            Cell::new(&a.application),
            Cell::new(a.domain_or_detail.as_deref().unwrap_or("-")),
            Cell::new(a.category.to_string()).fg(cat_color),
            Cell::new(format_duration(a.duration_mins)),
        ]);
    }

    println!("{table}");
    Ok(())
}
