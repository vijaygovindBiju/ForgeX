use colored::Colorize;
use forgex_core::Result;
use crate::app::AppContext;
use crate::commands::task::execute_task_miss;
use crate::ui::formatting::*;

pub fn execute_evaluate(ctx: &AppContext, auto_miss: bool) -> Result<()> {
    let overdue = ctx.check_overdue_commitments()?;

    println!("{}", header("COMMITMENT EVALUATION"));

    if overdue.is_empty() {
        println!("{}", "✓ All commitments are up to date! No overdue tasks.".green().bold());
        return Ok(());
    }

    println!(
        "Found {} past-due commitment(s) that exceeded the scheduled window + {}m grace period:",
        overdue.len().to_string().bold().red(),
        ctx.config.grace_period_mins
    );

    for c in &overdue {
        println!(
            "  • [{}] {} (Scheduled end was {})",
            &c.id.to_string()[..8].cyan(),
            c.title.bold(),
            c.scheduled_end().format("%Y-%m-%d %H:%M UTC").to_string().yellow()
        );
    }

    if auto_miss {
        println!("\nProcessing overdue commitments as missed...");
        for c in overdue {
            let short_id = &c.id.to_string()[..8];
            execute_task_miss(ctx, short_id, false, false)?;
        }
    } else {
        println!(
            "\nRun {} to apply consequences automatically, or complete/excuse them individually with:",
            "forgex evaluate --auto-miss".bold().white()
        );
        println!("  {} <ID>", "forgex task complete".dimmed());
        println!("  {} <ID>", "forgex task miss".dimmed());
    }

    Ok(())
}
