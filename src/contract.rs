use cosmwasm_std::{to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError, StdResult, Storage, Uint128};

use crate::msg::{HandleMsg, InitMsg, QueryMsg, ConfigResponse, RefDataResponse, ReferenceData};
use crate::state::{RefData, State, config, config_read};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub static E9: u64 = 1_000_000_000;
pub static E18: u128 = 1_000_000_000_000_000_000;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        refs: HashMap::new(),
    };
    config(&mut deps.storage).save(&state)?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Relay { symbols, rates, resolve_times, request_ids } => update_refs(deps, &symbols, &rates, &resolve_times, &request_ids),
    }
}

pub fn update_refs<S: Storage, A: Api, Q: Querier>(deps: &mut Extern<S, A, Q>, symbols: &[String], new_rates: &[u64], new_resolve_times: &[u64], new_request_ids: &[u64]) -> Result<HandleResponse, StdError> {
    let len = symbols.len();
    if new_rates.len() != len || new_request_ids.len() != len || new_resolve_times.len() != len {
        return Err(StdError::generic_err("Different array length"));
    }
    let mut state = config(&mut deps.storage).load()?;
    for idx in 0..len {
        state.refs.insert(symbols[idx].clone(), RefData {
            rate: new_rates[idx],
            resolve_time: new_resolve_times[idx],
            request_id: new_request_ids[idx],
        });
    };
    config(&mut deps.storage).save(&state)?;
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetRefs {} => to_binary(&query_refs(deps)?),
        QueryMsg::GetReferenceData { base, quote } => {
            let base_ref_data = get_ref_data(deps,base).unwrap();
            let quote_ref_data = get_ref_data(deps, quote).unwrap();
            to_binary(&ReferenceData {
                rate: Uint128((base_ref_data.rate.u128() * E18)/ quote_ref_data.rate.u128()),
                last_updated_base: base_ref_data.last_update,
                last_updated_quote: quote_ref_data.last_update,
            })
        }
    }
}

fn query_refs<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<ConfigResponse> {
    let state = config_read(&deps.storage).load()?;
    Ok(state)
}

fn get_ref_data<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, symbol: String) -> Result<RefDataResponse, StdError> {
    if symbol == String::from("USD") {
        return Ok(RefDataResponse {
            rate: Uint128::from(E9),
            last_update: Uint128::from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()),
        });
    }
    let state = config_read(&deps.storage).load()?;
    let ref_data = state.refs.get(&symbol).unwrap();
    if ref_data.resolve_time <= 0 {
        return Err(StdError::not_found("RefData"));
    }
    return Ok(RefDataResponse {
        rate: Uint128::from(ref_data.rate),
        last_update:Uint128::from(ref_data.resolve_time),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{from_binary};
    use std::collections::HashMap;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {};

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, mock_env("anyone", &[]), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&mut deps, QueryMsg::GetRefs{}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!(HashMap::new(), value.refs);
    }

    #[test]
    fn insert_one() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {};
        let _res = init(&mut deps, mock_env("anyone", &[]), msg).unwrap();

        let msg = HandleMsg::Relay { symbols: vec![String::from("ETH")], rates: vec![1u64], resolve_times: vec![2u64], request_ids: vec![3u64] };
        let _res = handle(&mut deps, mock_env("anyone", &[]), msg).unwrap();

        let res = query(&mut deps, QueryMsg::GetRefs {}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();
        let mut mock_map = HashMap::new();

        mock_map.insert(String::from("ETH"), RefData{rate: 1u64, resolve_time: 2u64, request_id: 3u64});

        assert_eq!(mock_map, value.refs);
    }

    #[test]
    fn insert_batch() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {};
        let _res = init(&mut deps, mock_env("anyone", &[]), msg).unwrap();

        let msg = HandleMsg::Relay { symbols: vec![String::from("ETH"), String::from("BAND")], rates: vec![1u64, 100u64], resolve_times: vec![2u64, 200u64], request_ids: vec![3u64, 300u64] };
        let _res = handle(&mut deps, mock_env("anyone", &[]), msg).unwrap();

        let res = query(&mut deps,QueryMsg::GetRefs {}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();
        let mut mock_map = HashMap::new();

        mock_map.insert(String::from("ETH"), RefData{rate: 1u64, resolve_time: 2u64, request_id: 3u64});
        mock_map.insert(String::from("BAND"), RefData{rate: 100u64, resolve_time: 200u64, request_id: 300u64});

        assert_eq!(mock_map, value.refs);
    }

    #[test]
    fn update_rate() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {};
        let _res = init(&mut deps, mock_env("anyone", &[]), msg).unwrap();

        let msg = HandleMsg::Relay { symbols: vec![String::from("MATIC")], rates: vec![12u64], resolve_times: vec![124824u64], request_ids: vec![69u64] };
        let _res = handle(&mut deps, mock_env("anyone", &[]), msg).unwrap();

        let res = query(&mut deps, QueryMsg::GetRefs {}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();

        let mut mock_map01 = HashMap::new();
        mock_map01.insert(String::from("MATIC"), RefData{rate: 12u64, resolve_time: 124824u64, request_id: 69u64});
        assert_eq!(mock_map01, value.refs);

        let msg = HandleMsg::Relay { symbols: vec![String::from("MATIC")], rates: vec![24u64], resolve_times: vec![124824u64], request_ids: vec![69u64] };
        let _res = handle(&mut deps, mock_env("anyone", &[]), msg).unwrap();

        let res = query(&mut deps, QueryMsg::GetRefs {}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();

        let mut mock_map02 = HashMap::new();
        mock_map02.insert(String::from("MATIC"), RefData{rate: 24u64, resolve_time: 124824u64, request_id: 69u64});
        assert_eq!(mock_map02, value.refs);
    }

    #[test]
    fn query_test_valid() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {};
        let _res = init(&mut deps, mock_env("anyone", &[]), msg).unwrap();

        let msg = HandleMsg::Relay { symbols: vec![String::from("MATIC")], rates: vec![112u64], resolve_times: vec![1625108298u64], request_ids: vec![124u64] };
        let _res = handle(&mut deps, mock_env("anyone", &[]), msg).unwrap();

        let msg = QueryMsg::GetReferenceData { base: String::from("MATIC"), quote: String::from("USD") };
        let res = query(&mut deps, msg).unwrap();
        let value: ReferenceData = from_binary(&res).unwrap();

        assert_eq!(ReferenceData{rate: Uint128::from(8928571428571428571428571u128), last_updated_base: Uint128::from(1625119856u128), last_updated_quote: Uint128::from(1625108298u128)}, value);
    }
}
