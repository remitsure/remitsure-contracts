use soroban_sdk::{contracttype, Address, Env};

use crate::{Error, Payee, Remittance};

/// Roughly 30 days at 5 second ledgers.
const TTL_THRESHOLD: u32 = 518_400;
/// Roughly 60 days at 5 second ledgers.
const TTL_EXTEND_TO: u32 = 1_036_800;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    RemittanceCount,
    Remittance(u64),
    Payee(Address),
}

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

pub fn admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
    bump_instance(env);
}

pub fn remittance_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::RemittanceCount)
        .unwrap_or(0)
}

/// Ids start at 1 so 0 can never be mistaken for a real remittance.
pub fn next_remittance_id(env: &Env) -> u64 {
    let id = remittance_count(env) + 1;
    env.storage().instance().set(&DataKey::RemittanceCount, &id);
    bump_instance(env);
    id
}

pub fn load_remittance(env: &Env, id: u64) -> Result<Remittance, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Remittance(id))
        .ok_or(Error::RemittanceNotFound)
}

pub fn save_remittance(env: &Env, r: &Remittance) {
    let key = DataKey::Remittance(r.id);
    env.storage().persistent().set(&key, r);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
}

pub fn load_payee(env: &Env, address: &Address) -> Option<Payee> {
    env.storage()
        .persistent()
        .get(&DataKey::Payee(address.clone()))
}

pub fn save_payee(env: &Env, payee: &Payee) {
    let key = DataKey::Payee(payee.address.clone());
    env.storage().persistent().set(&key, payee);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
}
