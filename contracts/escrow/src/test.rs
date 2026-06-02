#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger},
    token, Address, BytesN, Env, IntoVal, String as SorobanString, Symbol, TryFromVal, Val,
};
use crate::{
    AutoReleased, ContractError, DisputeRaised, DisputeResolved, Escrow, EscrowClient,
    EscrowCompleted, EscrowCreated, EscrowFunded, EscrowState, ResolutionType,
};

fn setup_env() -> (Env, Address, Address, Address, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let resolver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let fee_collector = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract(token_admin.clone());

    (env, admin, seller, buyer, resolver, token_address, fee_collector)
}

fn mint_tokens(env: &Env, token: &Address, to: &Address, amount: i128) {
    let sac = token::StellarAssetClient::new(env, token);
    sac.mint(to, &amount);
}

fn get_balance(env: &Env, token: &Address, user: &Address) -> i128 {
    let tc = token::Client::new(env, token);
    tc.balance(user)
}

fn register_alt_token(env: &Env) -> (Address, Address) {
    let admin = Address::generate(env);
    let token_address = env.register_stellar_asset_contract(admin.clone());
    (token_address, admin)
}

fn has_event<T, F>(env: &Env, contract_id: &Address, topic: &str, predicate: F) -> bool
where
    T: TryFromVal<Env, Val>,
    F: Fn(&T) -> bool,
{
    let expected_topic = Symbol::new(env, topic);
    env.events()
        .all()
        .filter_by_contract(contract_id)
        .events()
        .iter()
        .any(|event| match &event.body {
            soroban_sdk::xdr::ContractEventBody::V0(v0) => {
                let Some(topic) = v0.topics.iter().next() else {
                    return false;
                };
                let Ok(topic) = Symbol::try_from_val(env, topic) else {
                    return false;
                };
                if topic != expected_topic {
                    return false;
                }
                let Ok(data) = Val::try_from_val(env, &v0.data) else {
                    return false;
                };
                T::try_from_val(env, &data)
                    .map(|event| predicate(&event))
                    .unwrap_or(false)
            }
            _ => false,
        })
}

#[test]
fn test_create_escrow() {
<<<<<<< HEAD
    let (env, admin, seller, _buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
=======
    let (env, admin, seller, _buyer, resolver, _token_admin, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);
>>>>>>> 7c0b9a0 (feat: implement issues #205, #206, #209)

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &100_i128, &200_u32, &3600_u64);
    assert_eq!(id, 1u32);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.seller, seller);
    assert_eq!(escrow.resolver, resolver);
    assert_eq!(escrow.token, token);
    assert_eq!(escrow.amount, 100);
    assert_eq!(escrow.fee_bps, 200);
    assert_eq!(escrow.shipping_window, 3600);
    assert_eq!(escrow.state, EscrowState::Pending);
    assert!(escrow.buyer.is_none());
}

#[test]
fn test_fund_escrow() {
<<<<<<< HEAD
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
=======
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);
>>>>>>> 7c0b9a0 (feat: implement issues #205, #206, #209)

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &100_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Funded);
    assert_eq!(get_balance(&env, &token, &buyer), 900);
    assert_eq!(get_balance(&env, &token, &contract_id), 100);
}

#[test]
fn test_confirm_delivery() {
<<<<<<< HEAD
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
    client.set_protocol_fee(&admin, &200_u32);
=======
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);
>>>>>>> 7c0b9a0 (feat: implement issues #205, #206, #209)

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-010"));

    let escrow = client.get_escrow(&id);
    env.ledger().set_timestamp(escrow.dispute_deadline + 1);
    client.confirm_delivery(&buyer, &id);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Completed);
    assert_eq!(get_balance(&env, &token, &seller), 980);
    assert_eq!(get_balance(&env, &token, &fee_collector), 20);
    assert_eq!(get_balance(&env, &token, &contract_id), 0);
}

#[test]
fn test_raise_and_resolve_dispute_release_to_seller() {
<<<<<<< HEAD
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
=======
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);
>>>>>>> 7c0b9a0 (feat: implement issues #205, #206, #209)

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-DISPUTE-1"));
    client.raise_dispute(
        &buyer,
        &id,
        &Symbol::new(&env, "reason"),
        &SorobanString::from_str(&env, "desc"),
        &BytesN::from_array(&env, &[0u8; 32]),
    );

    client.resolve_dispute(&resolver, &id, &ResolutionType::Release);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Completed);
    assert_eq!(get_balance(&env, &token, &seller), 980);
    assert_eq!(get_balance(&env, &token, &contract_id), 20);
}

#[test]
fn test_raise_and_resolve_dispute_refund_buyer() {
<<<<<<< HEAD
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
=======
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);
>>>>>>> 7c0b9a0 (feat: implement issues #205, #206, #209)

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-DISPUTE-2"));
    client.raise_dispute(
        &buyer,
        &id,
        &Symbol::new(&env, "reason"),
        &SorobanString::from_str(&env, "desc"),
        &BytesN::from_array(&env, &[0u8; 32]),
    );
    client.resolve_dispute(&resolver, &id, &ResolutionType::Refund);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Refunded);
    assert_eq!(get_balance(&env, &token, &buyer), 980);
    assert_eq!(get_balance(&env, &token, &contract_id), 20);
}

