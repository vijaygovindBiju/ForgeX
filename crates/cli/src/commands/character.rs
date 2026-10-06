use colored::Colorize;
use forgex_core::Result;
use forgex_scoring::CharacterCalculator;
use crate::app::AppContext;
use crate::ui::formatting::*;

pub fn execute_character(ctx: &AppContext) -> Result<()> {
    let commitments = ctx.db.list_commitments()?;
    let penalties = ctx.db.list_penalties()?;

    let profile = CharacterCalculator::calculate(&commitments, &penalties);

    println!("{}", header("FORGEX CHARACTER PROFILE"));
    println!(
        "{:<20} Level {} {}",
        "Mastery:",
        profile.overall_level.to_string().bold().cyan(),
        format!("(Overall Average: {}%)", (profile.consistency + profile.reliability + profile.resilience + profile.self_control + profile.discipline) / 5).dimmed()
    );
    println!();

    print_metric("Consistency", profile.consistency, "How often commitments are fulfilled");
    print_metric("Reliability", profile.reliability, "Execution without prior avoidance or skips");
    print_metric("Resilience", profile.resilience, "Recovery rate after a missed commitment");
    print_metric("Self-Control", profile.self_control, "Absence of entertainment during active tasks");
    print_metric("Discipline", profile.discipline, "Follow-through on high-effort & high-importance tasks");

    println!();
    println!("{}", "These metrics reflect observed behavior, not your human worth.".italic().dimmed());
    println!("{}", "You forge character through repeated follow-through.".dimmed());
    println!();

    Ok(())
}

fn print_metric(name: &str, score: u32, desc: &str) {
    let score_str = format!("{:>3}%", score);
    let color_score = if score >= 80 {
        score_str.green().bold()
    } else if score >= 60 {
        score_str.yellow().bold()
    } else {
        score_str.red().bold()
    };

    println!(
        "{:<16} {}  {}  {}",
        name.bold(),
        color_score,
        progress_bar(score, 100, 15),
        desc.dimmed()
    );
}
