use elrond_wasm::types::{
    Address, BigUint, EsdtLocalRole, ManagedAddress, MultiArg4, MultiArgVec, OptionalArg,
    TokenIdentifier,
};
use elrond_wasm_debug::tx_mock::TxInputESDT;
use elrond_wasm_debug::{
    managed_address, managed_biguint, managed_buffer, managed_token_id, rust_biguint,
    testing_framework::*, DebugApi,
};

const PAIR_WASM_PATH: &'static str = "pair/output/pair.wasm";
const ROUTER_WASM_PATH: &'static str = "router/output/router.wasm";
const MEX_TOKEN_ID: &[u8] = b"MEX-abcdef";
const WEGLD_TOKEN_ID: &[u8] = b"WEGLD-abcdef";
const USDC_TOKEN_ID: &[u8] = b"USDC-abcdef";
const LPMEX_TOKEN_ID: &[u8] = b"LPMEX-abcdef";
const LPUSDC_TOKEN_ID: &[u8] = b"LPUSDC-abcdef";

const USER_TOTAL_MEX_TOKENS: u64 = 5_001_001_000;
const USER_TOTAL_WEGLD_TOKENS: u64 = 5_002_002_000;
const USER_TOTAL_USDC_TOKENS: u64 = 5_001_001_000;

const ADD_LIQUIDITY_TOKENS: u64 = 1_001_000;

use pair::config::*;
use pair::*;
use router::factory::*;
use router::lib::*;
use router::*;

#[allow(dead_code)]
struct RouterSetup<RouterObjBuilder, PairObjBuilder>
where
    RouterObjBuilder: 'static + Copy + Fn() -> router::ContractObj<DebugApi>,
    PairObjBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub user_address: Address,
    pub router_wrapper: ContractObjWrapper<router::ContractObj<DebugApi>, RouterObjBuilder>,
    pub mex_pair_wrapper: ContractObjWrapper<pair::ContractObj<DebugApi>, PairObjBuilder>,
    pub usdc_pair_wrapper: ContractObjWrapper<pair::ContractObj<DebugApi>, PairObjBuilder>,
}