#[test]
fn test_auto_release() {
<<<<<<< HEAD
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
    client.set_protocol_fee(&admin, &200_u32);
=======
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);
>>>>>>> 7c0b9a0 (feat: implement issues #205, #206, #209)

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-AUTO-1"));
    env.ledger().set_timestamp(1_700_000_000);
    client.record_delivery(&admin, &id);

    let escrow = client.get_escrow(&id);
    env.ledger().set_timestamp(escrow.delivered_at.unwrap() + 172_801);
    client.auto_release(&id);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Completed);
    assert_eq!(get_balance(&env, &token, &seller), 980);
    assert_eq!(get_balance(&env, &token, &fee_collector), 20);
    assert_eq!(get_balance(&env, &token, &contract_id), 0);
}

#[test]
fn test_fund_non_pending_escrow_fails() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
    mint_tokens(&env, &token, &buyer, 1000);
    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &100_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    let res = client.try_fund_escrow(&id, &buyer);
    assert!(matches!(res, Err(Ok(ContractError::InvalidState))));
}

#[test]
fn test_auto_release_before_window_fails() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
    client.set_protocol_fee(&admin, &200_u32);
    mint_tokens(&env, &token, &buyer, 1000);
    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &100_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-AUTO-2"));
    env.ledger().set_timestamp(1_700_000_000);
    client.record_delivery(&admin, &id);
    let escrow = client.get_escrow(&id);
    env.ledger().set_timestamp(escrow.delivered_at.unwrap() + 1);
    let res = client.try_auto_release(&id);
    assert!(matches!(res, Err(Ok(ContractError::ShippingWindowNotElapsed))));
}

#[test]
fn test_raise_dispute_invalid_evidence_hash_rejected() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &100_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-BAD-HASH"));

    let short_hash = BytesN::from_array(&env, &[0u8; 16]);
    let res = client.try_raise_dispute(
        &buyer,
        &id,
        &Symbol::new(&env, "reason"),
        &SorobanString::from_str(&env, "desc"),
        &short_hash,
    );
    assert!(matches!(res, Err(Ok(ContractError::InvalidEvidenceHash))));
}

#[test]
fn test_raise_dispute_only_once() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
    mint_tokens(&env, &token, &buyer, 1000);
    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &100_i128, &0_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-DISPUTE-3"));
    client.raise_dispute(
        &buyer,
        &id,
        &Symbol::new(&env, "reason"),
        &SorobanString::from_str(&env, "desc"),
        &BytesN::from_array(&env, &[0u8; 32]),
    );
    let res = client.try_raise_dispute(
        &buyer,
        &id,
        &Symbol::new(&env, "reason"),
        &SorobanString::from_str(&env, "desc"),
        &BytesN::from_array(&env, &[0u8; 32]),
    );
    assert!(matches!(res, Err(Ok(ContractError::InvalidState))));
}

#[test]
fn test_multiple_escrows() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
    mint_tokens(&env, &token, &buyer, 2000);
    let id1 = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &100_i128, &200_u32, &3600_u64);
    let id2 = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &200_i128, &200_u32, &7200_u64);
    assert_eq!(id1, 1u32);
    assert_eq!(id2, 2u32);
}

#[test]
fn test_create_escrow_with_non_usdc_token() {
    let (env, admin, seller, _buyer, resolver, _token, fee_collector) = setup_env();
    let (alt_token, _) = register_alt_token(&env);
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &alt_token, &500_i128, &0_u32, &7200_u64);
    assert_eq!(id, 1u32);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.token, alt_token);
    assert_eq!(escrow.amount, 500);
    assert_eq!(escrow.shipping_window, 7200);
    assert_eq!(escrow.state, EscrowState::Pending);
    assert!(escrow.buyer.is_none());
}

#[test]
fn test_fund_and_confirm_delivery_with_non_usdc_token() {
    let (env, admin, seller, buyer, resolver, _token, fee_collector) = setup_env();
    let (alt_token, _) = register_alt_token(&env);
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
    client.set_protocol_fee(&admin, &100_u32);
    mint_tokens(&env, &alt_token, &buyer, 1000);
    let id = client.create_escrow(
        &seller, &None::<Address>, &resolver, &alt_token, &300_i128, &100_u32, &3600_u64,
    );
    client.fund_escrow(&id, &buyer);

    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-SEP41"));

    let escrow = client.get_escrow(&id);
    env.ledger().set_timestamp(escrow.dispute_deadline + 1);
    client.confirm_delivery(&buyer, &id);
    assert_eq!(get_balance(&env, &alt_token, &seller), 297);
    assert_eq!(get_balance(&env, &alt_token, &fee_collector), 3);
    assert_eq!(get_balance(&env, &alt_token, &contract_id), 0);
}

#[test]
fn test_dispute_resolved_to_seller_with_non_usdc_token() {
    let (env, admin, seller, buyer, resolver, _token, _fee_collector) = setup_env();
    let (alt_token, _) = register_alt_token(&env);
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &_fee_collector, &0_u32);

    mint_tokens(&env, &alt_token, &buyer, 1_000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &alt_token, &400_i128, &0_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-SEP41-DISPUTE"));
    client.raise_dispute(
        &buyer,
        &id,
        &Symbol::new(&env, "reason"),
        &SorobanString::from_str(&env, "desc"),
        &BytesN::from_array(&env, &[0u8; 32]),
    );

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Disputed);

    client.resolve_dispute(&resolver, &id, &ResolutionType::Release);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Completed);
    assert_eq!(get_balance(&env, &alt_token, &seller), 400);
    assert_eq!(get_balance(&env, &alt_token, &contract_id), 0);
}

