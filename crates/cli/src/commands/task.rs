use chrono::{DateTime, Duration, Local, NaiveTime, TimeZone, Utc};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};

use forgex_core::{
    Commitment, CommitmentStatus, ForgeEvent, ForgeEventPayload,
    RecurrenceRule, Result, SaveDay,
};
use forgex_scoring::{
    AvoidanceAnalyzer, PenaltyCalculator, RecoveryEngine,
};
use crate::app::AppContext;
use crate::ui::formatting::*;

pub fn parse_schedule_time(s: &str) -> Result<DateTime<Utc>> {
    let s = s.trim();

    // 1. Try parsing "HH:MM" for today
    if let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M") {
        let local_today = Local::now().date_naive();
        let local_dt = local_today.and_time(time);
        if let Some(dt) = Local.from_local_datetime(&local_dt).single() {
            return Ok(dt.with_timezone(&Utc));
        }
    }

    // 2. Try parsing relative "+Xm" or "+Xh"
    if s.starts_with('+') {
        let trimmed = &s[1..];
        if let Some(m) = trimmed.strip_suffix('m') {
            if let Ok(mins) = m.parse::<i64>() {
                return Ok(Utc::now() + Duration::minutes(mins));
            }
        } else if let Some(h) = trimmed.strip_suffix('h') {
            if let Ok(hours) = h.parse::<i64>() {
                return Ok(Utc::now() + Duration::hours(hours));
            }
        }
    }

    // 3. Try RFC3339 / ISO
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Default: now
    Ok(Utc::now())
}

pub fn execute_task_add(
    ctx: &AppContext,
    title: &str,
    start_str: Option<&str>,
    duration: u32,
    importance: u8,
    effort: u8,
    category: &str,
    recurrence_str: Option<&str>,
    description: Option<&str>,
) -> Result<()> {
    let scheduled_start = match start_str {
        Some(s) => parse_schedule_time(s)?,
        None => Utc::now(),
    };

    let recurrence = match recurrence_str.unwrap_or("daily").to_lowercase().as_str() {
        "once" => RecurrenceRule::Once,
        "weekdays" => RecurrenceRule::Weekdays,
        _ => RecurrenceRule::Daily,
    };

    let mut commitment = Commitment::new(
        title,
        importance,
        effort,
        scheduled_start,
        duration,
        recurrence,
        category,
    )?;

    if let Some(desc) = description {
        commitment.description = Some(desc.to_string());
    }

    ctx.db.insert_commitment(&commitment)?;
    ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::TaskCreated {
        commitment_id: commitment.id,
        title: commitment.title.clone(),
    }))?;

    let short_id = &commitment.id.to_string()[..8];
    println!(
        "{} Commitment created [{}]: {} (Importance: {}/5, Effort: {}/5)",
        success_icon(),
        short_id.cyan(),
        commitment.title.bold(),
        commitment.importance,
        commitment.expected_effort
    );
    println!(
        "  Scheduled: {}",
        commitment.scheduled_start.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string().yellow()
    );

    Ok(())
}

pub fn execute_task_list(ctx: &AppContext, filter_status: Option<&str>) -> Result<()> {
    let commitments = ctx.db.list_commitments()?;

    let filtered: Vec<_> = commitments
        .into_iter()
        .filter(|c| {
            if let Some(st) = filter_status {
                c.status.to_string().eq_ignore_ascii_case(st)
            } else {
                true
            }
        })
        .collect();

    if filtered.is_empty() {
        println!("{}", "No commitments found.".dimmed());
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("ID").fg(Color::Cyan),
            Cell::new("Status").fg(Color::White),
            Cell::new("Title").fg(Color::White),
            Cell::new("Scheduled").fg(Color::Yellow),
            Cell::new("Duration").fg(Color::White),
            Cell::new("Imp/Eff").fg(Color::Magenta),
            Cell::new("Category").fg(Color::Blue),
        ]);

    for c in filtered {
        let status_cell = match c.status {
            CommitmentStatus::Completed => Cell::new(format!("{} Completed", success_icon())).fg(Color::Green),
            CommitmentStatus::Missed => Cell::new(format!("{} Missed", missed_icon())).fg(Color::Red),
            CommitmentStatus::Active => Cell::new(format!("{} Active", active_icon())).fg(Color::Yellow),
            CommitmentStatus::Planned => Cell::new(format!("{} Planned", planned_icon())).fg(Color::White),
            CommitmentStatus::Excused => Cell::new(format!("{} Excused", excused_icon())).fg(Color::DarkGrey),
        };

        table.add_row(vec![
            Cell::new(&c.id.to_string()[..8]),
            status_cell,
            Cell::new(&c.title),
            Cell::new(c.scheduled_start.with_timezone(&Local).format("%m-%d %H:%M").to_string()),
            Cell::new(format_duration(c.scheduled_duration_mins)),
            Cell::new(format!("{}/{}", c.importance, c.expected_effort)),
            Cell::new(&c.category),
        ]);
    }

    println!("{table}");
    Ok(())
}

