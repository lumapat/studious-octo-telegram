use std::path::PathBuf;

use clap::Parser;

/// Processes an input CSV file of payments transactions
/// and outputs a CSV file of outstanding account balances
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path of the input CSV file
    input_file: PathBuf,

    /// Emit debug
    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

fn main() {
    let args = Args::parse();
    println!("Hello, world! {:#?}", args);
}
