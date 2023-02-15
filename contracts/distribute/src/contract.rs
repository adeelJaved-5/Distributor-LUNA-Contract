
use cosmwasm_std::{
    entry_point, to_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,
};

use crate::error::ContractError;
use crate::msg::{AccountsResponse, RecordsResponse, LimitResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
use crate::state::{Record, RECORD};
use cw2::set_contract_version;

// Version info, for migration info
const CONTRACT_NAME: &str = "distributor";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        arbiter: deps.api.addr_validate(&msg.arbiter)?,
        burn: deps.api.addr_validate(&msg.burn)?,
        development: deps.api.addr_validate(&msg.development)?,
        amount: msg.amount,
    };
    let record = Record {
        burn: Uint128::new(0),
        jackpot: Uint128::new(0),
        development: Uint128::new(0)
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;
    RECORD.save(deps.storage, &record)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {} => execute_deposit(deps, info),
        ExecuteMsg::Withdraw { receiver, amount } => execute_withdraw(deps, env, info, receiver, amount),
        ExecuteMsg::Set {amount} => execute_set(deps, env, info, amount),
        ExecuteMsg::SetOwner {arbiter} => owner_set(deps, env, info, arbiter),
    }
}

fn execute_deposit(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let record = RECORD.load(deps.storage)?;
    if info.funds[0].amount != config.amount {
        return Err(ContractError::Invalid {});
    }
    let burn_perc = Uint128::new(85);
    let burn_amt = vec![Coin::new(info.funds[0].amount.multiply_ratio(burn_perc, Uint128::new(100)).into(), &info.funds[0].denom)];
    let jack_perc = Uint128::new(10);
    let jack_amt = vec![Coin::new(info.funds[0].amount.multiply_ratio(jack_perc, Uint128::new(100)).into(), &info.funds[0].denom)];
    let dev_perc = Uint128::new(5);
    let dev_amt = vec![Coin::new(info.funds[0].amount.multiply_ratio(dev_perc, Uint128::new(100)).into(), &info.funds[0].denom)];
    let new_record = Record {
        burn: burn_amt[0].amount + record.burn,
        jackpot: jack_amt[0].amount + record.jackpot,
        development: dev_amt[0].amount + record.development
    };
    RECORD.save(deps.storage, &new_record)?;
    Ok(send_batch_tokens(config.burn, burn_amt, config.development, dev_amt, "distribute"))
}

fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receiver: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let record = RECORD.load(deps.storage)?;
    if info.sender != config.arbiter {
        return Err(ContractError::Unauthorized {});
    }
    let dev_amt = vec![Coin::new(amount.into(), "uluna")];
    let to = deps.api.addr_validate(&receiver)?;
    let new_record = Record {
        burn: record.burn,
        jackpot: record.jackpot - amount,
        development:record.development
    };
    RECORD.save(deps.storage, &new_record)?;
    Ok(send_tokens(to, dev_amt, "withdraw"))
}

fn execute_set(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.arbiter {
        return Err(ContractError::Unauthorized {});
    }
    let new_config = Config{
        arbiter: config.arbiter,
        burn: config.burn,
        development: config.development,
        amount: amount
    };
    CONFIG.save(deps.storage, &new_config)?;
    Ok(Response::default())
}

fn owner_set(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    arbiter: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.arbiter {
        return Err(ContractError::Unauthorized {});
    }
    let new_config = Config{
        arbiter: deps.api.addr_validate(&arbiter)?,
        burn: config.burn,
        development: config.development,
        amount: config.amount
    };
    CONFIG.save(deps.storage, &new_config)?;
    Ok(Response::default())
}

// this is a helper to send the tokens so the business logic is easy to read
fn send_tokens(to_address: Addr, amount: Vec<Coin>, action: &str) -> Response {
    Response::new()
      .add_message(BankMsg::Send {
        to_address: to_address.clone().into(),
        amount,
      })
      .add_attribute("action", action)
      .add_attribute("to", to_address)
}

// this is a helper to send the tokens so the business logic is easy to read
fn send_batch_tokens(to_address1: Addr, amount1: Vec<Coin>, to_address2: Addr, amount2: Vec<Coin>, action: &str) -> Response {
    Response::new()
      .add_message(BankMsg::Send {
        to_address: to_address1.clone().into(),
        amount: amount1,
      })
      .add_message(BankMsg::Send {
        to_address: to_address2.clone().into(),
        amount: amount2,
      })
      .add_attribute("action", action)
      .add_attribute("to", to_address1)
      .add_attribute("action", action)
      .add_attribute("to", to_address2)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Accounts {} => to_binary(&query_accounts(deps)?),
        QueryMsg::Records {} => to_binary(&query_records(deps)?),
        QueryMsg::Limit {} => to_binary(&query_limit(deps)?),
    }
}

fn query_accounts(deps: Deps) -> StdResult<AccountsResponse> {
    let config = CONFIG.load(deps.storage)?;
    let arb_addr = config.arbiter;
    let bur_addr = config.burn;
    let dev_addr = config.development;
    let amt = config.amount;
    Ok(AccountsResponse { arbiter: arb_addr, burn: bur_addr, development: dev_addr, amount: amt })
}

fn query_records(deps: Deps) -> StdResult<RecordsResponse> {
    let record = RECORD.load(deps.storage)?;
    let burn_amt = record.burn;
    let jack_amt = record.jackpot;
    let dev_amt = record.development;
    Ok(RecordsResponse { burn: burn_amt, jackpot: jack_amt, development: dev_amt })
}

fn query_limit(deps: Deps) -> StdResult<LimitResponse> {
    let config = CONFIG.load(deps.storage)?;
    let limit_amt = config.amount;
    Ok(LimitResponse { limit: limit_amt })
}