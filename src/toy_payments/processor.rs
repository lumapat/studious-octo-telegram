use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;

type ClientId = u16;
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
    client_id: ClientId,
    #[serde(rename = "tx")]
    transaction_id: u32,
    #[serde(deserialize_with = "deserialize_amount")]
    amount: Amount,
}

#[derive(Debug)]
pub struct Account {
    available_funds: Amount,
    held_funds: Amount,
    is_locked: bool,
}

impl Account {
    pub fn new() -> Self {
        Self {
            available_funds: 0,
            held_funds: 0,
            is_locked: false,
        }
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PaymentProcessor {
    accounts: HashMap<ClientId, Account>,
}

impl PaymentProcessor {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    pub fn process(&mut self, transaction: &Transaction) {
        let account = self
            .accounts
            .entry(transaction.client_id)
            .or_insert_with(Account::new);

        match transaction.ty {
            TransactionType::Deposit => {
                account.available_funds += transaction.amount;
            }
            TransactionType::Withdrawal => {
                if account.available_funds >= transaction.amount {
                    account.available_funds -= transaction.amount;
                }
            }
            // TODO: These for later
            TransactionType::Dispute => {}
            TransactionType::Resolve => {}
            TransactionType::Chargeback => {}
        }
    }

    pub fn dump_csv(&self) -> Result<(), Box<dyn std::error::Error>> {
        use csv::Writer;

        let mut wtr = Writer::from_writer(std::io::stdout());

        // TODO: Write in here for now, put in a separate class later
        #[derive(Serialize)]
        struct AccountRow {
            #[serde(rename = "client")]
            client_id: ClientId,
            #[serde(rename = "available", serialize_with = "serialize_amount")]
            available_funds: Amount,
            #[serde(rename = "held", serialize_with = "serialize_amount")]
            held_funds: Amount,
            #[serde(rename = "total", serialize_with = "serialize_total")]
            total_funds: Amount,
            #[serde(rename = "locked")]
            is_locked: bool,
        }

        fn serialize_total<S>(amount: &Amount, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let amount_float = *amount as f64 / 10000.0;
            serializer.serialize_f64(amount_float)
        }

        for client_id in self.accounts.keys() {
            let account = &self.accounts[client_id];
            wtr.serialize(AccountRow {
                client_id: *client_id,
                available_funds: account.available_funds,
                held_funds: account.held_funds,
                total_funds: account.available_funds + account.held_funds,
                is_locked: account.is_locked,
            })?;
        }

        wtr.flush()?;
        Ok(())
    }
}

impl Default for PaymentProcessor {
    fn default() -> Self {
        Self::new()
    }
}

pub fn deserialize_amount<'de, D>(deserializer: D) -> Result<Amount, D::Error>
where
    D: Deserializer<'de>,
{
    let amount_float: f64 = Deserialize::deserialize(deserializer)?;
    Ok((amount_float * 10000.0) as Amount)
}

pub fn serialize_amount<S>(amount: &Amount, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let amount_float = *amount as f64 / 10000.0;
    serializer.serialize_f64(amount_float)
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
