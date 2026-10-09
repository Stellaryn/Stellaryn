use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use stellaryn_core::{DISCLAIMER, PRODUCT_NAME};
use stellaryn_diff::{diff_contracts, DiffError, ExitPolicy, FailOn, EXIT_ANALYSIS_ERROR};
use stellaryn_report::{render_json, render_terminal, ReportError};
use stellaryn_wasm::{extract_interface_from_path, WasmError};
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(
    name = "stellaryn",
    version,
    about = "Local-first Soroban contract compatibility analyzer",
    long_about = "Compare compiled Soroban contract WASM files and inspect public contract-spec compatibility changes.",
    after_help = "A COMPATIBLE spec verdict does not prove runtime, storage migration, or deployment safety. Passing Stellaryn is not a security audit."
)]
struct Cli {
    /// Print machine-friendly product metadata.
    #[arg(long)]
    product_info: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Compare two local Soroban WASM contract specifications.
    Compare {
        /// Earlier contract WASM file.
        before: PathBuf,
        /// Newer contract WASM file.
        after: PathBuf,
        /// Output format for the completed analysis.
        #[arg(long, value_enum, default_value_t = ReportFormat::Terminal)]
        format: ReportFormat,
        /// CI failure threshold: breaking (default), review, or never.
        #[arg(long, default_value = "breaking")]
        fail_on: FailOn,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ReportFormat {
    Terminal,
    Json,
}

#[derive(Debug, Error)]
enum CliError {
    #[error("could not extract before-contract specification from '{}': {source}", path.display())]
    Before {
        path: PathBuf,
        #[source]
        source: WasmError,
    },

    #[error("could not extract after-contract specification from '{}': {source}", path.display())]
    After {
        path: PathBuf,
        #[source]
        source: WasmError,
    },

    #[error(transparent)]
    Diff(#[from] DiffError),

    #[error(transparent)]
    Report(#[from] ReportError),
}

fn run(cli: Cli) -> Result<i32, CliError> {
    if cli.product_info {
        println!("product={PRODUCT_NAME}");
        println!("version={}", env!("CARGO_PKG_VERSION"));
        println!("status=pre-alpha");
        println!("disclaimer={DISCLAIMER}");
        return Ok(0);
    }

    match cli.command {
        Some(Commands::Compare {
            before,
            after,
            format,
            fail_on,
        }) => {
            let old = extract_interface_from_path(&before).map_err(|source| CliError::Before {
                path: before.clone(),
                source,
            })?;
            let new = extract_interface_from_path(&after).map_err(|source| CliError::After {
                path: after.clone(),
                source,
            })?;
            let analysis = diff_contracts(&old, &new)?;
            let old_label = before.to_string_lossy();
            let new_label = after.to_string_lossy();
            let report = match format {
                ReportFormat::Terminal => render_terminal(&analysis, &old_label, &new_label),
                ReportFormat::Json => render_json(&analysis, &old_label, &new_label)?,
            };
            println!("{report}");
            Ok(ExitPolicy { fail_on }.exit_code(&analysis))
        }
        None => {
            // Clap handles missing required subcommands with its own error.
            // Product metadata is still explicitly supported at top level.
            println!("Use 'stellaryn compare --help' to compare two contract WASM files.");
            Ok(0)
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => ExitCode::from(code as u8),
        Err(error) => {
            eprintln!("stellaryn: {error}");
            ExitCode::from(EXIT_ANALYSIS_ERROR as u8)
        }
    }
}