pub fn execute_task_start(ctx: &AppContext, id_prefix: &str) -> Result<()> {
    let mut commitment = ctx.find_commitment_by_prefix(id_prefix)?;
    let now = Utc::now();

    commitment.start(now)?;
    ctx.db.update_commitment(&commitment)?;
    ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::TaskStarted {
        commitment_id: commitment.id,
    }))?;

    println!(
        "{} Commitment [{}] started: {}",
        active_icon(),
        &commitment.id.to_string()[..8].cyan(),
        commitment.title.bold().yellow()
    );
    Ok(())
}

pub fn execute_task_complete(ctx: &AppContext, id_prefix: &str) -> Result<()> {
    let mut commitment = ctx.find_commitment_by_prefix(id_prefix)?;
    let now = Utc::now();

    commitment.complete(now)?;
    ctx.db.update_commitment(&commitment)?;
    ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::TaskCompleted {
        commitment_id: commitment.id,
    }))?;

    println!(
        "{} Commitment kept [{}]! \"{}\"",
        success_icon(),
        &commitment.id.to_string()[..8].cyan(),
        commitment.title.bold().green()
    );

    // Apply Recovery to active penalties
    let active_penalties = ctx.db.get_active_penalties()?;
    let assessment = RecoveryEngine::evaluate_task_completion(&commitment, active_penalties.first());

    if let Some(mut penalty) = active_penalties.into_iter().next() {
        let reduction = assessment.penalty_reduction_mins;
        penalty.reduce(reduction, now);
        ctx.db.update_penalty(&penalty)?;
        ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::PenaltyReduced {
            penalty_id: penalty.id,
            reduced_by_mins: reduction,
            remaining_mins: penalty.remaining_penalty_mins,
        }))?;

        println!(
            "  {} Recovered {} of penalty! Remaining: {}",
            "⚡".yellow(),
            format_duration(reduction).green().bold(),
            format_duration(penalty.remaining_penalty_mins).red()
        );

        if penalty.remaining_penalty_mins == 0 {
            println!("  {} All penalties cleared!", "🎉".green());
        }
    }

    // Screen time allowance bonus
    let bonus = assessment.screen_time_bonus_mins;
    if bonus > 0 {
        ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::ScreenTimeGranted {
            minutes: bonus,
        }))?;
        println!("  {} Earned +{} entertainment allowance.", "⏱️".cyan(), format_duration(bonus).cyan().bold());
    }

    // Check consistency streak for Save Day reward
    let all_commitments = ctx.db.list_commitments()?;
    let completed_streak = all_commitments.iter().filter(|c| c.status == CommitmentStatus::Completed).count() as u32;
    let available_save_days = ctx.db.get_available_save_days()?.len() as u32;

    if RecoveryEngine::should_earn_save_day(available_save_days, completed_streak, &ctx.config) {
        let new_save_day = SaveDay::new_earned(now);
        ctx.db.insert_save_day(&new_save_day)?;
        ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::SaveDayEarned {
            save_day_id: new_save_day.id,
        }))?;
        println!(
            "  {} Consistency milestone reached! You earned a {} (Balance: {})",
            "🛡️".yellow().bold(),
            "Save Day".green().bold(),
            (available_save_days + 1).to_string().bold()
        );
    }

    Ok(())
}

