use colored::Colorize;
use forgex_core::{CommitmentStatus, Result};
use crate::app::AppContext;
use crate::ui::formatting::*;

pub fn execute_recover(ctx: &AppContext) -> Result<()> {
    let active_penalties = ctx.db.get_active_penalties()?;
    let commitments = ctx.db.list_commitments()?;

    println!("{}", header("RECOVERY CENTER"));

    let total_remaining_mins: u32 = active_penalties.iter().map(|p| p.remaining_penalty_mins).sum();
    let max_restricted_days: u32 = active_penalties.iter().map(|p| p.restricted_days).max().unwrap_or(0);

    if total_remaining_mins == 0 && max_restricted_days == 0 {
        println!("{}", "✓ No active penalties or restrictions!".green().bold());
        println!("You have full freedom. Keep building your consistency streak to earn Save Days.");
        return Ok(());
    }

    println!(
        "Active Restriction: {} remaining across {} restricted day(s)",
        format_duration(total_remaining_mins).red().bold(),
        max_restricted_days.to_string().red().bold()
    );

    println!("\n{}", "HOW TO RECOVER:".bold().yellow());
    println!("  • Completing an effort 1 task -> Recovers 15m penalty + 5m entertainment");
    println!("  • Completing an effort 3 task -> Recovers 45m penalty + 15m entertainment");
    println!("  • Completing an effort 5 task -> Recovers 75m penalty + 25m entertainment");
    println!("  • Clearing all minutes on a day with restricted days will reduce restricted days by 1!");

    let planned_tasks: Vec<_> = commitments
        .iter()
        .filter(|c| c.status == CommitmentStatus::Planned || c.status == CommitmentStatus::Active)
        .collect();

    if !planned_tasks.is_empty() {
        println!("\n{}", "Available upcoming tasks you can complete right now:".cyan().bold());
        for t in planned_tasks {
            let potential_recovery = (t.expected_effort as u32) * 15;
            println!(
                "  • [{}] {} (Effort {}/5) -> Earns {} recovery",
                &t.id.to_string()[..8].cyan(),
                t.title.bold(),
                t.expected_effort,
                format_duration(potential_recovery).green().bold()
            );
        }
    } else {
        println!("\nSchedule a new commitment with {} to start recovering.", "forgex task add".white().bold());
    }

    println!();
    Ok(())
}