#[test]
fn test_dispute_refunded_to_buyer_with_non_usdc_token() {
    let (env, admin, seller, buyer, resolver, _token, _fee_collector) = setup_env();
    let (alt_token, _) = register_alt_token(&env);
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &_fee_collector, &0_u32);

    mint_tokens(&env, &alt_token, &buyer, 1_000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &alt_token, &400_i128, &0_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-SEP41-REFUND"));
    client.raise_dispute(
        &buyer,
        &id,
        &Symbol::new(&env, "reason"),
        &SorobanString::from_str(&env, "desc"),
        &BytesN::from_array(&env, &[0u8; 32]),
    );
    client.resolve_dispute(&resolver, &id, &ResolutionType::Refund);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Refunded);
    assert_eq!(get_balance(&env, &alt_token, &buyer), 1_000);
    assert_eq!(get_balance(&env, &alt_token, &seller), 0);
    assert_eq!(get_balance(&env, &alt_token, &contract_id), 0);
}

#[test]
fn test_auto_release_with_non_usdc_token() {
    let (env, admin, seller, buyer, resolver, _token, _fee_collector) = setup_env();
    let (alt_token, _) = register_alt_token(&env);
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &_fee_collector, &0_u32);

    mint_tokens(&env, &alt_token, &buyer, 1_000);

    let shipping_window: u64 = 86_400;
    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &alt_token, &250_i128, &0_u32, &shipping_window);
    client.fund_escrow(&id, &buyer);
    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-SEP41-AUTO"));
    env.ledger().set_timestamp(1_700_000_000);
    client.record_delivery(&admin, &id);

    let escrow = client.get_escrow(&id);
    env.ledger().set_timestamp(escrow.delivered_at.unwrap() + 172_801);

    client.auto_release(&id);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Completed);
    assert_eq!(get_balance(&env, &alt_token, &seller), 250);
    assert_eq!(get_balance(&env, &alt_token, &contract_id), 0);
}

#[test]
fn test_multi_asset_concurrent_escrows_different_tokens() {
    let (env, admin, seller, buyer_a, resolver, _token, _fee_collector) = setup_env();
    let buyer_b = Address::generate(&env);
    let (token_a, _admin_a) = register_alt_token(&env);
    let (token_b, _admin_b) = register_alt_token(&env);

    assert_ne!(token_a, token_b);

    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &_fee_collector, &0_u32);

    mint_tokens(&env, &token_a, &buyer_a, 1_000);
    mint_tokens(&env, &token_b, &buyer_b, 2_000);

    let id1 = client.create_escrow(&seller, &None::<Address>, &resolver, &token_a, &150_i128, &0_u32, &3600_u64);
    let id2 = client.create_escrow(&seller, &None::<Address>, &resolver, &token_b, &500_i128, &0_u32, &3600_u64);

    assert_eq!(id1, 1u32);
    assert_eq!(id2, 2u32);

    client.fund_escrow(&id1, &buyer_a);
    client.fund_escrow(&id2, &buyer_b);

    assert_eq!(get_balance(&env, &token_a, &buyer_a), 850);
    assert_eq!(get_balance(&env, &token_b, &buyer_b), 1_500);
    assert_eq!(get_balance(&env, &token_a, &contract_id), 150);
    assert_eq!(get_balance(&env, &token_b, &contract_id), 500);

    client.mark_shipped(&seller, &id1, &SorobanString::from_str(&env, "TRK-A"));
    let escrow1 = client.get_escrow(&id1);
    env.ledger().set_timestamp(escrow1.dispute_deadline + 1);
    client.confirm_delivery(&buyer_a, &id1);

    client.mark_shipped(&seller, &id2, &SorobanString::from_str(&env, "TRK-B"));
    client.raise_dispute(
        &buyer_b,
        &id2,
        &Symbol::new(&env, "reason"),
        &SorobanString::from_str(&env, "desc"),
        &BytesN::from_array(&env, &[0u8; 32]),
    );
    client.resolve_dispute(&resolver, &id2, &ResolutionType::Refund);

    let escrow1 = client.get_escrow(&id1);
    let escrow2 = client.get_escrow(&id2);
    assert_eq!(escrow1.state, EscrowState::Completed);
    assert_eq!(escrow2.state, EscrowState::Refunded);

    assert_eq!(get_balance(&env, &token_a, &seller), 150);
    assert_eq!(get_balance(&env, &token_a, &contract_id), 0);

    assert_eq!(get_balance(&env, &token_b, &buyer_b), 2_000);
    assert_eq!(get_balance(&env, &token_b, &seller), 0);
    assert_eq!(get_balance(&env, &token_b, &contract_id), 0);
}