fn setup_router<RouterObjBuilder, PairObjBuilder>(
    router_builder: RouterObjBuilder,
    pair_builder: PairObjBuilder,
) -> RouterSetup<RouterObjBuilder, PairObjBuilder>
where
    RouterObjBuilder: 'static + Copy + Fn() -> router::ContractObj<DebugApi>,
    PairObjBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_addr = blockchain_wrapper.create_user_account(&rust_zero);

    let router_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_addr),
        router_builder,
        ROUTER_WASM_PATH,
    );

    let mex_pair_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_addr),
        pair_builder,
        PAIR_WASM_PATH,
    );

    let usdc_pair_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_addr),
        pair_builder,
        PAIR_WASM_PATH,
    );

    blockchain_wrapper
        .execute_tx(&owner_addr, &mex_pair_wrapper, &rust_zero, |sc| {
            let first_token_id = managed_token_id!(WEGLD_TOKEN_ID);
            let second_token_id = managed_token_id!(MEX_TOKEN_ID);
            let router_address = managed_address!(&owner_addr);
            let router_owner_address = managed_address!(&owner_addr);
            let total_fee_percent = 300u64;
            let special_fee_percent = 50u64;

            sc.init(
                first_token_id,
                second_token_id,
                router_address,
                router_owner_address,
                total_fee_percent,
                special_fee_percent,
                OptionalArg::None,
            );

            let lp_token_id = managed_token_id!(LPMEX_TOKEN_ID);
            sc.lp_token_identifier().set(&lp_token_id);

            sc.state().set(&State::Active);

            StateChange::Commit
        })
        .assert_ok();

    blockchain_wrapper
        .execute_tx(&owner_addr, &usdc_pair_wrapper, &rust_zero, |sc| {
            let first_token_id = managed_token_id!(WEGLD_TOKEN_ID);
            let second_token_id = managed_token_id!(USDC_TOKEN_ID);
            let router_address = managed_address!(&owner_addr);
            let router_owner_address = managed_address!(&owner_addr);
            let total_fee_percent = 300u64;
            let special_fee_percent = 50u64;

            sc.init(
                first_token_id,
                second_token_id,
                router_address,
                router_owner_address,
                total_fee_percent,
                special_fee_percent,
                OptionalArg::None,
            );

            let lp_token_id = managed_token_id!(LPUSDC_TOKEN_ID);
            sc.lp_token_identifier().set(&lp_token_id);

            sc.state().set(&State::Active);

            StateChange::Commit
        })
        .assert_ok();

    blockchain_wrapper
        .execute_tx(&owner_addr, &router_wrapper, &rust_zero, |sc| {
            sc.init(OptionalArg::None);

            sc.pair_map().insert(
                PairTokens {
                    first_token_id: managed_token_id!(WEGLD_TOKEN_ID),
                    second_token_id: managed_token_id!(MEX_TOKEN_ID),
                },
                managed_address!(mex_pair_wrapper.address_ref()),
            );
            sc.pair_map().insert(
                PairTokens {
                    first_token_id: managed_token_id!(WEGLD_TOKEN_ID),
                    second_token_id: managed_token_id!(USDC_TOKEN_ID),
                },
                managed_address!(usdc_pair_wrapper.address_ref()),
            );

            StateChange::Commit
        })
        .assert_ok();

    let lp_token_roles = [EsdtLocalRole::Mint, EsdtLocalRole::Burn];
    blockchain_wrapper.set_esdt_local_roles(
        mex_pair_wrapper.address_ref(),
        LPMEX_TOKEN_ID,
        &lp_token_roles[..],
    );

    let lp_token_roles = [EsdtLocalRole::Mint, EsdtLocalRole::Burn];
    blockchain_wrapper.set_esdt_local_roles(
        usdc_pair_wrapper.address_ref(),
        LPUSDC_TOKEN_ID,
        &lp_token_roles[..],
    );

    let user_addr = blockchain_wrapper.create_user_account(&rust_biguint!(100_000_000));
    blockchain_wrapper.set_esdt_balance(
        &user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(USER_TOTAL_WEGLD_TOKENS),
    );
    blockchain_wrapper.set_esdt_balance(
        &user_addr,
        MEX_TOKEN_ID,
        &rust_biguint!(USER_TOTAL_MEX_TOKENS),
    );
    blockchain_wrapper.set_esdt_balance(
        &user_addr,
        USDC_TOKEN_ID,
        &rust_biguint!(USER_TOTAL_USDC_TOKENS),
    );

    RouterSetup {
        blockchain_wrapper,
        owner_address: owner_addr,
        user_address: user_addr,
        router_wrapper,
        mex_pair_wrapper,
        usdc_pair_wrapper,
    }
}

fn add_liquidity<RouterObjBuilder, PairObjBuilder>(
    pair_setup: &mut RouterSetup<RouterObjBuilder, PairObjBuilder>,
) where
    RouterObjBuilder: 'static + Copy + Fn() -> router::ContractObj<DebugApi>,
    PairObjBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
{
    let payments = vec![
        TxInputESDT {
            token_identifier: WEGLD_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(ADD_LIQUIDITY_TOKENS),
        },
        TxInputESDT {
            token_identifier: MEX_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(ADD_LIQUIDITY_TOKENS),
        },
    ];

    pair_setup
        .blockchain_wrapper
        .execute_esdt_multi_transfer(
            &pair_setup.user_address,
            &pair_setup.mex_pair_wrapper,
            &payments,
            |sc| {
                sc.add_liquidity(
                    managed_biguint!(ADD_LIQUIDITY_TOKENS),
                    managed_biguint!(ADD_LIQUIDITY_TOKENS),
                    OptionalArg::None,
                );

                StateChange::Commit
            },
        )
        .assert_ok();

    let payments = vec![
        TxInputESDT {
            token_identifier: WEGLD_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(ADD_LIQUIDITY_TOKENS),
        },
        TxInputESDT {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(ADD_LIQUIDITY_TOKENS),
        },
    ];

    pair_setup
        .blockchain_wrapper
        .execute_esdt_multi_transfer(
            &pair_setup.user_address,
            &pair_setup.usdc_pair_wrapper,
            &payments,
            |sc| {
                sc.add_liquidity(
                    managed_biguint!(ADD_LIQUIDITY_TOKENS),
                    managed_biguint!(ADD_LIQUIDITY_TOKENS),
                    OptionalArg::None,
                );

                StateChange::Commit
            },
        )
        .assert_ok();
}

