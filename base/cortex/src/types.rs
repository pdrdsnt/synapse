use std::os::unix;

use alloy::primitives::Address;
use shape::{
    d_any::{DexId, V2Fees, V3Fees},
    p_config::{V2Config, V3Config, V4Config},
    p_state::{V2State, V3State},
};

#[derive(Debug)]
pub struct PartialV2Pool {
    pub chain: u64,
    pub address: Address,
    pub config: Option<V2Config>,
    pub state: Option<V2State>,
}

#[derive(Debug)]
pub struct PartialV3Pool {
    pub chain: u64,
    pub address: Address,
    pub config: Option<V3Config>,
    pub state: Option<V3State>,
}

#[derive(Debug)]
pub struct PartialV4Pool {
    pub chain: u64,
    pub address: Address,
    pub config: Option<V4Config>,
    pub state: Option<V3State>,
}

#[derive(Debug)]
pub struct PartialV2Dex {
    pub chain: u64,
    pub address: Address,
    pub id: Option<DexId>,
    pub fees: Option<V2Fees>,
}

#[derive(Debug)]
pub struct PartialV3Dex {
    pub chain: u64,
    pub address: Address,
    pub id: Option<DexId>,
    pub fees: Option<V3Fees>,
}

#[derive(Debug)]
pub struct PartialV4Dex {
    pub chain: u64,
    pub state_view: Address,
    pub id: Option<DexId>,
}
#[derive(Debug)]
pub enum AnyPartialPool {
    V2(PartialV2Pool),
    V3(PartialV3Pool),
    V4(PartialV4Pool),
}

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct PoolEvaluation {
    timestamp: SystemTime,
    idk: u128,
}
