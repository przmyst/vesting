use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Storage, Uint128, CosmosMsg, WasmMsg
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use serde::{Deserialize, Serialize};

const CONTRACT_NAME: &str = "terra-cw20-lock";
const CONTRACT_VERSION: &str = "0.1.0";

const LOCK_PERIOD: u64 = 605227;
const TARGET_ADDRESS: &str = "terra164kf48vusvnmsku8v37uy9ynxpr5u333hvcz0wd6mfr8el56wx9sfzuhxq";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Unlock { lock_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Lock { memo: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum QueryMsg {
    LockInfo { lock_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LockInfo {
    owner: String,
    token_address: String,
    amount: Uint128,
    unlock_time: u64,
    expected_interest: Uint128,
    is_unlocked: bool,
    nonce: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct State {
    nonce: u64,
    locked: bool,
}

const STATE_KEY: &[u8] = b"state";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    _: MessageInfo,
    _: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let state = State { nonce: 0, locked: false };
    deps.storage.set(STATE_KEY, &to_binary(&state)?);
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[entry_point]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => {
            set_reentrancy_lock(deps.storage)?;
            let res = try_lock(&mut deps, env, info, msg);
            clear_reentrancy_lock(deps.storage)?;
            res
        },
        ExecuteMsg::Unlock { lock_id } => {
            set_reentrancy_lock(deps.storage)?;
            let res = execute_unlock(&mut deps, env, info, lock_id);
            clear_reentrancy_lock(deps.storage)?;
            res
        },
    }
}

fn set_reentrancy_lock(storage: &mut dyn Storage) -> StdResult<()> {
    let mut state: State = storage.get(STATE_KEY)
        .map_or_else(|| Err(StdError::not_found("state")), |bytes| from_binary(&Binary::from(bytes)))?;
    if state.locked {
        return Err(StdError::generic_err("Reentrancy detected"));
    }
    state.locked = true;
    storage.set(STATE_KEY, &to_binary(&state)?);
    Ok(())
}

fn clear_reentrancy_lock(storage: &mut dyn Storage) -> StdResult<()> {
    let mut state: State = storage.get(STATE_KEY)
        .map_or_else(|| Err(StdError::not_found("state")), |bytes| from_binary(&Binary::from(bytes)))?;
    state.locked = false;
    storage.set(STATE_KEY, &to_binary(&state)?);
    Ok(())
}

fn try_lock(
    deps: &mut DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let lock_msg: ReceiveMsg = from_binary(&cw20_msg.msg)
        .map_err(|_| StdError::generic_err("Failed to parse lock message"))?;

    match lock_msg {
        ReceiveMsg::Lock { memo } => {
            let sender =  cw20_msg.sender.clone();
            let amount = cw20_msg.amount;
            let token_address = info.sender.to_string();

            if token_address != TARGET_ADDRESS {
                return Err(StdError::generic_err("Operation not allowed with this address"));
            }

            if amount.is_zero() {
                return Err(StdError::generic_err("Amount cannot be zero"));
            }

            let unlock_time = env.block.time.seconds() + LOCK_PERIOD;
            let expected_interest = calculate_interest(amount);

            let mut state: State = deps.storage.get(STATE_KEY)
                .map_or(Ok(State::default()), |bytes| from_binary(&Binary::from(bytes)))?;
            state.nonce += 1;
            deps.storage.set(STATE_KEY, &to_binary(&state)?);

            let token_address_clone = token_address.clone();

            let lock_info = LockInfo {
                owner: sender.clone(),
                token_address: token_address_clone,
                amount,
                unlock_time,
                expected_interest,
                is_unlocked: false,
                nonce: state.nonce,
            };

            store_lock_info(deps.storage, &lock_info, state.nonce)?;

            let lock_id = format!("{}_{}_{}", sender.clone(), token_address, state.nonce);

            store_lock_info(deps.storage, &lock_info, state.nonce)?;

            Ok(Response::new()
                .add_attribute("action", "lock")
                .add_attribute("memo", memo)
                .add_attribute("owner", &sender)
                .add_attribute("amount", &amount.to_string())
                .add_attribute("unlock_time", &unlock_time.to_string())
                .add_attribute("nonce", &state.nonce.to_string())
                .add_attribute("lock_id", &lock_id))
        }
    }
}

fn execute_unlock(
    deps: &mut DepsMut,
    env: Env,
    info: MessageInfo,
    lock_id: String,
) -> StdResult<Response> {
    let lock_info = get_lock_info(deps.storage, &lock_id)
        .map_err(|_| StdError::not_found("LockInfo not found"))?;

    if lock_info.is_unlocked {
        return Err(StdError::generic_err("Tokens already unlocked"));
    }

    if env.block.time.seconds() < lock_info.unlock_time {
        return Err(StdError::generic_err("Unlock period has not yet elapsed"));
    }

    if info.sender != lock_info.owner {
        return Err(StdError::generic_err("Only the owner can unlock the tokens"));
    }

    let mut lock_info = lock_info;
    lock_info.is_unlocked = true;
    deps.storage.set(lock_id.as_bytes(), &to_binary(&lock_info)?);

    // Calculate total amount to return (initial amount + interest)
    let total_amount = lock_info.amount + lock_info.expected_interest;

    // Create a transfer message for the CW20 contract
    let cw20_transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lock_info.token_address.clone(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
            recipient: lock_info.owner.clone(),
            amount: total_amount,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(cw20_transfer_msg)
        .add_attribute("action", "unlock")
        .add_attribute("lock_id", &lock_id)
        .add_attribute("nonce", &lock_info.nonce.to_string()))
}

fn get_lock_info(storage: &dyn Storage, lock_id: &str) -> StdResult<LockInfo> {
    storage.get(lock_id.as_bytes()).map_or_else(
        || Err(StdError::not_found("LockInfo")),
        |data| from_binary(&Binary::from(data))
    )
}

fn store_lock_info(storage: &mut dyn Storage, lock_info: &LockInfo, nonce: u64) -> StdResult<()> {
    let lock_id = format!("{}_{}_{}", lock_info.owner, lock_info.token_address, nonce);
    storage.set(lock_id.as_bytes(), &to_binary(&lock_info)?);
    Ok(())
}

fn calculate_interest(amount: Uint128) -> Uint128 {
    let interest_rate_per_hundred_thousand = 7770;
    let interest = amount.u128()
        .checked_mul(interest_rate_per_hundred_thousand as u128)
        .map(|v| v / 100_000)
        .unwrap_or_default();
    Uint128::new(interest)
}

#[entry_point]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LockInfo { lock_id } => {
            let lock_info = get_lock_info(deps.storage, &lock_id)?;
            to_binary(&lock_info)
        },
    }
}