#[test]
fn test_sequential_escrows_same_non_usdc_token() {
    let (env, admin, seller, buyer, resolver, _token, _fee_collector) = setup_env();
    let (alt_token, _alt_admin) = register_alt_token(&env);

    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &_fee_collector, &0_u32);

    mint_tokens(&env, &alt_token, &buyer, 5_000);

    for (i, amount) in [100_i128, 200_i128, 300_i128].iter().enumerate() {
        let expected_id = (i as u32) + 1;
        let id = client.create_escrow(&seller, &None::<Address>, &resolver, &alt_token, amount, &0_u32, &3600_u64);
        assert_eq!(id, expected_id);

        client.fund_escrow(&id, &buyer);
        client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-SEQ"));
        let escrow = client.get_escrow(&id);
        env.ledger().set_timestamp(escrow.dispute_deadline + 1);
        client.confirm_delivery(&buyer, &id);

        let escrow = client.get_escrow(&id);
        assert_eq!(escrow.state, EscrowState::Completed);
        assert_eq!(escrow.token, alt_token);
    }

    assert_eq!(get_balance(&env, &alt_token, &seller), 600);
    assert_eq!(get_balance(&env, &alt_token, &buyer), 4_400);
    assert_eq!(get_balance(&env, &alt_token, &contract_id), 0);
}

#[test]
fn test_zero_fee_no_collector_transfer() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
    mint_tokens(&env, &token, &buyer, 1000);
    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &0_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);

    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-ZERO"));

    let escrow = client.get_escrow(&id);
    env.ledger().set_timestamp(escrow.dispute_deadline + 1);
    client.confirm_delivery(&buyer, &id);
    assert_eq!(get_balance(&env, &token, &seller), 1000);
    assert_eq!(get_balance(&env, &token, &fee_collector), 0);
    assert_eq!(get_balance(&env, &token, &contract_id), 0);
}

#[test]
fn test_get_fee_config() {
    let (env, _, _, _, _, _, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &fee_collector, &0_u32);
    let config = client.get_fee_config();
    assert_eq!(config.protocol_fee_bps, 0);
    assert_eq!(config.arbitration_fee_bps, 0);
}

#[test]
fn test_fee_exceeds_max_bps_fails() {
    let (env, _, seller, _, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &fee_collector, &0_u32);
    let res = client.try_create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &301_u32, &3600_u64);
    assert!(matches!(res, Err(Ok(ContractError::FeeExceedsMax))));
}

#[test]
fn test_dispute_after_shipping_succeeds() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-DISPUTE-4"));

    client.raise_dispute(
        &buyer,
        &id,
        &Symbol::new(&env, "reason"),
        &SorobanString::from_str(&env, "desc"),
        &BytesN::from_array(&env, &[0u8; 32]),
    );

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Disputed);
}

#[test]
fn test_dispute_requires_shipped_state() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);

    // After issue #200 fix, disputes can now be raised from Funded state
    let res = client.try_raise_dispute(
        &buyer,
        &id,
        &Symbol::new(&env, "reason"),
        &SorobanString::from_str(&env, "desc"),
        &BytesN::from_array(&env, &[0u8; 32]),
    );
    // Should succeed now that Funded -> Disputed transition is allowed
    assert!(res.is_ok());
    
    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Disputed);
}

#[test]
fn test_auto_release_after_dispute_deadline() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
    client.set_protocol_fee(&admin, &200_u32);

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRACK-AUTO-3"));
    env.ledger().set_timestamp(1_700_000_000);
    client.record_delivery(&admin, &id);

    let escrow = client.get_escrow(&id);
    let delivered_at = escrow.delivered_at.unwrap();

    env.ledger().set_timestamp(delivered_at + 172_801);

    client.auto_release(&id);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Completed);
    assert_eq!(get_balance(&env, &token, &seller), 980);
    assert_eq!(get_balance(&env, &token, &fee_collector), 20);
}

#[test]
<<<<<<< HEAD
fn test_fee_change_does_not_affect_funded_escrow() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
=======
#[should_panic(expected = "escrow not pending")]
fn test_fund_non_pending_escrow_fails() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);
>>>>>>> 7c0b9a0 (feat: implement issues #205, #206, #209)

    mint_tokens(&env, &token, &buyer, 1_000_000);

    let escrow_amount = 1_000_000_i128;
    let escrow_id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &escrow_amount, &100_u32, &3600_u64);
    client.fund_escrow(&escrow_id, &buyer);

    client.set_fee(&admin, &300_u32);

    client.mark_shipped(&seller, &escrow_id, &SorobanString::from_str(&env, "TRACK-FEE-SNAP"));
    let escrow = client.get_escrow(&escrow_id);
    env.ledger().set_timestamp(escrow.dispute_deadline + 1);
    client.confirm_delivery(&buyer, &escrow_id);

    let escrow_state = client.get_escrow(&escrow_id);
    assert_eq!(escrow_state.fee_bps, 100);
    assert_eq!(get_balance(&env, &token, &seller), 990_000);
}

// ============================================================================
// Event Data Integrity Tests — Issue #91
// ============================================================================
//
// Capture emitted events, decode them, and verify every field to
// ensure zero data corruption across the event-logging pipeline.