fn multi_pair_swap<RoouterObjBuilder, PairObjBuilder>(
    pair_setup: &mut RouterSetup<RoouterObjBuilder, PairObjBuilder>,
    payment_token: &[u8],
    payment_amount: u64,
    args: &[(Address, &[u8], &[u8], u64)],
) where
    RoouterObjBuilder: 'static + Copy + Fn() -> router::ContractObj<DebugApi>,
    PairObjBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
{
    let payment_amount_big = rust_biguint!(payment_amount);

    pair_setup
        .blockchain_wrapper
        .execute_esdt_transfer(
            &pair_setup.user_address,
            &pair_setup.router_wrapper,
            payment_token,
            0,
            &payment_amount_big,
            |sc| {
                let mut vec_with_managed = Vec::new();
                for x in args.iter() {
                    vec_with_managed.push(MultiArg4::from((
                        managed_address!(&x.0),
                        managed_buffer!(&x.1),
                        managed_token_id!(x.2.to_owned()),
                        managed_biguint!(x.3),
                    )));
                }

                sc.multi_pair_swap(
                    managed_token_id!(payment_token),
                    managed_biguint!(payment_amount),
                    0,
                    MultiArgVec(vec_with_managed),
                    OptionalArg::None,
                );

                StateChange::Commit
            },
        )
        .assert_ok();
}

#[test]
fn test_router_setup() {
    let _ = setup_router(router::contract_obj, pair::contract_obj);
}

#[test]
fn test_multi_pair_swap() {
    let mut router_setup = setup_router(router::contract_obj, pair::contract_obj);

    add_liquidity(&mut router_setup);

    router_setup.blockchain_wrapper.check_esdt_balance(
        &router_setup.user_address,
        WEGLD_TOKEN_ID,
        &rust_biguint!(5_000_000_000),
    );
    router_setup.blockchain_wrapper.check_esdt_balance(
        &router_setup.user_address,
        MEX_TOKEN_ID,
        &rust_biguint!(5_000_000_000),
    );
    router_setup.blockchain_wrapper.check_esdt_balance(
        &router_setup.user_address,
        USDC_TOKEN_ID,
        &rust_biguint!(5_000_000_000),
    );

    let ops = vec![
        (
            router_setup.mex_pair_wrapper.address_ref().clone(),
            SWAP_TOKENS_FIXED_INPUT_FUNC_NAME,
            WEGLD_TOKEN_ID, //swap to wegld
            1,
        ),
        (
            router_setup.usdc_pair_wrapper.address_ref().clone(),
            SWAP_TOKENS_FIXED_INPUT_FUNC_NAME,
            USDC_TOKEN_ID, //swap to usdc
            1,
        ),
    ];

    multi_pair_swap(&mut router_setup, MEX_TOKEN_ID, 100_000, &ops);

    router_setup.blockchain_wrapper.check_esdt_balance(
        &router_setup.user_address,
        WEGLD_TOKEN_ID,
        &rust_biguint!(5_000_000_000), //unchanged
    );
    router_setup.blockchain_wrapper.check_esdt_balance(
        &router_setup.user_address,
        MEX_TOKEN_ID,
        &rust_biguint!(4_999_900_000), //spent 100_000
    );
    router_setup.blockchain_wrapper.check_esdt_balance(
        &router_setup.user_address,
        USDC_TOKEN_ID,
        &rust_biguint!(5_000_082_909), //gained 82_909
    );
}
