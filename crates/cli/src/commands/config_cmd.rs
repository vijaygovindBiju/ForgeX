use colored::Colorize;
use forgex_core::Result;
use crate::app::AppContext;
use crate::ui::formatting::*;

pub fn execute_config_show(ctx: &AppContext) -> Result<()> {
    println!("{}", header("FORGEX CONFIGURATION"));
    println!("Config File: {}", ctx.config_path.display().to_string().cyan());
    println!("{:<36} {}m", "Max Daily Penalty:", ctx.config.max_daily_penalty_mins);
    println!("{:<36} {}", "Max Save Days:", ctx.config.save_days_max);
    println!("{:<36} {} days", "Save Day Streak Required:", ctx.config.save_day_streak_required);
    println!("{:<36} {}/5", "Auto Save Day Threshold:", ctx.config.auto_save_day_importance_threshold);
    println!("{:<36} {}m", "Default Entertainment Allowance:", ctx.config.default_entertainment_allowance_mins);
    println!("{:<36} {}m", "Grace Period:", ctx.config.grace_period_mins);
    Ok(())
}

pub fn execute_config_set(ctx: &mut AppContext, key: &str, value: &str) -> Result<()> {
    match key.to_lowercase().as_str() {
        "max_daily_penalty" | "max_daily_penalty_mins" => {
            ctx.config.max_daily_penalty_mins = value.parse().map_err(|_| {
                forgex_core::ForgeError::Validation("Value must be a positive integer".into())
            })?;
        }
        "save_days_max" => {
            ctx.config.save_days_max = value.parse().map_err(|_| {
                forgex_core::ForgeError::Validation("Value must be a positive integer".into())
            })?;
        }
        "auto_save_day_threshold" => {
            let thresh: u8 = value.parse().map_err(|_| {
                forgex_core::ForgeError::Validation("Value must be an integer between 1 and 5".into())
            })?;
            ctx.config.auto_save_day_importance_threshold = thresh.clamp(1, 5);
        }
        "grace_period_mins" => {
            ctx.config.grace_period_mins = value.parse().map_err(|_| {
                forgex_core::ForgeError::Validation("Value must be a positive integer".into())
            })?;
        }
        _ => {
            return Err(forgex_core::ForgeError::Validation(format!(
                "Unknown configuration key: {}",
                key
            )));
        }
    }

    ctx.save_config()?;
    println!("{} Updated {} = {}", success_icon(), key.bold(), value.cyan());
    Ok(())
}