#[test]
fn test_event_integrity_escrow_created() {
    let (env, admin, seller, _buyer, resolver, token, fee_collector) = setup_env();
    let cid = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &cid);
    client.initialize(&admin, &fee_collector, &0_u32);

    let escrow_id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &500_i128, &150_u32, &7200_u64);

    assert!(has_event::<EscrowCreated, _>(&env, &cid, "escrow_created", |ev| {
        ev.escrow_id == escrow_id as u64
            && ev.seller == seller
            && ev.resolver == resolver
            && ev.token == token
            && ev.amount == 500
            && ev.fee_bps == 150
            && ev.shipping_window == 7200
    }));
}

#[test]
<<<<<<< HEAD
fn test_event_integrity_escrow_funded() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let cid = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &cid);
    client.initialize(&admin, &fee_collector, &0_u32);
=======
#[should_panic(expected = "shipping window not elapsed")]
fn test_auto_release_before_window_fails() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);

>>>>>>> 7c0b9a0 (feat: implement issues #205, #206, #209)
    mint_tokens(&env, &token, &buyer, 1000);

    let escrow_id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &500_i128, &150_u32, &7200_u64);
    client.fund_escrow(&escrow_id, &buyer);

    assert!(has_event::<EscrowFunded, _>(&env, &cid, "escrow_funded", |ev| {
        ev.escrow_id == escrow_id as u64
            && ev.buyer == buyer
            && ev.amount == 500
    }));
}

#[test]
<<<<<<< HEAD
fn test_event_integrity_escrow_completed_via_confirm_delivery() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let cid = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &cid);
    client.initialize(&admin, &fee_collector, &0_u32);
    mint_tokens(&env, &token, &buyer, 1000);

    let escrow_id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&escrow_id, &buyer);
    client.mark_shipped(&seller, &escrow_id, &SorobanString::from_str(&env, "TRK-EVT-CD"));
=======
#[should_panic(expected = "evidence_hash must be exactly 32 bytes")]
fn test_raise_dispute_invalid_evidence_hash_rejected() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &resolver, &token, &100_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
>>>>>>> 7c0b9a0 (feat: implement issues #205, #206, #209)

    let escrow = client.get_escrow(&escrow_id);
    env.ledger().set_timestamp(escrow.dispute_deadline + 1);
    client.confirm_delivery(&buyer, &escrow_id);

    assert!(has_event::<EscrowCompleted, _>(&env, &cid, "escrow_completed", |ev| {
        ev.escrow_id == escrow_id as u64
            && ev.recipient == seller
            && ev.amount > 0
            && ev.fee_bps == 200
    }));
}

#[test]
<<<<<<< HEAD
fn test_event_integrity_dispute_raised() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let cid = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &cid);
    client.initialize(&admin, &fee_collector, &0_u32);
    mint_tokens(&env, &token, &buyer, 1000);

    let escrow_id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&escrow_id, &buyer);
    client.mark_shipped(&seller, &escrow_id, &SorobanString::from_str(&env, "TRK-EVT-DR"));
=======
#[should_panic(expected = "escrow not funded")]
fn test_raise_dispute_only_once() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &resolver, &token, &100_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
>>>>>>> 7c0b9a0 (feat: implement issues #205, #206, #209)

    let evidence = BytesN::from_array(&env, &[42u8; 32]);
    client.raise_dispute(
        &buyer,
        &escrow_id,
        &Symbol::new(&env, "non_delivery"),
        &SorobanString::from_str(&env, "Item never arrived"),
        &evidence,
    );

    assert!(has_event::<DisputeRaised, _>(&env, &cid, "dispute_raised", |ev| {
        ev.escrow_id == escrow_id as u64
            && ev.buyer == buyer
            && ev.evidence_hash == evidence
    }));
}

#[test]
<<<<<<< HEAD
fn test_event_integrity_dispute_resolved_release_to_seller() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let cid = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &cid);
    client.initialize(&admin, &fee_collector, &0_u32);
    mint_tokens(&env, &token, &buyer, 1000);

    let escrow_id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&escrow_id, &buyer);
    client.mark_shipped(&seller, &escrow_id, &SorobanString::from_str(&env, "TRK-EVT-RES"));
    client.raise_dispute(
        &buyer,
        &escrow_id,
        &Symbol::new(&env, "reason"),
        &SorobanString::from_str(&env, "desc"),
        &BytesN::from_array(&env, &[0u8; 32]),
    );
    client.resolve_dispute(&resolver, &escrow_id, &ResolutionType::Release);
=======
fn test_multiple_escrows() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);
>>>>>>> 7c0b9a0 (feat: implement issues #205, #206, #209)

    assert!(has_event::<DisputeResolved, _>(&env, &cid, "dispute_resolved", |ev| {
        ev.escrow_id == escrow_id as u64
            && ev.resolution == ResolutionType::Release
    }));
}

#[test]
fn test_event_integrity_dispute_resolved_refund_buyer() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let cid = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &cid);
    client.initialize(&admin, &fee_collector, &0_u32);
    mint_tokens(&env, &token, &buyer, 1000);

    let escrow_id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&escrow_id, &buyer);
    client.mark_shipped(&seller, &escrow_id, &SorobanString::from_str(&env, "TRK-EVT-REF"));
    client.raise_dispute(
        &buyer,
        &escrow_id,
        &Symbol::new(&env, "reason"),
        &SorobanString::from_str(&env, "desc"),
        &BytesN::from_array(&env, &[0u8; 32]),
    );
    client.resolve_dispute(&resolver, &escrow_id, &ResolutionType::Refund);

    assert!(has_event::<DisputeResolved, _>(&env, &cid, "dispute_resolved", |ev| {
        ev.escrow_id == escrow_id as u64
            && ev.resolution == ResolutionType::Refund
    }));
}

