#![allow(unused_imports)]

use elrond_wasm::types::{Address, EsdtLocalRole, ManagedAddress, SCResult, TokenIdentifier};
use elrond_wasm_debug::{
    assert_sc_error, managed_address, managed_token_id, rust_biguint, DebugApi,
};
use elrond_wasm_debug::{managed_biguint, testing_framework::*};
use num_traits::ToPrimitive;
use pair_mock::*;
use price_discovery::common_storage::*;
use price_discovery::create_pool::*;
use price_discovery::redeem_token::*;
use price_discovery::*;

mod tests_common;
use tests_common::*;

#[test]
fn test_init() {
    let _ = init(price_discovery::contract_obj, pair_mock::contract_obj);
}

#[test]
fn test_deposit_initial_tokens_too_late() {
    let mut pd_setup = init(price_discovery::contract_obj, pair_mock::contract_obj);
    pd_setup.blockchain_wrapper.set_block_epoch(5);

    let sc_result = call_deposit_initial_tokens(
        &mut pd_setup,
        &rust_biguint!(5_000_000_000),
        StateChange::Revert,
    );
    assert_sc_error!(sc_result, b"May only deposit before start epoch");
}

#[test]
fn test_deposit_initial_tokens_ok() {
    let mut pd_setup = init(price_discovery::contract_obj, pair_mock::contract_obj);

    let init_deposit_amt = rust_biguint!(5_000_000_000);
    let sc_result =
        call_deposit_initial_tokens(&mut pd_setup, &init_deposit_amt, StateChange::Commit);
    assert_eq!(sc_result, SCResult::Ok(()));

    pd_setup.blockchain_wrapper.check_esdt_balance(
        pd_setup.pd_wrapper.address_ref(),
        LAUNCHED_TOKEN_ID,
        &init_deposit_amt,
    );
}

#[test]
fn user_deposit_too_early() {
    let mut pd_setup = init(price_discovery::contract_obj, pair_mock::contract_obj);

    let mut sc_result = call_deposit_initial_tokens(
        &mut pd_setup,
        &rust_biguint!(5_000_000_000),
        StateChange::Commit,
    );
    assert_eq!(sc_result, SCResult::Ok(()));

    pd_setup.blockchain_wrapper.set_block_epoch(START_EPOCH - 1);

    // must clone, as we can't borrow pd_setup as mutable and as immutable at the same time
    let first_user_address = pd_setup.first_user_address.clone();
    sc_result = call_deposit(
        &mut pd_setup,
        &first_user_address,
        &rust_biguint!(1_000_000_000),
        StateChange::Revert,
    );
    assert_sc_error!(sc_result, b"Deposit period not started yet");
}

#[test]
fn user_deposit_without_launched_tokens_deposited() {
    let mut pd_setup = init(price_discovery::contract_obj, pair_mock::contract_obj);

    pd_setup.blockchain_wrapper.set_block_epoch(START_EPOCH + 1);

    // must clone, as we can't borrow pd_setup as mutable and as immutable at the same time
    let first_user_address = pd_setup.first_user_address.clone();
    let sc_result = call_deposit(
        &mut pd_setup,
        &first_user_address,
        &rust_biguint!(1_000_000_000),
        StateChange::Revert,
    );
    assert_sc_error!(sc_result, b"Launched tokens not deposited");
}

pub fn user_deposit_ok_steps<PriceDiscObjBuilder, DexObjBuilder>(
    pd_setup: &mut PriceDiscSetup<PriceDiscObjBuilder, DexObjBuilder>,
) where
    PriceDiscObjBuilder: 'static + Copy + Fn(DebugApi) -> price_discovery::ContractObj<DebugApi>,
    DexObjBuilder: 'static + Copy + Fn(DebugApi) -> pair_mock::ContractObj<DebugApi>,
{
    let mut sc_result =
        call_deposit_initial_tokens(pd_setup, &rust_biguint!(5_000_000_000), StateChange::Commit);
    assert_eq!(sc_result, SCResult::Ok(()));

    pd_setup.blockchain_wrapper.set_block_epoch(START_EPOCH + 1);

    // must clone, as we can't borrow pd_setup as mutable and as immutable at the same time
    let first_user_address = pd_setup.first_user_address.clone();
    let first_deposit_amt = rust_biguint!(1_000_000_000);
    sc_result = call_deposit(
        pd_setup,
        &first_user_address,
        &first_deposit_amt,
        StateChange::Commit,
    );
    assert_eq!(sc_result, SCResult::Ok(()));

    pd_setup.blockchain_wrapper.check_nft_balance(
        &first_user_address,
        REDEEM_TOKEN_ID,
        ACCEPTED_TOKEN_REDEEM_NONCE,
        &first_deposit_amt,
        &(),
    );

    // second user deposit
    let second_user_address = pd_setup.second_user_address.clone();
    let second_deposit_amt = rust_biguint!(500_000_000);
    sc_result = call_deposit(
        pd_setup,
        &second_user_address,
        &second_deposit_amt,
        StateChange::Commit,
    );
    assert_eq!(sc_result, SCResult::Ok(()));

    pd_setup.blockchain_wrapper.check_nft_balance(
        &second_user_address,
        REDEEM_TOKEN_ID,
        ACCEPTED_TOKEN_REDEEM_NONCE,
        &second_deposit_amt,
        &(),
    );

    // check SC balance
    pd_setup.blockchain_wrapper.check_esdt_balance(
        pd_setup.pd_wrapper.address_ref(),
        ACCEPTED_TOKEN_ID,
        &(first_deposit_amt + second_deposit_amt),
    );
}

#[test]
fn user_deposit_ok() {
    let mut pd_setup = init(price_discovery::contract_obj, pair_mock::contract_obj);
    user_deposit_ok_steps(&mut pd_setup);
}

