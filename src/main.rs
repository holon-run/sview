use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use sview::{analyze_file, render_text, RenderOptions};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// File to inspect.
    path: PathBuf,

    /// Emit JSON output. Equivalent to `--format json`.
    #[arg(long, conflicts_with = "format")]
    json: bool,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,

    /// Maximum node depth to render.
    #[arg(long, alias = "max-depth")]
    depth: Option<usize>,

    /// Maximum number of nodes to render.
    #[arg(long, default_value_t = 200)]
    max_nodes: usize,

    /// Maximum preview length per node.
    #[arg(long, default_value_t = 120)]
    preview_len: usize,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let view = analyze_file(&cli.path, cli.preview_len)?;
    let options = RenderOptions {
        max_depth: cli.depth,
        max_nodes: cli.max_nodes,
    };

    if cli.json || matches!(cli.format, OutputFormat::Json) {
        println!("{}", serde_json::to_string_pretty(&view)?);
    } else {
        print!("{}", render_text(&view, &options));
    }

    Ok(())
}
