use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;

use super::amount::Amount;

type TransactionId = u32;
type ClientId = u16;

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
    transaction_id: TransactionId,
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
            available_funds: Amount::from(0),
            held_funds: Amount::from(0),
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
    compressed_transactions: HashMap<TransactionId, Amount>,
}

impl PaymentProcessor {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            compressed_transactions: HashMap::new(),
        }
    }

    fn find_transaction(&self, transaction_id: TransactionId) -> Option<&Amount> {
        self.compressed_transactions.get(&transaction_id)
    }

    fn store_transaction(&mut self, transaction_id: TransactionId, amount: Amount) {
        self.compressed_transactions.insert(transaction_id, amount);
    }

    fn get_account(&mut self, client_id: ClientId) -> &mut Account {
        self.accounts.entry(client_id).or_insert_with(Account::new)
    }

    pub fn process(&mut self, transaction: &Transaction) {
        match transaction.ty {
            TransactionType::Deposit => {
                let account = self.get_account(transaction.client_id);
                account.available_funds += transaction.amount;
                self.store_transaction(transaction.transaction_id, transaction.amount);
            }
            TransactionType::Withdrawal => {
                let account = self.get_account(transaction.client_id);
                // Only process withdrawal if there are sufficient available funds
                // Ignore any withdrawals that go beyond the available amount (per requirements)
                if account.available_funds >= transaction.amount {
                    account.available_funds -= transaction.amount;
                    // We can represent withdrawals as negative amounts, so we only need to store
                    // the amount and its transaction ID for a more compressed log
                    self.store_transaction(transaction.transaction_id, -transaction.amount);
                }
            }
            TransactionType::Dispute => {
                if let Some(txn_amount) = self.find_transaction(transaction.transaction_id).copied()
                {
                    let account = self.get_account(transaction.client_id);
                    account.available_funds -= txn_amount;
                    account.held_funds += txn_amount;
                }
            }
            TransactionType::Resolve => {
                if let Some(txn_amount) = self.find_transaction(transaction.transaction_id).copied()
                {
                    let account = self.get_account(transaction.client_id);
                    account.available_funds += txn_amount;
                    account.held_funds -= txn_amount;
                }
            }
            TransactionType::Chargeback => {
                if let Some(txn_amount) = self.find_transaction(transaction.transaction_id).copied()
                {
                    let account = self.get_account(transaction.client_id);
                    account.held_funds -= txn_amount;
                    account.is_locked = true;
                }
            }
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
            let amount_float: f64 = (*amount).into();
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
    Ok(Amount::from(amount_float))
}

pub fn serialize_amount<S>(amount: &Amount, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let amount_float: f64 = (*amount).into();
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
        let amount_float: f64 = self.amount.into();
        write!(
            f,
            "type: {}, client: {}, tx: {}, amount: {:.4}",
            self.ty, self.client_id, self.transaction_id, amount_float
        )
    }
}

impl Transaction {
    #[cfg(test)]
    pub fn new(
        ty: TransactionType,
        client_id: ClientId,
        transaction_id: TransactionId,
        amount: Amount,
    ) -> Self {
        Self {
            ty,
            client_id,
            transaction_id,
            amount,
        }
    }
}

