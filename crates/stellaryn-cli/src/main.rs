use clap::Parser;
use stellaryn_core::{DISCLAIMER, PRODUCT_NAME};

#[derive(Debug, Parser)]
#[command(
    name = "stellaryn",
    version,
    about = "Local-first Soroban contract compatibility analyzer",
    long_about = "Stellaryn compares Soroban contract interfaces to explain compatibility changes before an upgrade ships. Phase 1 establishes the CLI and workspace only; comparison logic is added in later verified phases.",
    after_help = "Stellaryn currently exposes foundation metadata only. No compatibility result is produced in Phase 1.\n\nPassing Stellaryn is not a security audit and does not prove that a contract upgrade is safe to deploy."
)]
struct Cli {
    /// Print product metadata for automation and smoke tests.
    #[arg(long)]
    product_info: bool,
}

fn main() {
    let cli = Cli::parse();
    if cli.product_info {
        println!("product={PRODUCT_NAME}");
        println!("version={}", env!("CARGO_PKG_VERSION"));
        println!("status=foundation");
        println!("disclaimer={DISCLAIMER}");
    }
}