#[test]
fn test_event_integrity_auto_released() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let cid = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &cid);
    client.initialize(&admin, &fee_collector, &0_u32);
    mint_tokens(&env, &token, &buyer, 1000);

    let escrow_id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&escrow_id, &buyer);
    client.mark_shipped(&seller, &escrow_id, &SorobanString::from_str(&env, "TRK-EVT-AR"));
    env.ledger().set_timestamp(1_700_000_000);
    client.record_delivery(&admin, &escrow_id);

    let escrow = client.get_escrow(&escrow_id);
    env.ledger().set_timestamp(escrow.delivered_at.unwrap() + 172_801);
    client.auto_release(&escrow_id);

    assert!(has_event::<AutoReleased, _>(&env, &cid, "auto_released", |ev| {
        ev.escrow_id == escrow_id as u64
            && ev.seller == seller
            && ev.amount == 1000
            && ev.fee_bps == 200
    }));
}

#[test]
fn test_event_integrity_full_lifecycle_all_events_decoded() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let cid = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &cid);
    client.initialize(&admin, &fee_collector, &0_u32);
    mint_tokens(&env, &token, &buyer, 2000);

    let id1 = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &500_i128, &200_u32, &3600_u64);
    assert!(has_event::<EscrowCreated, _>(&env, &cid, "escrow_created", |ev| {
        ev.escrow_id == id1 as u64 && ev.seller == seller
    }));

    let id2 = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &200_u32, &7200_u64);
    assert!(has_event::<EscrowCreated, _>(&env, &cid, "escrow_created", |ev| {
        ev.escrow_id == id2 as u64
    }));

    client.fund_escrow(&id1, &buyer);
    assert!(has_event::<EscrowFunded, _>(&env, &cid, "escrow_funded", |ev| {
        ev.escrow_id == id1 as u64 && ev.buyer == buyer
    }));

    client.mark_shipped(&seller, &id1, &SorobanString::from_str(&env, "TRK-LIFE-1"));
    let escrow1 = client.get_escrow(&id1);
    env.ledger().set_timestamp(escrow1.dispute_deadline + 1);
    client.confirm_delivery(&buyer, &id1);
    assert!(has_event::<EscrowCompleted, _>(&env, &cid, "escrow_completed", |ev| {
        ev.escrow_id == id1 as u64
    }));

    client.fund_escrow(&id2, &buyer);
    assert!(has_event::<EscrowFunded, _>(&env, &cid, "escrow_funded", |ev| {
        ev.escrow_id == id2 as u64
    }));

    client.mark_shipped(&seller, &id2, &SorobanString::from_str(&env, "TRK-LIFE-2"));
    client.raise_dispute(
        &buyer,
        &id2,
        &Symbol::new(&env, "reason"),
        &SorobanString::from_str(&env, "desc"),
        &BytesN::from_array(&env, &[0u8; 32]),
    );
    assert!(has_event::<DisputeRaised, _>(&env, &cid, "dispute_raised", |ev| {
        ev.escrow_id == id2 as u64
    }));

    client.resolve_dispute(&resolver, &id2, &ResolutionType::Refund);
    assert!(has_event::<DisputeResolved, _>(&env, &cid, "dispute_resolved", |ev| {
        ev.escrow_id == id2 as u64 && ev.resolution == ResolutionType::Refund
    }));
}

<<<<<<< HEAD
// ---------------------------------------------------------------------------
// cancel_escrow tests — Issue #89
// ---------------------------------------------------------------------------
=======
#[test]
fn test_zero_fee_no_collector_transfer() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
>>>>>>> 7c0b9a0 (feat: implement issues #205, #206, #209)

#[test]
fn test_cancel_escrow_by_buyer_refunds_full_amount() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
<<<<<<< HEAD
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector, &0_u32);
=======
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);
>>>>>>> 7c0b9a0 (feat: implement issues #205, #206, #209)

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &500_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);

    assert_eq!(get_balance(&env, &token, &buyer), 500);
    assert_eq!(get_balance(&env, &token, &contract_id), 500);

    client.cancel_escrow(&buyer, &id);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Refunded);

    assert_eq!(get_balance(&env, &token, &buyer), 1000);
    assert_eq!(get_balance(&env, &token, &contract_id), 0);
    assert_eq!(get_balance(&env, &token, &seller), 0);
    assert_eq!(get_balance(&env, &token, &fee_collector), 0);
}

#[test]
<<<<<<< HEAD
fn test_cancel_escrow_state_transitions_correctly() {
    let (env, admin, seller, buyer, resolver, token, _fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &_fee_collector, &0_u32);
=======
#[should_panic(expected = "fee exceeds maximum")]
fn test_fee_exceeds_max_bps_fails() {
    let (env, admin, seller, _buyer, resolver, _token_admin, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);
>>>>>>> 7c0b9a0 (feat: implement issues #205, #206, #209)

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &300_i128, &100_u32, &7200_u64);
    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Pending);
    assert!(escrow.buyer.is_none());

    client.fund_escrow(&id, &buyer);
    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Funded);
    assert_eq!(escrow.buyer, Some(buyer.clone()));

    client.cancel_escrow(&buyer, &id);
    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Refunded);
    assert_eq!(escrow.buyer, Some(buyer));
}

