use chrono::{Duration, Utc};
use colored::Colorize;
use forgex_core::{
    Commitment, ForgeEvent, ForgeEventPayload, RecurrenceRule, Result, SaveDay,
};
use crate::app::AppContext;

pub fn execute_init(ctx: &AppContext, sample: bool) -> Result<()> {
    ctx.save_config()?;

    println!("{}", "ForgeX Initialized Successfully.".green().bold());
    println!("Database: {}", ctx.db_path.display().to_string().cyan());
    println!("Config:   {}", ctx.config_path.display().to_string().cyan());

    if sample {
        let now = Utc::now();

        // 1. Workout
        let workout = Commitment::new(
            "Morning Workout",
            4,
            4,
            now - Duration::hours(2),
            45,
            RecurrenceRule::Daily,
            "Health",
        )?;
        ctx.db.insert_commitment(&workout)?;
        ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::TaskCreated {
            commitment_id: workout.id,
            title: workout.title.clone(),
        }))?;

        // 2. Deep Work
        let study = Commitment::new(
            "Deep Work — Systems Architecture",
            5,
            4,
            now + Duration::minutes(30),
            90,
            RecurrenceRule::Daily,
            "Deep Work",
        )?;
        ctx.db.insert_commitment(&study)?;
        ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::TaskCreated {
            commitment_id: study.id,
            title: study.title.clone(),
        }))?;

        // 3. Reading
        let reading = Commitment::new(
            "Evening Reading",
            3,
            2,
            now + Duration::hours(5),
            30,
            RecurrenceRule::Daily,
            "Growth",
        )?;
        ctx.db.insert_commitment(&reading)?;
        ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::TaskCreated {
            commitment_id: reading.id,
            title: reading.title.clone(),
        }))?;

        // Seed 1 starter Save Day
        let initial_save_day = SaveDay::new_earned(now);
        ctx.db.insert_save_day(&initial_save_day)?;
        ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::SaveDayEarned {
            save_day_id: initial_save_day.id,
        }))?;

        println!("\n{}", "Sample data created:".bold().yellow());
        println!("  - 3 sample commitments for today");
        println!("  - 1 starter Save Day granted to protect your early streak");
        println!("\nRun {} to see your dashboard.", "forgex status".bold().white());
    }

    Ok(())
}
