use serde::{Deserialize, Deserializer};
use std::fmt;

type Amount = u64;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Chargeback,
    Deposit,
    Dispute,
    Resolve,
    Withdrawal,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    #[serde(rename = "type")]
    ty: TransactionType,
    #[serde(rename = "client")]
    client_id: u16,
    #[serde(rename = "tx")]
    transaction_id: u32,
    #[serde(deserialize_with = "deserialize_amount")]
    amount: Amount,
}

pub fn deserialize_amount<'de, D>(deserializer: D) -> Result<Amount, D::Error>
where
    D: Deserializer<'de>,
{
    let amount_float: f64 = Deserialize::deserialize(deserializer)?;
    Ok((amount_float * 10000.0) as Amount)
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionType::Chargeback => write!(f, "chargeback"),
            TransactionType::Deposit => write!(f, "deposit"),
            TransactionType::Dispute => write!(f, "dispute"),
            TransactionType::Resolve => write!(f, "resolve"),
            TransactionType::Withdrawal => write!(f, "withdrawal"),
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let amount_float = self.amount as f64 / 10000.0;
        write!(
            f,
            "type: {}, client: {}, tx: {}, amount: {:.4}",
            self.ty, self.client_id, self.transaction_id, amount_float
        )
    }
}
