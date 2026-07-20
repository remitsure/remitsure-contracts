//! Events are the contract's public API for the backend indexer.
//! Changing a field here is a breaking change for remitsure-backend.

use soroban_sdk::{contractevent, Address, String};

use crate::Category;

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminChanged {
    pub old_admin: Address,
    pub new_admin: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayeeRegistered {
    #[topic]
    pub payee: Address,
    pub name: String,
    pub category: Category,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayeeVerified {
    #[topic]
    pub payee: Address,
    pub verified: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemittanceSent {
    #[topic]
    pub id: u64,
    pub sender: Address,
    pub payee: Address,
    pub token: Address,
    pub amount: i128,
    pub reference: String,
    pub expires_at: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemittanceClaimed {
    #[topic]
    pub id: u64,
    pub payee: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemittanceRejected {
    #[topic]
    pub id: u64,
    pub payee: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemittanceRefunded {
    #[topic]
    pub id: u64,
    pub sender: Address,
    pub amount: i128,
}
