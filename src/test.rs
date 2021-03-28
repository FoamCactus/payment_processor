#[cfg(test)]

mod test {
    use crate::{Transaction, TransactionError, TransactionProcessor, TransactionType};
    use rust_decimal::Decimal;

    #[test]
    fn apply_one() {
        let mut engine = TransactionProcessor::new();
        let trans = Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(Decimal::from(1)),
        };
        engine.apply(trans);
        let account = engine.bank.get(&1);
        assert!(account.is_some());
        assert!(account.unwrap().available == Decimal::from(1));
        assert!(account.unwrap().held == Decimal::from(0));
        let trans_count = engine.transactions.iter().count();
        assert!(trans_count == 1);
    }

    #[test]
    fn dispute_transaction() {
        let mut engine = TransactionProcessor::new();
        let trans = Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(Decimal::from(1)),
        };
        let dispute = Transaction {
            transaction_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        };
        engine.apply(trans).expect("could not deposit funds");
        engine
            .apply(dispute)
            .expect("could not dispute transaction");
        let account = engine.bank.get(&1);
        println!("{:?}", account);
        assert!(account.is_some());
        assert_eq!(account.unwrap().available, Decimal::from(0));
        assert_eq!(account.unwrap().held, Decimal::from(1));
    }

    #[test]
    fn chargeback_locks() {
        let mut engine = TransactionProcessor::new();
        let trans = Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(Decimal::from(1)),
        };
        let dispute = Transaction {
            transaction_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        };
        let chargeback = Transaction {
            transaction_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
        };
        let tran_two = Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(Decimal::from(1)),
        };
        engine.apply(trans).expect("could not deposit funds");
        engine
            .apply(dispute)
            .expect("could not dispute transaction");
        engine
            .apply(chargeback)
            .expect("chargeback can't go through");
        let rejects = match engine.apply(tran_two) {
            Err(TransactionError::AccountLocked) => true,
            _ => false,
        };
        assert!(rejects);
    }
}
