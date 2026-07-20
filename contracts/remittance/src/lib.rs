#![no_std]

//! RemitSure purpose locked remittance contract.
//!
//! A sender locks funds for one named payee (a school, landlord, hospital or utility).
//! Only that payee can claim them, and only before the deadline. The payee can also
//! reject a payment, which refunds the sender at once. After the deadline the sender
//! can take unclaimed funds back. Every outcome is an event, which is the receipt.

mod errors;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, token, Address, Env, String};

pub use errors::Error;
pub use types::{Category, Payee, Remittance, Status};

pub const MAX_NAME_LEN: u32 = 64;
pub const MAX_REFERENCE_LEN: u32 = 96;
/// Deadlines further out than this are almost always a typo.
pub const MAX_LOCK_SECS: u64 = 366 * 24 * 60 * 60;

#[contract]
pub struct RemitSure;

#[contractimpl]
impl RemitSure {
    pub fn __constructor(env: Env, admin: Address) {
        storage::set_admin(&env, &admin);
    }

    pub fn admin(env: Env) -> Address {
        storage::admin(&env)
    }

    pub fn set_admin(env: Env, new_admin: Address) {
        let admin = storage::admin(&env);
        admin.require_auth();
        storage::set_admin(&env, &new_admin);
        events::AdminChanged {
            old_admin: admin,
            new_admin,
        }
        .publish(&env);
    }

    /// A payee registers itself. It starts unverified until the admin checks it.
    pub fn register_payee(
        env: Env,
        payee: Address,
        name: String,
        category: Category,
    ) -> Result<(), Error> {
        payee.require_auth();

        if name.is_empty() || name.len() > MAX_NAME_LEN {
            return Err(Error::InvalidName);
        }
        if storage::load_payee(&env, &payee).is_some() {
            return Err(Error::PayeeAlreadyRegistered);
        }

        storage::save_payee(
            &env,
            &Payee {
                address: payee.clone(),
                name: name.clone(),
                category,
                verified: false,
            },
        );

        events::PayeeRegistered {
            payee,
            name,
            category,
        }
        .publish(&env);
        Ok(())
    }

    /// Admin marks a payee as checked (or removes the badge).
    pub fn set_verified(env: Env, payee: Address, verified: bool) -> Result<(), Error> {
        storage::admin(&env).require_auth();
        let mut record = storage::load_payee(&env, &payee).ok_or(Error::PayeeNotFound)?;
        record.verified = verified;
        storage::save_payee(&env, &record);

        events::PayeeVerified { payee, verified }.publish(&env);
        Ok(())
    }

    /// Lock `amount` of `token` for `payee` until `expires_at` (unix seconds).
    pub fn send(
        env: Env,
        sender: Address,
        payee: Address,
        token: Address,
        amount: i128,
        reference: String,
        expires_at: u64,
    ) -> Result<u64, Error> {
        sender.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if reference.len() > MAX_REFERENCE_LEN {
            return Err(Error::InvalidReference);
        }
        if sender == payee {
            return Err(Error::SelfPayment);
        }
        let now = env.ledger().timestamp();
        if expires_at <= now || expires_at - now > MAX_LOCK_SECS {
            return Err(Error::InvalidExpiry);
        }
        if storage::load_payee(&env, &payee).is_none() {
            return Err(Error::PayeeNotFound);
        }

        token::Client::new(&env, &token).transfer(&sender, env.current_contract_address(), &amount);

        let id = storage::next_remittance_id(&env);
        storage::save_remittance(
            &env,
            &Remittance {
                id,
                sender: sender.clone(),
                payee: payee.clone(),
                token: token.clone(),
                amount,
                reference: reference.clone(),
                created_at: now,
                expires_at,
                status: Status::Pending,
            },
        );

        events::RemittanceSent {
            id,
            sender,
            payee,
            token,
            amount,
            reference,
            expires_at,
        }
        .publish(&env);
        Ok(id)
    }

    /// Payee collects the funds before the deadline.
    pub fn claim(env: Env, id: u64) -> Result<(), Error> {
        let mut r = storage::load_remittance(&env, id)?;
        r.payee.require_auth();

        if r.status != Status::Pending {
            return Err(Error::NotPending);
        }
        if env.ledger().timestamp() >= r.expires_at {
            return Err(Error::Expired);
        }

        token::Client::new(&env, &r.token).transfer(
            &env.current_contract_address(),
            &r.payee,
            &r.amount,
        );
        r.status = Status::Claimed;
        storage::save_remittance(&env, &r);

        events::RemittanceClaimed {
            id,
            payee: r.payee,
            amount: r.amount,
        }
        .publish(&env);
        Ok(())
    }

    /// Payee turns a payment down (wrong student, wrong amount). Sender is refunded now.
    pub fn reject(env: Env, id: u64) -> Result<(), Error> {
        let mut r = storage::load_remittance(&env, id)?;
        r.payee.require_auth();

        if r.status != Status::Pending {
            return Err(Error::NotPending);
        }

        token::Client::new(&env, &r.token).transfer(
            &env.current_contract_address(),
            &r.sender,
            &r.amount,
        );
        r.status = Status::Rejected;
        storage::save_remittance(&env, &r);

        events::RemittanceRejected {
            id,
            payee: r.payee,
            amount: r.amount,
        }
        .publish(&env);
        Ok(())
    }

    /// Sender takes back funds the payee did not claim in time.
    pub fn refund(env: Env, id: u64) -> Result<(), Error> {
        let mut r = storage::load_remittance(&env, id)?;
        r.sender.require_auth();

        if r.status != Status::Pending {
            return Err(Error::NotPending);
        }
        if env.ledger().timestamp() < r.expires_at {
            return Err(Error::NotExpired);
        }

        token::Client::new(&env, &r.token).transfer(
            &env.current_contract_address(),
            &r.sender,
            &r.amount,
        );
        r.status = Status::Refunded;
        storage::save_remittance(&env, &r);

        events::RemittanceRefunded {
            id,
            sender: r.sender,
            amount: r.amount,
        }
        .publish(&env);
        Ok(())
    }

    pub fn get_remittance(env: Env, id: u64) -> Result<Remittance, Error> {
        storage::load_remittance(&env, id)
    }

    pub fn get_payee(env: Env, payee: Address) -> Result<Payee, Error> {
        storage::load_payee(&env, &payee).ok_or(Error::PayeeNotFound)
    }

    pub fn remittance_count(env: Env) -> u64 {
        storage::remittance_count(&env)
    }
}
