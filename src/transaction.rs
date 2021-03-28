use rust_decimal::prelude::*;
use serde::Deserialize;
use std::fmt;
use std::fmt::Display;

#[derive(Deserialize, Debug, Clone)]
pub struct Transaction {
    #[serde(rename(deserialize = "type"))]
    pub transaction_type: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<Decimal>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum TransactionType {
    #[serde(rename(deserialize = "deposit"))]
    Deposit,
    #[serde(rename(deserialize = "withdrawl"))]
    Withdrawal,
    #[serde(rename(deserialize = "dispute"))]
    Dispute,
    #[serde(rename(deserialize = "resolve"))]
    Resolve,
    #[serde(rename(deserialize = "chargeback"))]
    Chargeback,
}

#[derive(Debug, Clone)]
pub enum TransactionError {
    NotEnoughFunds,
    RefferencedTransactionDoesNotExist,
    RefferencedTransactionIsNotDisputed,
    RefferencedTransactionIsDisputed,
    AccountLocked,
}

impl Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TransactionError::*;
        let text = match self {
            NotEnoughFunds => "Not enough funds to process transaction",
            RefferencedTransactionIsDisputed => "The referenced transaction is already disputed",
            RefferencedTransactionDoesNotExist => "The referenced transaction does not yet exist",
            RefferencedTransactionIsNotDisputed => "The referenced transaction is not disputed",
            AccountLocked => "The Account is locked after a chargeback",
        };
        write!(f, "{}", text)
    }
}

pub struct PersistedTransaction {
    disputed: bool,
    pub transaction: Transaction,
}
impl PersistedTransaction {
    pub fn new(t: Transaction) -> Self {
        Self {
            disputed: false,
            transaction: t,
        }
    }
    pub fn is_disputed(&self) -> bool {
        self.disputed
    }
    pub fn set_disputed(&mut self) {
        self.disputed = true;
    }
    pub fn set_not_disputed(&mut self) {
        self.disputed = false;
    }
}
