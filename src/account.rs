use crate::transaction::TransactionError;
use rust_decimal::prelude::*;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub struct Account {
    pub client: u16,
    available: Decimal,
    held: Decimal,
    locked: bool,
}
impl Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},{},{},{},{}",
            self.client,
            self.available,
            self.held,
            self.available + self.held,
            self.locked
        )
    }
}
impl Account {
    pub fn apply(&mut self, transaction: AccountTransaction) -> Result<(), TransactionError> {
        use AccountTransactionType::*;
        if self.locked {
            Err(TransactionError::AccountLocked)
        } else {
            match transaction.transaction_type {
                Deposit => {
                    self.available += transaction.amount;
                    Ok(())
                }
                Withdrawal => {
                    if self.available >= transaction.amount {
                        self.available -= transaction.amount;
                        Ok(())
                    } else {
                        Err(TransactionError::NotEnoughFunds)
                    }
                }
                AddHold => {
                    self.held += transaction.amount;
                    self.available -= transaction.amount;
                    Ok(())
                }
                NegHold => {
                    self.held -= transaction.amount;
                    self.available += transaction.amount;
                    Ok(())
                }
                Chargeback => {
                    self.held -= transaction.amount;
                    self.locked = true;
                    Ok(())
                }
            }
        }
    }
    pub fn new(client_id: u16) -> Self {
        Self {
            client: client_id,
            available: Decimal::new(0, 0),
            held: Decimal::new(0, 0),
            locked: false,
        }
    }
    pub fn get_available(&self) -> Decimal {
        self.available
    }
    pub fn get_held(&self) -> Decimal {
        self.held
    }
}

pub struct AccountTransaction {
    transaction_type: AccountTransactionType,
    amount: Decimal,
}
impl AccountTransaction {
    pub fn new(transaction_type: AccountTransactionType, amount: Decimal) -> Self {
        Self {
            transaction_type,
            amount,
        }
    }
}

#[derive(Debug)]
pub enum AccountTransactionType {
    Deposit,
    Withdrawal,
    AddHold,
    NegHold,
    Chargeback,
}
