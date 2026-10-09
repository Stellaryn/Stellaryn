use clap::Parser;
use stellaryn_core::{DISCLAIMER, PRODUCT_NAME};

#[derive(Debug, Parser)]
#[command(
    name = "stellaryn",
    version,
    about = "Local-first Soroban contract compatibility analyzer",
    long_about = "Stellaryn reads Soroban contract specifications and compares public-interface compatibility through its Rust libraries. The user-facing compare command and reports are planned for Phase 8.",
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
        println!("status=pre-alpha");
        println!("disclaimer={DISCLAIMER}");
    }
}
