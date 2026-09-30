use std::io::{self, Read, Write};

use anyhow::Context;
use clap::{Args, Parser, Subcommand, ValueEnum};
use droid_mcp::{
    CollectRequest, Collector, SlackPostRequest, SourceKind, Topic, format_slack_post,
    markdown::render_markdown, mcp::serve_stdio, slack_form_catalog,
};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(
    name = "droid-mcp",
    version,
    about = "Collect Android and Kotlin news or run the MCP server"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Fetch and normalize recent articles.
    Collect(CollectArgs),
    /// Print the built-in source registry as JSON.
    Sources,
    /// Run the Model Context Protocol server over stdio.
    Serve,
    /// Print the Slack post types, required fields, and form hints as JSON.
    SlackForms,
    /// Read a Slack post request from JSON and emit a validated Block Kit payload.
    FormatSlack(FormatSlackArgs),
}

#[derive(Debug, Args)]
struct FormatSlackArgs {
    /// JSON input file, or - to read from stdin.
    #[arg(long, default_value = "-")]
    input: String,
}

#[derive(Debug, Args)]
struct CollectArgs {
    /// Number of elapsed 24-hour periods to include.
    #[arg(long, default_value_t = 7)]
    days: u32,

    /// Source ID to include. Repeat to select multiple sources.
    #[arg(long = "source")]
    source_ids: Vec<String>,

    /// Topic to include. Repeat to select multiple topics.
    #[arg(long)]
    topic: Vec<Topic>,

    /// Source kind to include: official or community.
    #[arg(long = "source-kind")]
    source_kinds: Vec<SourceKind>,

    /// Maximum number of returned articles.
    #[arg(long, default_value_t = 100)]
    limit: usize,

    /// Include entries that have no published or updated timestamp.
    #[arg(long)]
    include_undated: bool,

    /// Output JSON for automation or Markdown for human sharing.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,

    /// Emit compact JSON instead of pretty-printed JSON.
    #[arg(long)]
    compact: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
    Markdown,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    init_tracing();
    let cli = Cli::parse();
    let collector = Collector::new()?;

    match cli.command {
        Command::Collect(args) => {
            let result = collector
                .collect(CollectRequest {
                    days: args.days,
                    source_ids: args.source_ids,
                    topics: args.topic,
                    source_kinds: args.source_kinds,
                    limit: args.limit,
                    include_undated: args.include_undated,
                })
                .await?;
            let stdout = io::stdout();
            let mut writer = stdout.lock();
            match args.format {
                OutputFormat::Json if args.compact => {
                    serde_json::to_writer(&mut writer, &result)
                        .context("failed to serialize JSON output")?;
                    writeln!(writer)?;
                }
                OutputFormat::Json => {
                    serde_json::to_writer_pretty(&mut writer, &result)
                        .context("failed to serialize JSON output")?;
                    writeln!(writer)?;
                }
                OutputFormat::Markdown => {
                    writer.write_all(render_markdown(&result).as_bytes())?;
                }
            }
        }
        Command::Sources => {
            let stdout = io::stdout();
            let mut writer = stdout.lock();
            serde_json::to_writer_pretty(&mut writer, &collector.list_sources())?;
            writeln!(writer)?;
        }
        Command::Serve => serve_stdio(collector).await?,
        Command::SlackForms => {
            let stdout = io::stdout();
            let mut writer = stdout.lock();
            serde_json::to_writer_pretty(&mut writer, &slack_form_catalog())?;
            writeln!(writer)?;
        }
        Command::FormatSlack(args) => {
            let input = read_input(&args.input)?;
            let request: SlackPostRequest = serde_json::from_str(&input).with_context(|| {
                format!("failed to parse Slack post JSON from '{}'", args.input)
            })?;
            let result = format_slack_post(request)?;
            let stdout = io::stdout();
            let mut writer = stdout.lock();
            serde_json::to_writer_pretty(&mut writer, &result)?;
            writeln!(writer)?;
        }
    }

    Ok(())
}

fn read_input(path: &str) -> anyhow::Result<String> {
    if path == "-" {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .context("failed to read Slack post JSON from stdin")?;
        Ok(input)
    } else {
        std::fs::read_to_string(path)
            .with_context(|| format!("failed to read Slack post JSON from '{path}'"))
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(io::stderr)
        .with_target(false)
        .try_init();
}
