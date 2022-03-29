#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw0::maybe_addr;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, PropertyResponse, QueryMsg};
use crate::state::{Property, State, ADMIN, BROKERS, PROPERTIES, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:real-estate";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State { count: 0 };
    ADMIN.set(deps.branch(), Some(info.sender.clone()))?;
    let storage = deps.storage;
    set_contract_version(storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(storage, &state)?;
    BROKERS.save(storage, &Vec::<Addr>::new())?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;
    match msg {
        ExecuteMsg::Increment {} => try_increment(deps),
        ExecuteMsg::MakeBroker { address } => try_make_broker(deps, info, address),
        ExecuteMsg::MakeOffer { property } => try_make_offer(deps, info, property),
        ExecuteMsg::UpdateAdmin { address } => {
            Ok(ADMIN.execute_update_admin(deps, info, maybe_addr(api, Some(address))?)?)
        }
    }
}

pub fn try_make_offer(
    deps: DepsMut,
    info: MessageInfo,
    property: Property,
) -> Result<Response, ContractError> {
    // check if sender is broker
    let brokers = BROKERS.load(deps.storage)?;

    // load offers count
    let offers_count = STATE.load(deps.storage)?.count;
    if !brokers.contains(&info.sender) {
        return Err(ContractError::NotBroker {});
    } else {
        PROPERTIES.save(deps.storage, offers_count + 1, &property)?;
    }

    let response = Response::new()
        .add_attribute("action", "make_offer")
        .add_attribute("offer_id", (offers_count + 1).to_string());

    Ok(response)
}

pub fn try_make_broker(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    // check if sender is admin
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    // validate address
    let validated_address = deps.api.addr_validate(&address)?;

    // check if address is already a broker
    BROKERS.update(deps.storage, |mut brokers| -> Result<_, ContractError> {
        if brokers.contains(&validated_address) {
            Err(ContractError::AlreadyBroker {})
        } else {
            brokers.push(validated_address.clone());
            Ok(brokers)
        }
    })?;

    Ok(Response::new()
        .add_attribute("action", "make_broker")
        .add_attribute("new_broker", validated_address))
}

pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.count += 1;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_increment"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOffer { id } => to_binary(&query_property(deps, id)?),
    }
}

pub fn query_property(deps: Deps, id: i32) -> StdResult<PropertyResponse> {
    // load property by id

    let property = PROPERTIES.may_load(deps.storage, id)?;
    match property {
        Some(prop) => Ok(PropertyResponse { property: prop }),
        None => Err(StdError::NotFound {
            kind: ContractError::PropertyNotFound {}.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::state::{PropertyRegion, PropertyType};

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 0 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOffer { id: 1 });
        match res {
            Ok(_) => panic!("Should not found any offers"),
            Err(_) => {}
        }
    }

    #[test]
    fn only_admins_can_make_brokers() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let msg = InstantiateMsg { count: 0 };
        let admin_info = mock_info("admin", &coins(1000, "earth"));
        let non_admin_info = mock_info("non_admin", &coins(1000, "earth"));

        instantiate(deps.as_mut(), mock_env(), admin_info.clone(), msg).unwrap();

        let make_broker_msg = ExecuteMsg::MakeBroker {
            address: "broker_candidate".to_string(),
        };
        let err_result = execute(
            deps.as_mut(),
            mock_env(),
            non_admin_info,
            make_broker_msg.clone(),
        );

        match err_result {
            Ok(_) => panic!("Should throw err 'Unauthorized'"),
            Err(_) => {}
        }

        execute(deps.as_mut(), mock_env(), admin_info, make_broker_msg).unwrap();
    }

    #[test]
    fn only_brokers_can_make_offers() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let msg = InstantiateMsg { count: 0 };
        let admin_info = mock_info("admin", &coins(1000, "earth"));
        instantiate(deps.as_mut(), mock_env(), admin_info.clone(), msg).unwrap();

        let make_broker_msg = ExecuteMsg::MakeBroker {
            address: "broker_candidate".to_string(),
        };

        execute(deps.as_mut(), mock_env(), admin_info, make_broker_msg).unwrap();

        let broker_info = mock_info("broker_candidate", &coins(1000, "earth"));
        let non_broker_info = mock_info("non_broker", &coins(1000, "earth"));

        let property = Property {
            property_type: PropertyType::OneRoom,
            region: PropertyRegion::Varna,
            squaring: "90kv".to_string(),
            construction: "Tuhla".to_string(),
            floor: "5".to_string(),
            description: None,
        };

        let make_offer_msg = ExecuteMsg::MakeOffer { property };

        let err_result = execute(
            deps.as_mut(),
            mock_env(),
            non_broker_info,
            make_offer_msg.clone(),
        );

        match err_result {
            Ok(_) => panic!("Should throw 'NotBroker' error "),
            Err(_) => {}
        }

        execute(deps.as_mut(), mock_env(), broker_info, make_offer_msg).unwrap();

        // get brokers count and compare
        let brokers = BROKERS.load(&deps.storage).unwrap();

        assert_eq!(1, brokers.len());

        // get property with id 1
        let property = PROPERTIES.load(&deps.storage, 1).unwrap();
        assert_eq!(PropertyType::OneRoom, property.property_type);
        assert_eq!(PropertyRegion::Varna, property.region);
        assert_eq!("90kv".to_string(), property.squaring);
        assert_eq!("Tuhla".to_string(), property.construction);
        assert_eq!("5".to_string(), property.floor);
        assert_eq!(None, property.description);
    }

    // #[test]
    // fn reset() {
    //     let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //     // should now be 5
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
    // }
}
