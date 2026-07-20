use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    RemittanceNotFound = 1,
    PayeeNotFound = 2,
    PayeeAlreadyRegistered = 3,
    InvalidName = 4,
    InvalidAmount = 5,
    InvalidReference = 6,
    InvalidExpiry = 7,
    SelfPayment = 8,
    NotPending = 9,
    Expired = 10,
    NotExpired = 11,
}
