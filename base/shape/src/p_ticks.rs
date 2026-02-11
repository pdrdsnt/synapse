use alloy::primitives::{U256, aliases::I24};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PoolWords {
    pub words: BTreeMap<i16, TicksBitMap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicksBitMap {
    pub bitmap: U256,
    pub ticks: BTreeMap<I24, TickData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickData {
    pub liquidity_net: Option<i128>,
}
