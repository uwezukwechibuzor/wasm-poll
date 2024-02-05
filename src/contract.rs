#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetPollResponse, InstantiateMsg, QueryMsg};
use crate::state::{Config, Poll, CONFIG, POLLS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:wasm-poll";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let validated_admin_address: Addr = deps.api.addr_validate(&msg.admin_address)?;

    let config = Config {
        admin_address: validated_admin_address,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePoll { question } => execute_create_poll(deps, env, info, question),
        ExecuteMsg::Vote { question, choice } => execute_vote(deps, env, info, question, choice),
    }
}

fn execute_create_poll(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    question: String,
) -> Result<Response, ContractError> {
    if POLLS.has(deps.storage, question.clone()) {
        return Err(ContractError::CustomError {
            val: "Key already taken!".to_string(),
        });
    }

    let poll = Poll {
        question: question.clone(),
        yes_votes: 0,
        no_votes: 0,
    };

    POLLS.save(deps.storage, question, &poll)?;

    Ok(Response::new().add_attribute("action", "create_poll"))
}

fn execute_vote(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    question: String,
    choice: String,
) -> Result<Response, ContractError> {
    // check for if the poll exist
    if !POLLS.has(deps.storage, question.clone()) {
        return Err(ContractError::CustomError {
            val: "Poll does not exist!".to_string(),
        });
    }

    let mut poll = POLLS.load(deps.storage, question.clone())?;

    // use match
    // match choice {
    //   "yes" => {}
    //   "no" => {}
    //   _ =>  Err(ContractError::CustomError { val: "Unrecognised choice!".to_string(), });
    // }

    if choice != "yes" && choice != "no" {
        return Err(ContractError::CustomError {
            val: "Unrecognised choice!".to_string(),
        });
    } else {
        if choice == "yes" {
            poll.yes_votes += 1;
        } else {
            poll.no_votes += 1;
        }

        POLLS.save(deps.storage, question, &poll)?;
        Ok(Response::new().add_attribute("action", "vote"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPoll { question } => query_get_poll(deps, _env, question),
        QueryMsg::GetConfig {} => to_json_binary(&CONFIG.load(deps.storage)?),
    }
}

fn query_get_poll(deps: Deps, _env: Env, question: String) -> StdResult<Binary> {
    let poll = POLLS.may_load(deps.storage, question)?;
    to_json_binary(&GetPollResponse { poll })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        attr, from_json,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };

    use crate::{
        contract::query,
        msg::{ExecuteMsg, GetPollResponse, InstantiateMsg, QueryMsg},
        state::{Config, Poll},
    };

    use super::{execute, instantiate};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr", &[]);
        let msg = InstantiateMsg {
            admin_address: "addr".to_string(),
        };

        let resp = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(resp.attributes, vec![attr("action", "instantiate")]);

        let msg = QueryMsg::GetConfig {};
        let resp = query(deps.as_ref(), env, msg).unwrap();
        let config: Config = from_json(&resp).unwrap();
        assert_eq!(
            config,
            Config {
                admin_address: Addr::unchecked("addr")
            }
        );
    }

    #[test]
    fn test_create_poll() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr", &[]);
        let msg = InstantiateMsg {
            admin_address: "addr".to_string(),
        };

        let _resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::CreatePoll {
            question: "I did it, I can learn anything".to_string(),
        };

        let resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(resp.attributes, vec![attr("action", "create_poll")]);

        // test for Query
        let msg = QueryMsg::GetPoll {
            question: "I did it, I can learn anything".to_string(),
        };
        let resp = query(deps.as_ref(), env.clone(), msg).unwrap();
        let get_poll_response: GetPollResponse = from_json(&resp).unwrap();
        assert_eq!(
            get_poll_response,
            GetPollResponse {
                poll: Some(Poll {
                    question: "I did it, I can learn anything".to_string(),
                    yes_votes: 0,
                    no_votes: 0
                })
            }
        );

        // test for if same key is used
        let msg = ExecuteMsg::CreatePoll {
            question: "I did it, I can learn anything".to_string(),
        };

        let _resp = execute(deps.as_mut(), env, info, msg).unwrap_err();
    }

    #[test]
    fn test_vote() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr", &[]);
        let msg = InstantiateMsg {
            admin_address: "addr".to_string(),
        };

        let _resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::CreatePoll {
            question: "I did it, I can learn anything".to_string(),
        };

        let _resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // success cases, we vote on a poll that exists
        let msg = ExecuteMsg::Vote {
            question: "I did it, I can learn anything".to_string(),
            choice: "yes".to_string(),
        };
        let resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(resp.attributes, vec![attr("action", "vote")]);

        // Error cases, vote on a poll that does not exist
        let msg = ExecuteMsg::Vote {
            question: "I did it, I can learn anything????".to_string(),
            choice: "yes".to_string(),
        };
        let _resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();

        // Error cases, vote on a poll that doe exist but with an invalid choice
        let msg = ExecuteMsg::Vote {
            question: "I did it, I can learn anything".to_string(),
            choice: "maybe".to_string(),
        };
        let _resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
    }
}
