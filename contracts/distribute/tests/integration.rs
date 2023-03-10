//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests as follows:
//! 1. Copy them over verbatim
//! 2. Then change
//!      let mut deps = mock_dependencies(20, &[]);
//!    to
//!      let mut deps = mock_instance(WASM, &[]);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)

use cosmwasm_std::{
    coins, Addr, BlockInfo, Coin, ContractInfo, Env, MessageInfo, Response, Timestamp,
    TransactionInfo,
};
use cosmwasm_vm::testing::{instantiate, mock_info, mock_instance};
use cosmwasm_vm::{from_slice, Storage};

use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
use distributor::msg::InstantiateMsg;
use distributor::state::Config;
use cw_utils::Expiration;

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/distributor.wasm");
// You can uncomment this line instead to test productionified build from rust-optimizer
// static WASM: &[u8] = include_bytes!("../contract.wasm");

fn init_msg_expire_by_height(expiration: Expiration) -> InstantiateMsg {
    InstantiateMsg {
        arbiter: String::from("verifies"),
        recipient: String::from("benefits"),
        expiration: Some(expiration),
    }
}

fn mock_env_info_height(signer: &str, sent: &[Coin], height: u64, time: u64) -> (Env, MessageInfo) {
    let env = Env {
        block: BlockInfo {
            height,
            time: Timestamp::from_nanos(time),
            chain_id: String::from("test"),
        },
        contract: ContractInfo {
            address: Addr::unchecked(MOCK_CONTRACT_ADDR),
        },
        transaction: Some(TransactionInfo { index: 3 }),
    };
    let info = mock_info(signer, sent);
    (env, info)
}

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM, &[]);
    let msg = init_msg_expire_by_height(Expiration::AtHeight(1000));
    let (env, info) = mock_env_info_height("creator", &coins(1000, "earth"), 876, 0);
    let res: Response = instantiate(&mut deps, env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    deps.with_storage(|store| {
        let state: Config = from_slice(&store.get(b"config").0.unwrap().unwrap(), 2048).unwrap();
        assert_eq!(
            state,
            Config {
                arbiter: Addr::unchecked("verifies"),
                recipient: Addr::unchecked("benefits"),
                source: Addr::unchecked("creator"),
                expiration: Some(Expiration::AtHeight(1000))
            }
        );
        Ok(())
    })
    .unwrap();
}
