use std::path::PathBuf;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod app;
mod commands;
mod ui;

use app::AppContext;
use commands::{
    activity::{execute_activity_log, execute_activity_list},
    character::execute_character,
    config_cmd::{execute_config_set, execute_config_show},
    evaluate::execute_evaluate,
    init::execute_init,
    recover::execute_recover,
    serve::execute_serve,
    status::execute_status,
    task::{
        execute_task_add, execute_task_complete, execute_task_delete,
        execute_task_list, execute_task_miss, execute_task_start,
    },
};

#[derive(Parser)]
#[command(name = "forgex", author, version, about = "ForgeX: Character Development & Behavioral Accountability System")]
pub struct Cli {
    #[arg(long, global = true, help = "Custom SQLite database file path")]
    pub db: Option<PathBuf>,

    #[arg(long, global = true, help = "Custom configuration file path")]
    pub config: Option<PathBuf>,

    #[arg(long, global = true, help = "Output response in JSON format where supported")]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Display the ForgeX dashboard (commitments, behavior, penalties, Save Days)")]
    Status,

    #[command(about = "Initialize ForgeX database and configuration")]
    Init {
        #[arg(long, help = "Seed database with sample daily commitments and a starter Save Day")]
        sample: bool,
    },

    #[command(about = "Manage commitments and tasks")]
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },

    #[command(about = "Log and view activity / entertainment records")]
    Activity {
        #[command(subcommand)]
        action: ActivityAction,
    },

    #[command(about = "View your behavioral character profile")]
    Character,

    #[command(about = "View active restrictions and recovery paths")]
    Recover,

    #[command(about = "Check and process overdue commitments")]
    Evaluate {
        #[arg(long, help = "Automatically mark overdue commitments as missed")]
        auto_miss: bool,
    },

    #[command(about = "Inspect and update ForgeX settings")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    #[command(about = "Run local network sync daemon for mobile and cross-device syncing")]
    Serve {
        #[arg(long, default_value = "0.0.0.0", help = "Host interface to bind to")]
        bind: String,

        #[arg(short, long, default_value_t = 8080, help = "Port to listen on")]
        port: u16,

        #[arg(long, help = "Custom pairing token (auto-generated if omitted)")]
        token: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum TaskAction {
    #[command(about = "Add a new commitment")]
    Add {
        #[arg(short, long, help = "Title of the commitment")]
        title: String,

        #[arg(short, long, help = "Scheduled start time (e.g. '19:00', '+30m', 'now', or ISO)")]
        start: Option<String>,

        #[arg(short, long, default_value_t = 45, help = "Scheduled duration in minutes")]
        duration: u32,

        #[arg(short, long, default_value_t = 3, help = "Importance score (1 to 5)")]
        importance: u8,

        #[arg(short, long, default_value_t = 3, help = "Expected effort score (1 to 5)")]
        effort: u8,

        #[arg(short, long, default_value = "General", help = "Category / Domain")]
        category: String,

        #[arg(short, long, default_value = "daily", help = "Recurrence: daily, weekdays, once")]
        recurrence: String,

        #[arg(long, help = "Optional description")]
        description: Option<String>,
    },

    #[command(about = "List commitments")]
    List {
        #[arg(short, long, help = "Filter by status (planned, active, completed, missed)")]
        status: Option<String>,
    },

    #[command(about = "Mark a commitment as started")]
    Start {
        #[arg(help = "Commitment ID or prefix")]
        id: String,
    },

    #[command(about = "Mark a commitment as completed (earns recovery points)")]
    Complete {
        #[arg(help = "Commitment ID or prefix")]
        id: String,
    },

    #[command(about = "Mark a commitment as missed (calculates consequences)")]
    Miss {
        #[arg(help = "Commitment ID or prefix")]
        id: String,

        #[arg(long, help = "Force consumption of an available Save Day to shield failure")]
        use_save_day: bool,

        #[arg(long, help = "Prevent Save Day consumption even if eligible")]
        no_save_day: bool,
    },

    #[command(about = "Delete a commitment")]
    Delete {
        #[arg(help = "Commitment ID or prefix")]
        id: String,
    },
}

#[derive(Subcommand)]
pub enum ActivityAction {
    #[command(about = "Log an activity")]
    Log {
        #[arg(short, long, help = "Application name (e.g. 'Firefox', 'VS Code', 'Steam')")]
        app: String,

        #[arg(long, help = "Domain or detail (e.g. 'youtube.com', 'rust-lang.org')")]
        detail: Option<String>,

        #[arg(short, long, help = "Duration in minutes")]
        duration: u32,

        #[arg(
            short,
            long,
            default_value = "medium",
            help = "Category: productive, essential, medium, high, unknown"
        )]
        category: String,

        #[arg(long, help = "When the activity started (defaults to duration mins ago)")]
        started_at: Option<String>,
    },

    #[command(about = "List logged activities")]
    List,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    #[command(about = "Show current configuration")]
    Show,

    #[command(about = "Set a configuration value")]
    Set {
        #[arg(help = "Config key (e.g. max_daily_penalty, auto_save_day_threshold)")]
        key: String,

        #[arg(help = "New value")]
        value: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mut ctx = match AppContext::init(cli.db, cli.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} Error initializing ForgeX: {}", "✖".red().bold(), e);
            std::process::exit(1);
        }
    };

    let result = match cli.command {
        None | Some(Commands::Status) => execute_status(&ctx, cli.json),
        Some(Commands::Init { sample }) => execute_init(&ctx, sample),
        Some(Commands::Serve { bind, port, token }) => {
            execute_serve(&ctx, &bind, port, token.as_deref()).await
        }
        Some(Commands::Task { action }) => match action {
            TaskAction::Add {
                title,
                start,
                duration,
                importance,
                effort,
                category,
                recurrence,
                description,
            } => execute_task_add(
                &ctx,
                &title,
                start.as_deref(),
                duration,
                importance,
                effort,
                &category,
                Some(&recurrence),
                description.as_deref(),
            ),
            TaskAction::List { status } => execute_task_list(&ctx, status.as_deref()),
            TaskAction::Start { id } => execute_task_start(&ctx, &id),
            TaskAction::Complete { id } => execute_task_complete(&ctx, &id),
            TaskAction::Miss {
                id,
                use_save_day,
                no_save_day,
            } => execute_task_miss(&ctx, &id, use_save_day, no_save_day),
            TaskAction::Delete { id } => execute_task_delete(&ctx, &id),
        },
        Some(Commands::Activity { action }) => match action {
            ActivityAction::Log {
                app,
                detail,
                duration,
                category,
                started_at,
            } => execute_activity_log(
                &ctx,
                &app,
                detail.as_deref(),
                duration,
                &category,
                started_at.as_deref(),
            ),
            ActivityAction::List => execute_activity_list(&ctx),
        },
        Some(Commands::Character) => execute_character(&ctx),
        Some(Commands::Recover) => execute_recover(&ctx),
        Some(Commands::Evaluate { auto_miss }) => execute_evaluate(&ctx, auto_miss),
        Some(Commands::Config { action }) => match action {
            ConfigAction::Show => execute_config_show(&ctx),
            ConfigAction::Set { key, value } => execute_config_set(&mut ctx, &key, &value),
        },
    };

    if let Err(e) = result {
        eprintln!("{} Error: {}", "✖".red().bold(), e);
        std::process::exit(1);
    }
}
