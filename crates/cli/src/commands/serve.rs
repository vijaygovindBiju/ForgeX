use colored::Colorize;
use forgex_core::{ForgeError, Result};
use crate::app::AppContext;
use crate::ui::formatting::*;

pub async fn execute_serve(
    ctx: &AppContext,
    bind_host: &str,
    port: u16,
    custom_token: Option<&str>,
) -> Result<()> {
    let token = match custom_token {
        Some(t) => t.to_string(),
        None => format!("fx_{}", &uuid::Uuid::new_v4().to_string()[..8]),
    };

    println!("{}", header("FORGEX CROSS-PLATFORM SYNC DAEMON"));
    println!("{} Sync Daemon starting...", "🚀".green());
    println!("Database: {}", ctx.db_path.display().to_string().cyan());
    println!("{:<24} {}", "Binding Interface:", bind_host.bold());
    println!("{:<24} {}", "Port:", port.to_string().bold());
    println!("{:<24} {}", "Pairing Token:", token.bold().yellow());
    println!();

    println!("{}", "CONNECT YOUR ANDROID APP:".bold().cyan());
    println!("1. Open ForgeX on your Android phone.");
    println!("2. In Settings / Pair Device, enter your laptop's Wi-Fi IP and port:");
    println!("   {} http://<YOUR_LAPTOP_IP>:{}", "Server URL:".dimmed(), port);
    println!("   {} {}", "Pairing Token:".dimmed(), token.bold().yellow());
    println!();
    println!("{}", "Press Ctrl+C to stop the sync server.".dimmed());
    println!();

    forgex_server::start_server(
        ctx.db.clone(),
        ctx.config.clone(),
        token,
        bind_host,
        port,
    ).await.map_err(|e| ForgeError::Storage(format!("Server error: {}", e)))?;

    Ok(())
}
