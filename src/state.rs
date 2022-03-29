use cw_controllers::Admin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Property {
    pub property_type: PropertyType,
    pub region: PropertyRegion,
    pub squaring: String,
    pub construction: String,
    pub floor: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PropertyType {
    OneRoom,
    TwoRoom,
    ThreeRoom,
    FourRoom,
    MultiRoom,
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
pub const BROKERS: Item<Vec<Addr>> = Item::new("brokers");
pub const PROPERTIES: Map<i32, Property> = Map::new("properties");
pub const ADMIN: Admin = Admin::new("admin");