#[test]
fn test_cancel_escrow_pending_escrow_fails() {
    let (env, admin, seller, buyer, resolver, token, _fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &_fee_collector, &0_u32);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &500_i128, &200_u32, &3600_u64);
    let res = client.try_cancel_escrow(&buyer, &id);
    assert!(matches!(res, Err(Ok(ContractError::InvalidState))));
}

#[test]
fn test_cancel_escrow_completed_escrow_fails() {
    let (env, admin, seller, buyer, resolver, token, _fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &_fee_collector, &0_u32);

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.mark_shipped(&seller, &id, &SorobanString::from_str(&env, "TRK-CANCEL"));
    let escrow = client.get_escrow(&id);
    env.ledger().set_timestamp(escrow.dispute_deadline + 1);
    client.confirm_delivery(&buyer, &id);

    let res = client.try_cancel_escrow(&buyer, &id);
    assert!(matches!(res, Err(Ok(ContractError::InvalidState))));
}

#[test]
fn test_cancel_escrow_already_cancelled_fails() {
    let (env, admin, seller, buyer, resolver, token, _fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &_fee_collector, &0_u32);

    mint_tokens(&env, &token, &buyer, 1000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &500_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.cancel_escrow(&buyer, &id);

    let res = client.try_cancel_escrow(&buyer, &id);
    assert!(matches!(res, Err(Ok(ContractError::InvalidState))));
}

#[test]
fn test_cancel_escrow_with_zero_fee() {
    let (env, admin, seller, buyer, resolver, token, _fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &_fee_collector, &0_u32);

    mint_tokens(&env, &token, &buyer, 500);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &500_i128, &0_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.cancel_escrow(&buyer, &id);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Refunded);

    assert_eq!(get_balance(&env, &token, &buyer), 500);
    assert_eq!(get_balance(&env, &token, &contract_id), 0);
}

#[test]
fn test_cancel_escrow_preserves_escrow_metadata() {
    let (env, admin, seller, buyer, resolver, token, _fee_collector) = setup_env();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &_fee_collector, &0_u32);

    mint_tokens(&env, &token, &buyer, 2000);

    let id = client.create_escrow(&seller, &None::<Address>, &resolver, &token, &1500_i128, &250_u32, &86400_u64);
    client.fund_escrow(&id, &buyer);
    client.cancel_escrow(&buyer, &id);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.seller, seller);
    assert_eq!(escrow.buyer, Some(buyer));
    assert_eq!(escrow.resolver, resolver);
    assert_eq!(escrow.token, token);
    assert_eq!(escrow.amount, 1500);
    assert_eq!(escrow.fee_bps, 250);
    assert_eq!(escrow.shipping_window, 86400);
    assert_eq!(escrow.state, EscrowState::Refunded);
}


// ============ Tests for Issue #206: Pagination ============

#[test]
fn test_get_escrows_by_vendor_paginated() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);

    // Create 5 escrows for the same seller
    let mut ids = Vec::new(&env);
    for i in 1..=5 {
        let id = client.create_escrow(&seller, &resolver, &token, &(i * 100), &200_u32, &3600_u64);
        ids.push_back(id);
    }

    // Test first page (offset 0, limit 2)
    let page1 = client.get_escrows_by_vendor(&seller, &0, &2).unwrap();
    assert_eq!(page1.len(), 2);
    assert_eq!(page1.get(0).unwrap(), ids.get(0).unwrap());
    assert_eq!(page1.get(1).unwrap(), ids.get(1).unwrap());

    // Test second page (offset 2, limit 2)
    let page2 = client.get_escrows_by_vendor(&seller, &2, &2).unwrap();
    assert_eq!(page2.len(), 2);
    assert_eq!(page2.get(0).unwrap(), ids.get(2).unwrap());
    assert_eq!(page2.get(1).unwrap(), ids.get(3).unwrap());

    // Test last page (offset 4, limit 10)
    let page3 = client.get_escrows_by_vendor(&seller, &4, &10).unwrap();
    assert_eq!(page3.len(), 1);
    assert_eq!(page3.get(0).unwrap(), ids.get(4).unwrap());

    // Test count
    assert_eq!(client.get_escrow_count_by_vendor(&seller), 5);
}

#[test]
fn test_get_escrows_by_buyer_paginated() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);

    mint_tokens(&env, &token, &buyer, 5000);

    // Create and fund 3 escrows
    let mut ids = Vec::new(&env);
    for i in 1..=3 {
        let id = client.create_escrow(&seller, &resolver, &token, &(i * 100), &200_u32, &3600_u64);
        client.fund_escrow(&id, &buyer);
        ids.push_back(id);
    }

    // Test pagination
    let page = client.get_escrows_by_buyer(&buyer, &0, &2).unwrap();
    assert_eq!(page.len(), 2);
    
    let all = client.get_escrows_by_buyer(&buyer, &0, &50).unwrap();
    assert_eq!(all.len(), 3);

    // Test count
    assert_eq!(client.get_escrow_count_by_buyer(&buyer), 3);
}

