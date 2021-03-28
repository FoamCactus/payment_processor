use clap::{App, Arg};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
mod account;
mod test;
mod transaction;
use crate::account::{Account, AccountTransaction, AccountTransactionType};
use crate::transaction::{PersistedTransaction, Transaction, TransactionError, TransactionType};

fn main() {
    let matches = App::new("Basic TransactionProcessor")
        .version("0.1")
        .author("Caleb Hill, foamcactus@protonmail.com")
        .about("reads transactions from file and applies them to a bank/centralized store")
        .arg(
            Arg::with_name("INPUT")
                .help("sets the input file")
                .required(true)
                .index(1),
        )
        .get_matches();
    let filename = matches.value_of("INPUT").unwrap();
    if let Ok(reader) = open_file(filename.to_string()) {
        match read_csv(reader) {
            Ok(vec) => {
                let mut engine = TransactionProcessor::new();
                for t in vec {
                    if let Err(e) = engine.apply(t) {
                        println!("could not apply transaction, error: {}", e);
                    }
                }
                engine.print_totals();
            }
            Err(e) => {
                println!("error parsing records:{} ", e)
            }
        };
    }
}

fn open_file(filename: String) -> std::io::Result<BufReader<File>> {
    let file = File::open(filename)?;
    Ok(BufReader::new(file))
}

fn read_csv(reader: BufReader<File>) -> Result<Vec<Transaction>, csv::Error> {
    let mut csv_data = csv::Reader::from_reader(reader);
    csv_data.deserialize::<Transaction>().collect()
}

pub struct TransactionProcessor {
    bank: HashMap<u16, Account>,
    transactions: HashMap<u32, PersistedTransaction>,
}
impl TransactionProcessor {
    fn new() -> Self {
        Self {
            bank: HashMap::new(),
            transactions: HashMap::new(),
        }
    }
    fn apply(&mut self, transaction: Transaction) -> Result<(), TransactionError> {
        use TransactionType::*;
        let account_transaction = self.to_account_transaction(&transaction)?;
        match transaction.transaction_type {
            Deposit | Withdrawal => {
                self.transactions.insert(
                    transaction.tx,
                    PersistedTransaction::new(transaction.clone()),
                );
                Ok(())
            }
            Dispute => match self.transactions.entry(transaction.tx) {
                Vacant(_) => Err(TransactionError::RefferencedTransactionDoesNotExist),
                Occupied(entry) => {
                    let t = entry.into_mut();
                    if t.is_disputed() {
                        Err(TransactionError::RefferencedTransactionIsDisputed)
                    } else {
                        t.set_disputed();
                        Ok(())
                    }
                }
            },
            Resolve => match self.transactions.entry(transaction.tx) {
                Vacant(_) => Err(TransactionError::RefferencedTransactionDoesNotExist),
                Occupied(entry) => {
                    let t = entry.into_mut();
                    if t.is_disputed() {
                        t.set_not_disputed();
                        Ok(())
                    } else {
                        Err(TransactionError::RefferencedTransactionIsNotDisputed)
                    }
                }
            },
            Chargeback => match self.transactions.entry(transaction.tx) {
                Vacant(_) => Err(TransactionError::RefferencedTransactionDoesNotExist),
                Occupied(entry) => {
                    let t = entry.into_mut();
                    if t.is_disputed() {
                        t.set_not_disputed();
                        Ok(())
                    } else {
                        Err(TransactionError::RefferencedTransactionIsNotDisputed)
                    }
                }
            },
        }?;
        let account = self
            .bank
            .entry(transaction.client)
            .or_insert(Account::new(transaction.client));
        account.apply(account_transaction)?;
        Ok(())
    }
    fn to_account_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<AccountTransaction, TransactionError> {
        use TransactionType::*;
        //calling unwrap on amount should be safe here because I am only "persisting" Withdrawals
        //and deposits
        match transaction.transaction_type {
            Deposit => Ok(AccountTransaction::new(
                AccountTransactionType::Deposit,
                transaction.amount.unwrap(),
            )),
            Withdrawal => Ok(AccountTransaction::new(
                AccountTransactionType::Withdrawal,
                transaction.amount.unwrap(),
            )),
            Dispute => {
                if let Some(ref_tran) = self.transactions.get(&transaction.tx) {
                    Ok(AccountTransaction::new(
                        AccountTransactionType::AddHold,
                        ref_tran.transaction.amount.unwrap(),
                    ))
                } else {
                    Err(TransactionError::RefferencedTransactionDoesNotExist)
                }
            }
            Resolve => {
                if let Some(ref_tran) = self.transactions.get(&transaction.tx) {
                    Ok(AccountTransaction::new(
                        AccountTransactionType::NegHold,
                        ref_tran.transaction.amount.unwrap(),
                    ))
                } else {
                    Err(TransactionError::RefferencedTransactionDoesNotExist)
                }
            }
            Chargeback => {
                if let Some(ref_tran) = self.transactions.get(&transaction.tx) {
                    Ok(AccountTransaction::new(
                        AccountTransactionType::Chargeback,
                        ref_tran.transaction.amount.unwrap(),
                    ))
                } else {
                    Err(TransactionError::RefferencedTransactionDoesNotExist)
                }
            }
        }
    }

    fn print_totals(&self) {
        println!("client,available,held,total,locked");
        for account in self.bank.values() {
            println!("{}", account);
        }
    }
}
