use std::{fs::File, path::PathBuf};

use super::Transaction;
use csv::{DeserializeRecordsIter, Reader, ReaderBuilder};

pub struct TransactionReader {
    reader: Reader<File>,
}

impl TransactionReader {
    pub fn from_path(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let reader = ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_path(path)?;

        Ok(Self { reader })
    }

    // Expose an iter() here so we can stream CSV records
    pub fn iter(&mut self) -> DeserializeRecordsIter<'_, File, Transaction> {
        self.reader.deserialize()
    }
}
