use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    Address, Env, String,
};

use crate::{Category, Error, RemitSure, RemitSureClient, Status, MAX_LOCK_SECS};

const AMOUNT: i128 = 250_0000000;
const DAY: u64 = 24 * 60 * 60;
const START: u64 = 1_700_000_000;

struct Setup<'a> {
    env: Env,
    client: RemitSureClient<'a>,
    token: TokenClient<'a>,
    admin: Address,
    sender: Address,
    school: Address,
}

fn setup() -> Setup<'static> {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = START);

    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token = TokenClient::new(&env, &sac.address());

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let school = Address::generate(&env);
    StellarAssetClient::new(&env, &sac.address()).mint(&sender, &(AMOUNT * 10));

    let contract_id = env.register(RemitSure, (admin.clone(),));
    let client = RemitSureClient::new(&env, &contract_id);

    client.register_payee(
        &school,
        &String::from_str(&env, "Greenfield College"),
        &Category::School,
    );

    Setup {
        env,
        client,
        token,
        admin,
        sender,
        school,
    }
}

fn send(s: &Setup) -> u64 {
    s.client.send(
        &s.sender,
        &s.school,
        &s.token.address,
        &AMOUNT,
        &String::from_str(&s.env, "Term 1 fees, Ada Obi, JSS2"),
        &(START + 30 * DAY),
    )
}

fn advance(s: &Setup, secs: u64) {
    s.env.ledger().with_mut(|l| l.timestamp += secs);
}

#[test]
fn send_locks_funds_in_the_contract() {
    let s = setup();
    let id = send(&s);
    assert_eq!(id, 1);
    assert_eq!(s.token.balance(&s.client.address), AMOUNT);
    assert_eq!(s.token.balance(&s.sender), AMOUNT * 9);

    let r = s.client.get_remittance(&id);
    assert_eq!(r.status, Status::Pending);
    assert_eq!(r.payee, s.school);
    assert_eq!(r.created_at, START);
    assert_eq!(s.client.remittance_count(), 1);
}

#[test]
fn payee_claims_before_deadline() {
    let s = setup();
    let id = send(&s);
    advance(&s, DAY);
    s.client.claim(&id);

    assert_eq!(s.token.balance(&s.school), AMOUNT);
    assert_eq!(s.token.balance(&s.client.address), 0);
    assert_eq!(s.client.get_remittance(&id).status, Status::Claimed);
    assert_eq!(s.client.try_claim(&id), Err(Ok(Error::NotPending)));
}

#[test]
fn payee_cannot_claim_after_deadline() {
    let s = setup();
    let id = send(&s);
    advance(&s, 30 * DAY);
    assert_eq!(s.client.try_claim(&id), Err(Ok(Error::Expired)));
}

#[test]
fn sender_refunds_only_after_deadline() {
    let s = setup();
    let id = send(&s);
    assert_eq!(s.client.try_refund(&id), Err(Ok(Error::NotExpired)));

    advance(&s, 30 * DAY);
    s.client.refund(&id);
    assert_eq!(s.token.balance(&s.sender), AMOUNT * 10);
    assert_eq!(s.client.get_remittance(&id).status, Status::Refunded);
    assert_eq!(s.client.try_claim(&id), Err(Ok(Error::NotPending)));
}

#[test]
fn reject_refunds_sender_immediately() {
    let s = setup();
    let id = send(&s);
    s.client.reject(&id);
    assert_eq!(s.token.balance(&s.sender), AMOUNT * 10);
    assert_eq!(s.client.get_remittance(&id).status, Status::Rejected);
    assert_eq!(s.client.try_refund(&id), Err(Ok(Error::NotPending)));
}

#[test]
fn send_validates_input() {
    let s = setup();
    let t = &s.token.address;
    let r = String::from_str(&s.env, "fees");
    let ok_expiry = START + DAY;

    assert_eq!(
        s.client
            .try_send(&s.sender, &s.school, t, &0, &r, &ok_expiry),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        s.client
            .try_send(&s.sender, &s.school, t, &AMOUNT, &r, &START),
        Err(Ok(Error::InvalidExpiry))
    );
    assert_eq!(
        s.client.try_send(
            &s.sender,
            &s.school,
            t,
            &AMOUNT,
            &r,
            &(START + MAX_LOCK_SECS + 1)
        ),
        Err(Ok(Error::InvalidExpiry))
    );
    assert_eq!(
        s.client
            .try_send(&s.sender, &s.sender, t, &AMOUNT, &r, &ok_expiry),
        Err(Ok(Error::SelfPayment))
    );
    let stranger = Address::generate(&s.env);
    assert_eq!(
        s.client
            .try_send(&s.sender, &stranger, t, &AMOUNT, &r, &ok_expiry),
        Err(Ok(Error::PayeeNotFound))
    );
    let long = String::from_str(&s.env, &"x".repeat(97));
    assert_eq!(
        s.client
            .try_send(&s.sender, &s.school, t, &AMOUNT, &long, &ok_expiry),
        Err(Ok(Error::InvalidReference))
    );
}

#[test]
fn payee_registration_rules() {
    let s = setup();
    assert_eq!(
        s.client.try_register_payee(
            &s.school,
            &String::from_str(&s.env, "Again"),
            &Category::School
        ),
        Err(Ok(Error::PayeeAlreadyRegistered))
    );
    let landlord = Address::generate(&s.env);
    assert_eq!(
        s.client.try_register_payee(
            &landlord,
            &String::from_str(&s.env, ""),
            &Category::Landlord
        ),
        Err(Ok(Error::InvalidName))
    );

    let p = s.client.get_payee(&s.school);
    assert_eq!(p.category, Category::School);
    assert!(!p.verified);
}

#[test]
fn admin_verifies_payees() {
    let s = setup();
    s.client.set_verified(&s.school, &true);
    assert!(s.client.get_payee(&s.school).verified);

    let unknown = Address::generate(&s.env);
    assert_eq!(
        s.client.try_set_verified(&unknown, &true),
        Err(Ok(Error::PayeeNotFound))
    );
}

#[test]
fn admin_can_be_rotated() {
    let s = setup();
    assert_eq!(s.client.admin(), s.admin);
    let next = Address::generate(&s.env);
    s.client.set_admin(&next);
    assert_eq!(s.client.admin(), next);
}

#[test]
#[should_panic]
fn only_payee_can_claim() {
    let s = setup();
    let id = send(&s);
    // Drop the blanket auth mock: the payee has not signed, so claim must fail.
    s.env.set_auths(&[]);
    s.client.claim(&id);
}

/// The backend indexer matches on these exact topic names. If this test breaks,
/// remitsure-backend/src/indexer/events.ts must change in step.
#[test]
fn event_topics_match_the_indexer() {
    extern crate std;
    use soroban_sdk::testutils::Events;
    use soroban_sdk::xdr::{ContractEventBody, ScVal};

    let s = setup();
    let last_topic = || -> std::string::String {
        let events = s.env.events().all().filter_by_contract(&s.client.address);
        let ContractEventBody::V0(body) = events.events().last().unwrap().body.clone();
        match &body.topics[0] {
            ScVal::Symbol(sym) => sym.to_utf8_string_lossy(),
            other => panic!("unexpected topic {other:?}"),
        }
    };

    let id = send(&s);
    assert_eq!(last_topic(), "remittance_sent");
    s.client.claim(&id);
    assert_eq!(last_topic(), "remittance_claimed");
}
