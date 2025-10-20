use std::{fs::File, path::PathBuf};

use super::Transaction;
use csv::{Reader, ReaderBuilder};

pub struct TransactionReader {
    reader: Reader<File>,
}

impl TransactionReader {
    pub fn from_path(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let reader = ReaderBuilder::new()
            .flexible(false)
            .trim(csv::Trim::All)
            .from_path(path)?;

        Ok(Self { reader })
    }

    pub fn read_all(&mut self) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
        let mut txns: Vec<Transaction> = Vec::new();

        for result in self.reader.deserialize().into_iter() {
            match result {
                Ok(txn) => txns.push(txn),
                Err(err) => {
                    return Err(Box::new(err));
                }
            }
        }

        Ok(txns)
    }
}