pub fn withdraw_ok_steps<PriceDiscObjBuilder, DexObjBuilder>(
    pd_setup: &mut PriceDiscSetup<PriceDiscObjBuilder, DexObjBuilder>,
) where
    PriceDiscObjBuilder: 'static + Copy + Fn(DebugApi) -> price_discovery::ContractObj<DebugApi>,
    DexObjBuilder: 'static + Copy + Fn(DebugApi) -> pair_mock::ContractObj<DebugApi>,
{
    let first_user_address = pd_setup.first_user_address.clone();
    let balance_before = rust_biguint!(0);
    let deposit_amt = rust_biguint!(1_000_000_000);
    let withdraw_amt = rust_biguint!(400_000_000);
    let sc_result = call_withdraw(
        pd_setup,
        &first_user_address,
        &withdraw_amt,
        StateChange::Commit,
    );
    assert_eq!(sc_result, SCResult::Ok(()));

    pd_setup.blockchain_wrapper.check_nft_balance(
        &first_user_address,
        REDEEM_TOKEN_ID,
        ACCEPTED_TOKEN_REDEEM_NONCE,
        &(&deposit_amt - &withdraw_amt),
        &(),
    );

    // check that the SC burned the tokens
    // 1 remains for ESDTNFTAddQuantity purposes
    pd_setup.blockchain_wrapper.check_nft_balance(
        &pd_setup.pd_wrapper.address_ref(),
        REDEEM_TOKEN_ID,
        ACCEPTED_TOKEN_REDEEM_NONCE,
        &rust_biguint!(1),
        &(),
    );

    pd_setup.blockchain_wrapper.check_esdt_balance(
        &first_user_address,
        ACCEPTED_TOKEN_ID,
        &(&balance_before + &withdraw_amt),
    );

    let sc_balance_before = rust_biguint!(1_500_000_000);
    pd_setup.blockchain_wrapper.check_esdt_balance(
        &pd_setup.pd_wrapper.address_ref(),
        ACCEPTED_TOKEN_ID,
        &(&sc_balance_before - &withdraw_amt),
    );
}

#[test]
fn withdraw_ok() {
    let mut pd_setup = init(price_discovery::contract_obj, pair_mock::contract_obj);
    user_deposit_ok_steps(&mut pd_setup);
    withdraw_ok_steps(&mut pd_setup);
}

#[test]
fn withdraw_too_late() {
    let mut pd_setup = init(price_discovery::contract_obj, pair_mock::contract_obj);
    user_deposit_ok_steps(&mut pd_setup);

    pd_setup.blockchain_wrapper.set_block_epoch(END_EPOCH + 1);

    let first_user_address = pd_setup.first_user_address.clone();
    let withdraw_amt = rust_biguint!(400_000_000);
    let sc_result = call_withdraw(
        &mut pd_setup,
        &first_user_address,
        &withdraw_amt,
        StateChange::Revert,
    );
    assert_sc_error!(sc_result, b"Deposit period ended");
}

#[test]
fn create_pool_too_early() {
    let mut pd_setup = init(price_discovery::contract_obj, pair_mock::contract_obj);
    user_deposit_ok_steps(&mut pd_setup);
    withdraw_ok_steps(&mut pd_setup);

    let first_user_address = pd_setup.first_user_address.clone();
    let sc_result =
        call_create_dex_liquidity_pool(&mut pd_setup, &first_user_address, StateChange::Revert);
    assert_sc_error!(sc_result, b"Deposit period has not ended");
}

fn create_pool_ok_steps<PriceDiscObjBuilder, DexObjBuilder>(
    pd_setup: &mut PriceDiscSetup<PriceDiscObjBuilder, DexObjBuilder>,
) where
    PriceDiscObjBuilder: 'static + Copy + Fn(DebugApi) -> price_discovery::ContractObj<DebugApi>,
    DexObjBuilder: 'static + Copy + Fn(DebugApi) -> pair_mock::ContractObj<DebugApi>,
{
    pd_setup.blockchain_wrapper.set_block_epoch(END_EPOCH + 1);

    let first_user_address = pd_setup.first_user_address.clone();
    let sc_result =
        call_create_dex_liquidity_pool(pd_setup, &first_user_address, StateChange::Commit);
    assert_eq!(sc_result, SCResult::Ok(()));
}

#[test]
fn create_pool_ok() {
    let mut pd_setup = init(price_discovery::contract_obj, pair_mock::contract_obj);
    user_deposit_ok_steps(&mut pd_setup);
    withdraw_ok_steps(&mut pd_setup);
    create_pool_ok_steps(&mut pd_setup);

    let b_mock = &mut pd_setup.blockchain_wrapper;
    let expected_lp_token_balance = 1_100_000_000 - MINIMUM_LIQUIDITY;
    b_mock.check_esdt_balance(
        pd_setup.pd_wrapper.address_ref(),
        LP_TOKEN_ID,
        &rust_biguint!(expected_lp_token_balance),
    );
}

#[test]
fn redeem_before_pool_created() {
    let mut pd_setup = init(price_discovery::contract_obj, pair_mock::contract_obj);
    user_deposit_ok_steps(&mut pd_setup);
    withdraw_ok_steps(&mut pd_setup);

    pd_setup.blockchain_wrapper.set_block_epoch(END_EPOCH + 1);

    let first_user_address = pd_setup.first_user_address.clone();
    let sc_result = call_redeem(
        &mut pd_setup,
        &first_user_address,
        ACCEPTED_TOKEN_REDEEM_NONCE,
        &rust_biguint!(600_000_000),
        StateChange::Revert,
    );
    assert_sc_error!(sc_result, b"Pool not created yet");
}