pub fn execute_task_miss(
    ctx: &AppContext,
    id_prefix: &str,
    force_use_save_day: bool,
    force_no_save_day: bool,
) -> Result<()> {
    let mut commitment = ctx.find_commitment_by_prefix(id_prefix)?;
    let now = Utc::now();

    commitment.miss(now)?;
    ctx.db.update_commitment(&commitment)?;

    // 1. Gather overlapping entertainment activities around this window
    let window_start = commitment.scheduled_start - Duration::minutes(AvoidanceAnalyzer::WINDOW_BUFFER_MINS);
    let window_end = commitment.scheduled_end() + Duration::minutes(AvoidanceAnalyzer::WINDOW_BUFFER_MINS);
    let activities = ctx.db.list_activities_for_window(window_start, window_end)?;

    // 2. Avoidance Assessment
    let avoidance = AvoidanceAnalyzer::assess(&commitment, &activities);

    // 3. Penalty Calculation
    let active_penalties = ctx.db.get_active_penalties()?;
    let current_restricted_days = active_penalties.iter().map(|p| p.restricted_days).max().unwrap_or(0);
    let prev_hit_max = active_penalties.iter().any(|p| p.actual_penalty_mins >= ctx.config.max_daily_penalty_mins);

    let calc = PenaltyCalculator::calculate(
        &commitment,
        &avoidance,
        &ctx.config,
        current_restricted_days,
        prev_hit_max,
    );

    // 4. Save Day Shielding Decision
    let available_save_days = ctx.db.get_available_save_days()?;
    let should_shield = RecoveryEngine::should_shield_missed_task(
        &commitment,
        available_save_days.len() as u32,
        &ctx.config,
        force_use_save_day,
        force_no_save_day,
    );

    if should_shield {
        if let Some(consumed) = ctx.db.consume_oldest_save_day(commitment.id, now)? {
            ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::SaveDayConsumed {
                save_day_id: consumed.id,
                commitment_id: commitment.id,
            }))?;
        }
    }

    let penalty_record = PenaltyCalculator::to_penalty_record(&commitment, &calc, should_shield);
    ctx.db.insert_penalty(&penalty_record)?;

    ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::TaskMissed {
        commitment_id: commitment.id,
        penalty_mins: penalty_record.actual_penalty_mins,
        shielded: should_shield,
    }))?;

    // Display Consequences
    println!(
        "{} Commitment missed [{}] \"{}\"",
        missed_icon(),
        &commitment.id.to_string()[..8].cyan(),
        commitment.title.bold().red()
    );
    println!(
        "  Factors: Importance ({}/5) × Effort ({}/5) × Repetition (x{}) × Entertainment Factor ({}/5)",
        commitment.importance,
        commitment.expected_effort,
        calc.repetition_factor,
        calc.entertainment_factor
    );
    println!(
        "  Raw Score: {} | Avoidance Score: {}",
        calc.raw_penalty_mins.to_string().yellow(),
        calc.avoidance_score.to_string().yellow()
    );

    if should_shield {
        println!(
            "  {} {} Your earned Save Day absorbed this failure! No penalty applied.",
            "🛡️".green().bold(),
            "SHIELDED:".green().bold()
        );
        let remaining_save_days = available_save_days.len().saturating_sub(1);
        println!("  Remaining Save Days: {}", remaining_save_days.to_string().bold());
    } else {
        println!(
            "  {} System consequence: {} restriction applied.",
            "⚠️".red().bold(),
            format_duration(penalty_record.actual_penalty_mins).red().bold()
        );
        if penalty_record.restricted_days > 0 {
            println!(
                "  Restricted Days: {}",
                penalty_record.restricted_days.to_string().red().bold()
            );
        }
        println!(
            "  {}",
            "Remember: Recovery over perfection. Complete a task to reduce this penalty.".italic().dimmed()
        );
    }

    Ok(())
}

pub fn execute_task_delete(ctx: &AppContext, id_prefix: &str) -> Result<()> {
    let commitment = ctx.find_commitment_by_prefix(id_prefix)?;
    ctx.db.delete_commitment(commitment.id)?;
    println!(
        "{} Commitment [{}] deleted.",
        success_icon(),
        &commitment.id.to_string()[..8].cyan()
    );
    Ok(())
}