// Tests aren't exhaustive here, but captured mostly
// the important ones that are defined in the requirements
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit_only() {
        let mut processor = PaymentProcessor::new();

        processor.process(&Transaction::new(
            TransactionType::Deposit,
            1,
            1,
            Amount::from(1),
        ));
        processor.process(&Transaction::new(
            TransactionType::Deposit,
            1,
            2,
            Amount::from(2),
        ));

        let account = &processor.accounts[&1];
        assert_eq!(account.available_funds, Amount::from(3));
        assert_eq!(account.held_funds, Amount::from(0));
    }

    #[test]
    fn test_withdraw() {
        let mut processor = PaymentProcessor::new();

        processor.process(&Transaction::new(
            TransactionType::Deposit,
            1,
            1,
            Amount::from(5),
        ));
        processor.process(&Transaction::new(
            TransactionType::Withdrawal,
            1,
            2,
            Amount::from(1.5),
        ));

        let account = &processor.accounts[&1];
        assert_eq!(account.available_funds, Amount::from(3.5));
        assert_eq!(account.held_funds, Amount::from(0));
    }

    #[test]
    fn test_withdrawal_insufficient_funds() {
        let mut processor = PaymentProcessor::new();

        processor.process(&Transaction::new(
            TransactionType::Deposit,
            1,
            1,
            Amount::from(10),
        ));

        processor.process(&Transaction::new(
            TransactionType::Withdrawal,
            1,
            2,
            Amount::from(15),
        ));

        let account = &processor.accounts[&1];
        assert_eq!(account.available_funds, Amount::from(10));
        assert_eq!(account.held_funds, Amount::from(0));
    }

    #[test]
    fn test_withdraw_deposit() {
        let mut processor = PaymentProcessor::new();

        processor.process(&Transaction::new(
            TransactionType::Deposit,
            1,
            1,
            Amount::from(1),
        ));
        processor.process(&Transaction::new(
            TransactionType::Deposit,
            1,
            2,
            Amount::from(2),
        ));
        processor.process(&Transaction::new(
            TransactionType::Withdrawal,
            1,
            3,
            Amount::from(1.5),
        ));
        processor.process(&Transaction::new(
            TransactionType::Deposit,
            1,
            4,
            Amount::from(0.5),
        ));
        processor.process(&Transaction::new(
            TransactionType::Withdrawal,
            1,
            5,
            Amount::from(0.8),
        ));

        let account = &processor.accounts[&1];
        assert_eq!(account.available_funds, Amount::from(1.2));
        assert_eq!(account.held_funds, Amount::from(0));
    }

    #[test]
    fn test_deposit_withdraw_dispute() {
        let mut processor = PaymentProcessor::new();

        processor.process(&Transaction::new(
            TransactionType::Deposit,
            1,
            1,
            Amount::from(10),
        ));
        processor.process(&Transaction::new(
            TransactionType::Withdrawal,
            1,
            2,
            Amount::from(3),
        ));
        processor.process(&Transaction::new(
            TransactionType::Dispute,
            1,
            2,
            Amount::from(0),
        ));

        let account = &processor.accounts[&1];
        assert_eq!(account.available_funds, Amount::from(10));
        // Negative since we're holding back a withdrawal
        assert_eq!(account.held_funds, -Amount::from(3));
    }

    #[test]
    fn test_deposit_dispute() {
        let mut processor = PaymentProcessor::new();

        processor.process(&Transaction::new(
            TransactionType::Deposit,
            1,
            1,
            Amount::from(10),
        ));
        processor.process(&Transaction::new(
            TransactionType::Dispute,
            1,
            1,
            Amount::from(0),
        ));

        let account = &processor.accounts[&1];
        assert_eq!(account.available_funds, Amount::from(0));
        assert_eq!(account.held_funds, Amount::from(10));
    }

    #[test]
    fn test_deposit_dispute_resolve() {
        let mut processor = PaymentProcessor::new();

        processor.process(&Transaction::new(
            TransactionType::Deposit,
            1,
            1,
            Amount::from(10),
        ));
        processor.process(&Transaction::new(
            TransactionType::Dispute,
            1,
            1,
            Amount::from(0),
        ));
        processor.process(&Transaction::new(
            TransactionType::Resolve,
            1,
            1,
            Amount::from(0),
        ));

        let account = &processor.accounts[&1];
        assert_eq!(account.available_funds, Amount::from(10));
        assert_eq!(account.held_funds, Amount::from(0));
    }

    #[test]
    fn test_deposit_dispute_chargeback() {
        let mut processor = PaymentProcessor::new();

        processor.process(&Transaction::new(
            TransactionType::Deposit,
            1,
            1,
            Amount::from(10),
        ));
        processor.process(&Transaction::new(
            TransactionType::Dispute,
            1,
            1,
            Amount::from(0),
        ));
        processor.process(&Transaction::new(
            TransactionType::Chargeback,
            1,
            1,
            Amount::from(0),
        ));

        let account = &processor.accounts[&1];
        assert_eq!(account.available_funds, Amount::from(0));
        assert_eq!(account.held_funds, Amount::from(0));
        assert_eq!(account.is_locked, true);
    }

    #[test]
    fn test_deposit_withdraw_deposit_dispute_withdrawal_chargeback() {
        let mut processor = PaymentProcessor::new();

        processor.process(&Transaction::new(
            TransactionType::Deposit,
            1,
            1,
            Amount::from(100),
        ));
        processor.process(&Transaction::new(
            TransactionType::Withdrawal,
            1,
            2,
            Amount::from(20),
        ));
        processor.process(&Transaction::new(
            TransactionType::Deposit,
            1,
            3,
            Amount::from(50),
        ));
        processor.process(&Transaction::new(
            TransactionType::Dispute,
            1,
            2,
            Amount::from(0),
        ));
        processor.process(&Transaction::new(
            TransactionType::Chargeback,
            1,
            2,
            Amount::from(0),
        ));

        let account = &processor.accounts[&1];
        assert_eq!(account.available_funds, Amount::from(150));
        assert_eq!(account.held_funds, Amount::from(0));
        assert_eq!(account.is_locked, true);
    }

    #[test]
    fn test_invalid_transaction_id_no_state_change() {
        let transaction_types = vec![
            TransactionType::Dispute,
            TransactionType::Resolve,
            TransactionType::Chargeback,
        ];

        for tx_type in transaction_types {
            let mut processor = PaymentProcessor::new();

            processor.process(&Transaction::new(
                TransactionType::Deposit,
                1,
                1,
                Amount::from(100),
            ));

            let account_before = &processor.accounts[&1];
            let available_before = account_before.available_funds;
            let held_before = account_before.held_funds;

            processor.process(&Transaction::new(tx_type, 1, 999, Amount::from(0)));

            let account_after = &processor.accounts[&1];
            assert_eq!(account_after.available_funds, available_before);
            assert_eq!(account_after.held_funds, held_before);
        }
    }
}
