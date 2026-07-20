use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Category {
    School,
    Landlord,
    Hospital,
    Utility,
    Other,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payee {
    pub address: Address,
    /// Display name, for example "Greenfield College, Ibadan".
    pub name: String,
    pub category: Category,
    /// Set by the admin after an off chain check. Unverified payees can still receive.
    pub verified: bool,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// Funds locked, waiting for the payee.
    Pending,
    /// Payee collected the funds.
    Claimed,
    /// Payee turned it down and the sender was refunded.
    Rejected,
    /// Deadline passed and the sender took the funds back.
    Refunded,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Remittance {
    pub id: u64,
    pub sender: Address,
    pub payee: Address,
    pub token: Address,
    /// In the token's smallest unit.
    pub amount: i128,
    /// What the money is for, for example "Term 1 fees, Ada Obi, JSS2".
    pub reference: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub status: Status,
}
