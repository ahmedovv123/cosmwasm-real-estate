use cw_controllers::Admin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Brokers {
    pub addresses: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Property {
    propery_type: PropertyType,
    region: PropertyRegion,
    squaring: String,
    construction: String,
    floor: String,
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PropertyType {
    OneRoom {},
    TwoRoom {},
    ThreeRoom {},
    FourRoom {},
    MultiRoom {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PropertyRegion {
    Varna,
    Byala,
    Sofia,
    Razgrad,
    Obzor,
    Burgas,
    Plovdiv,
}

pub const STATE: Item<State> = Item::new("state");
pub const BROKERS: Item<Brokers> = Item::new("brokers");
pub const PROPERTIES: Map<i32, Property> = Map::new("properties");
pub const ADMIN: Admin = Admin::new("admin");
