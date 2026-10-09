use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use stellaryn_core::{ContractInterface, DISCLAIMER, PRODUCT_NAME};
use stellaryn_diff::{diff_contracts, DiffError, ExitPolicy, FailOn, EXIT_ANALYSIS_ERROR};
use stellaryn_report::{render_json, render_terminal, ReportError};
use stellaryn_wasm::{extract_interface_from_path, extract_interface_from_wasm, WasmError};
use thiserror::Error;

mod git;
use git::{read_git_wasm, GitArtifactError};

#[derive(Debug, Parser)]
#[command(
    name = "stellaryn",
    version,
    about = "Local-first Soroban contract compatibility analyzer",
    long_about = "Compare Soroban contract WASM files, including versions committed at Git revisions, and inspect public contract-spec compatibility changes.",
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

    /// Compare committed Soroban WASM blobs at two Git revisions without checkout.
    Git {
        /// Local Git repository (default: current directory).
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        /// Baseline commit, branch, tag, or ancestry expression.
        #[arg(long = "from", required = true)]
        from: String,
        /// Target commit, branch, tag, or ancestry expression.
        #[arg(long = "to", required = true)]
        to: String,
        /// Path to the baseline WASM artifact, relative to the Git repository root.
        #[arg(long, required = true)]
        wasm: String,
        /// Different repository-relative WASM path at the target revision.
        #[arg(long)]
        after_wasm: Option<String>,
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

    #[error("could not read before Git artifact '{revision}:{path}': {source}")]
    GitBefore {
        revision: String,
        path: String,
        #[source]
        source: GitArtifactError,
    },

    #[error("could not read after Git artifact '{revision}:{path}': {source}")]
    GitAfter {
        revision: String,
        path: String,
        #[source]
        source: GitArtifactError,
    },

    #[error("could not extract before Git artifact specification '{revision}:{path}': {source}")]
    GitSpecBefore {
        revision: String,
        path: String,
        #[source]
        source: WasmError,
    },

    #[error("could not extract after Git artifact specification '{revision}:{path}': {source}")]
    GitSpecAfter {
        revision: String,
        path: String,
        #[source]
        source: WasmError,
    },

    #[error(transparent)]
    Diff(#[from] DiffError),

    #[error(transparent)]
    Report(#[from] ReportError),
}

fn complete_analysis(
    old: &ContractInterface,
    new: &ContractInterface,
    before_label: &str,
    after_label: &str,
    format: ReportFormat,
    fail_on: FailOn,
) -> Result<i32, CliError> {
    let analysis = diff_contracts(old, new)?;
    let report = match format {
        ReportFormat::Terminal => render_terminal(&analysis, before_label, after_label),
        ReportFormat::Json => render_json(&analysis, before_label, after_label)?,
    };
    println!("{report}");
    Ok(ExitPolicy { fail_on }.exit_code(&analysis))
}

fn run(cli: Cli) -> Result<i32, CliError> {
    if cli.product_info {
        println!("product={PRODUCT_NAME}");
        println!("version={}", env!("CARGO_PKG_VERSION"));
        println!("status=initial-release");
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
            complete_analysis(
                &old,
                &new,
                &before.to_string_lossy(),
                &after.to_string_lossy(),
                format,
                fail_on,
            )
        }
        Some(Commands::Git {
            repo,
            from,
            to,
            wasm,
            after_wasm,
            format,
            fail_on,
        }) => {
            let target_wasm = after_wasm.as_deref().unwrap_or(&wasm);
            let old_bytes =
                read_git_wasm(&repo, &from, &wasm).map_err(|source| CliError::GitBefore {
                    revision: from.clone(),
                    path: wasm.clone(),
                    source,
                })?;
            let new_bytes =
                read_git_wasm(&repo, &to, target_wasm).map_err(|source| CliError::GitAfter {
                    revision: to.clone(),
                    path: target_wasm.to_owned(),
                    source,
                })?;
            let old = extract_interface_from_wasm(&old_bytes).map_err(|source| {
                CliError::GitSpecBefore {
                    revision: from.clone(),
                    path: wasm.clone(),
                    source,
                }
            })?;
            let new = extract_interface_from_wasm(&new_bytes).map_err(|source| {
                CliError::GitSpecAfter {
                    revision: to.clone(),
                    path: target_wasm.to_owned(),
                    source,
                }
            })?;
            complete_analysis(
                &old,
                &new,
                &format!("git:{from}:{wasm}"),
                &format!("git:{to}:{target_wasm}"),
                format,
                fail_on,
            )
        }
        None => {
            println!(
                "Use 'stellaryn compare --help' or 'stellaryn git --help' to analyze contracts."
            );
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
