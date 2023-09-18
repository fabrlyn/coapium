use crate::codec::{MessageId, Token};

use super::transaction::{Transaction, NSTART};

#[derive(Debug)]
pub struct TransactionStore {
    nstart: usize,
    transactions: Vec<Transaction>,
}

impl TransactionStore {
    pub fn new(nstart: usize) -> Self {
        Self {
            nstart,
            transactions: vec![],
        }
    }

    pub fn count(&self) -> usize {
        self.transactions.len()
    }

    pub fn add(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn find_by_message_id(&mut self, message_id: &MessageId) -> Option<&Transaction> {
        self.transactions
            .iter()
            .find(|t| t.message_id() == *message_id)
    }

    pub fn find_mut_by_message_id(&mut self, message_id: &MessageId) -> Option<&mut Transaction> {
        self.transactions
            .iter_mut()
            .find(|t| t.message_id() == *message_id)
    }

    pub fn find_by_token(&mut self, token: &Token) -> Option<&Transaction> {
        self.transactions.iter().find(|t| t.token() == token)
    }

    pub fn exists_by_token(&mut self, token: &Token) -> bool {
        self.find_by_token(token).is_some()
    }

    pub fn remove_by_message_id(&mut self, message_id: &MessageId) -> Option<Transaction> {
        let Some(position) = self
            .transactions
            .iter()
            .position(Self::compare_message_id(message_id))
        else {
            return None;
        };

        Some(self.transactions.swap_remove(position))
    }

    pub fn remove_by_token(&mut self, token: &Token) -> Option<Transaction> {
        let Some(position) = self.transactions.iter().position(|t| t.token() == token) else {
            return None;
        };

        Some(self.transactions.swap_remove(position))
    }

    pub fn current_nstart(&self) -> usize {
        self.transactions
            .iter()
            .filter(|t| t.is_non_confirmable() || t.is_acknowledged())
            .count()
    }

    pub fn at_max_inflight_capacity(&self) -> bool {
        self.current_nstart() >= self.nstart
    }

    fn compare_message_id<'a>(right: &'a MessageId) -> impl FnMut(&'a Transaction) -> bool {
        move |left| left.message_id() == *right
    }
}

impl Default for TransactionStore {
    fn default() -> Self {
        Self::new(NSTART)
    }
}
