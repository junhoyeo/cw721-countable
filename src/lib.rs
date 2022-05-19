pub mod msg;
pub mod state;

use crate::msg::{Cw721MetadataContract, ExecuteMsg, Extension, Metadata, MintMsgInput};
use crate::state::{State, STATE};

use cw721_base::state::TokenInfo;
pub use cw721_base::{ContractError, InstantiateMsg, MintMsg, MinterResponse, QueryMsg};

#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;

    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

    // This is a simple type to let us handle empty extensions

    // This makes a conscious choice on the various generics used by the contract
    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        let state = State { token_count: 1 };
        STATE.save(deps.storage, &state)?;
        Cw721MetadataContract::default().instantiate(deps, env, info, msg)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
            ExecuteMsg::Mint(mint_msg) => execute_mint(deps, env, info, mint_msg),
            _ => Cw721MetadataContract::default().execute(
                deps,
                env,
                info,
                cw721_base::ExecuteMsg::<std::option::Option<Metadata>>::from(msg),
            ),
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Cw721MetadataContract::default().query(deps, env, msg)
    }

    pub fn execute_mint(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        mint_msg_input: MintMsgInput<Extension>,
    ) -> Result<Response, ContractError> {
        let cw721_contract = Cw721MetadataContract::default();

        let _state = STATE.load(deps.storage)?;
        let token_id = _state.token_count.to_string();
        STATE.update(deps.storage, |mut __state| -> Result<_, ContractError> {
            __state.token_count += 1;
            Ok(__state)
        })?;

        let response = cw721_contract.mint(
            deps,
            env,
            info,
            MintMsg {
                token_id,
                token_uri: mint_msg_input.token_uri,
                owner: mint_msg_input.owner,
                extension: mint_msg_input.extension,
            },
        )?;
        Ok(response)
    }

    pub fn execute_burn(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response, ContractError> {
        let cw721_contract = Cw721MetadataContract::default();

        let token = cw721_contract.tokens.load(deps.storage, &token_id)?;
        _check_can_send(&cw721_contract, deps.as_ref(), &env, &info, &token)?;

        cw721_contract.tokens.remove(deps.storage, &token_id)?;
        cw721_contract
            .token_count
            .update(deps.storage, |count| -> Result<u64, ContractError> {
                Ok(count - 1)
            })?;

        Ok(Response::new()
            .add_attribute("action", "burn")
            .add_attribute("token_id", token_id))
    }

    fn _check_can_send<T>(
        cw721_contract: &Cw721MetadataContract,
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        token: &TokenInfo<T>,
    ) -> Result<(), ContractError> {
        if token.owner == info.sender {
            return Ok(());
        }

        if token
            .approvals
            .iter()
            .any(|apr| apr.spender == info.sender && !apr.is_expired(&env.block))
        {
            return Ok(());
        }

        let op = cw721_contract
            .operators
            .may_load(deps.storage, (&token.owner, &info.sender))?;
        match op {
            Some(ex) => {
                if ex.is_expired(&env.block) {
                    Err(ContractError::Unauthorized {})
                } else {
                    Ok(())
                }
            }
            None => Err(ContractError::Unauthorized {}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw721::Cw721Query;

    const CREATOR: &str = "creator";

    #[test]
    fn use_metadata_extension() {
        let mut deps = mock_dependencies(&[]);
        let contract = Cw721MetadataContract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            minter: CREATOR.to_string(),
        };
        contract
            .instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)
            .unwrap();

        let token_id = "Enterprise";
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: "john".to_string(),
            token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
            extension: Some(Metadata {
                description: Some("Spaceship with Warp Drive".into()),
                name: Some("Starship USS Enterprise".to_string()),
                ..Metadata::default()
            }),
        };
        let exec_msg = cw721_base::ExecuteMsg::Mint(mint_msg.clone());
        contract
            .execute(deps.as_mut(), mock_env(), info, exec_msg)
            .unwrap();

        let res = contract.nft_info(deps.as_ref(), token_id.into()).unwrap();
        assert_eq!(res.token_uri, mint_msg.token_uri);
        assert_eq!(res.extension, mint_msg.extension);
    }
}