#[test]
fn test_pagination_limit_exceeded() {
    let (env, admin, seller, _buyer, resolver, _token_admin, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);

    client.create_escrow(&seller, &resolver, &token, &100, &200_u32, &3600_u64);

    // Try to exceed MAX_PAGE_SIZE (50)
    let result = client.try_get_escrows_by_vendor(&seller, &0, &51);
    assert!(result.is_err());
}

// ============ Tests for Issue #205: Configurable Windows ============

#[test]
fn test_update_dispute_window() {
    let (env, admin, _seller, _buyer, _resolver, _token_admin, _token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);

    // Default is 172_800 (48 hours)
    let config = client.get_config();
    assert_eq!(config.dispute_window_secs, 172_800);

    // Update to 86_400 (24 hours)
    client.update_dispute_window(&admin, &86_400).unwrap();

    let config = client.get_config();
    assert_eq!(config.dispute_window_secs, 86_400);
}

#[test]
fn test_update_delivery_window() {
    let (env, admin, _seller, _buyer, _resolver, _token_admin, _token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);

    // Default is 172_800 (48 hours)
    let config = client.get_config();
    assert_eq!(config.delivery_release_window_secs, 172_800);

    // Update to 259_200 (72 hours)
    client.update_delivery_window(&admin, &259_200).unwrap();

    let config = client.get_config();
    assert_eq!(config.delivery_release_window_secs, 259_200);
}

#[test]
fn test_window_update_boundaries() {
    let (env, admin, _seller, _buyer, _resolver, _token_admin, _token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);

    // Too short (less than 1 hour)
    let result = client.try_update_dispute_window(&admin, &3_599);
    assert!(result.is_err());

    // Too long (more than 30 days)
    let result = client.try_update_dispute_window(&admin, &2_592_001);
    assert!(result.is_err());

    // Valid boundaries
    assert!(client.try_update_dispute_window(&admin, &3_600).is_ok());  // 1 hour
    assert!(client.try_update_dispute_window(&admin, &2_592_000).is_ok());  // 30 days
}

// ============ Tests for Issue #209: Withdraw Fees Guard ============

#[test]
fn test_withdraw_fees_with_guard() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);

    mint_tokens(&env, &token, &buyer, 2000);

    // Create and complete an escrow (generates fees)
    let id = client.create_escrow(&seller, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id, &buyer);
    client.confirm_delivery(&id);

    // Fee is 20 (2% of 1000), contract has 20 in fees
    let accumulated = client.get_accumulated_fees(&token);
    assert_eq!(accumulated, 20);

    // Withdraw the fees
    client.withdraw_fees(&admin, &token, &20).unwrap();

    assert_eq!(client.get_accumulated_fees(&token), 0);
    assert_eq!(get_balance(&env, &token, &fee_collector), 40); // 20 from deduction + 20 withdrawn
}

#[test]
fn test_withdraw_fees_respects_locked_balance() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);

    mint_tokens(&env, &token, &buyer, 3000);

    // Create and complete one escrow (generates 20 in fees)
    let id1 = client.create_escrow(&seller, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id1, &buyer);
    client.confirm_delivery(&id1);

    // Create and fund another escrow (locks 1000)
    let id2 = client.create_escrow(&seller, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id2, &buyer);

    // Contract now has: 20 (fees from id1) + 1000 (locked for id2) = 1020
    // Locked balance is 1000
    assert_eq!(client.get_locked_balance(&token), 1000);
    assert_eq!(get_balance(&env, &token, &contract_id), 1020);

    // Try to withdraw 21 - should fail (contract would have 999, less than locked 1000)
    let result = client.try_withdraw_fees(&admin, &token, &21);
    assert!(result.is_err());

    // Withdraw 20 - should succeed
    client.withdraw_fees(&admin, &token, &20).unwrap();
    assert_eq!(get_balance(&env, &token, &contract_id), 1000);
}

#[test]
fn test_locked_balance_tracking() {
    let (env, admin, seller, buyer, resolver, token, fee_collector) = setup_env();

    let contract_id = env.register(Escrow, ());
    let client = super::EscrowClient::new(&env, &contract_id);
    client.initialize(&admin, &fee_collector);

    mint_tokens(&env, &token, &buyer, 3000);

    // Initially zero
    assert_eq!(client.get_locked_balance(&token), 0);

    // Fund escrow - locks 1000
    let id1 = client.create_escrow(&seller, &resolver, &token, &1000_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id1, &buyer);
    assert_eq!(client.get_locked_balance(&token), 1000);

    // Fund another - locks another 500
    let id2 = client.create_escrow(&seller, &resolver, &token, &500_i128, &200_u32, &3600_u64);
    client.fund_escrow(&id2, &buyer);
    assert_eq!(client.get_locked_balance(&token), 1500);

    // Complete first - unlocks 1000
    client.confirm_delivery(&id1);
    assert_eq!(client.get_locked_balance(&token), 500);

    // Complete second - unlocks 500
    client.confirm_delivery(&id2);
    assert_eq!(client.get_locked_balance(&token), 0);
}
