use std::path::PathBuf;

use clap::Parser;

mod toy_payments;
use toy_payments::TransactionReader;

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

    match TransactionReader::from_path(args.input_file) {
        Ok(mut reader) => match reader.read_all() {
            Ok(txns) => txns.iter().for_each(|txn| println!("{}", txn)),
            Err(err) => eprintln!("Error reading transactions: {}", err),
        },
        Err(err) => eprintln!("Error opening file: {}", err),
    }
}
