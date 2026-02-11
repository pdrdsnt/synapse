use alloy::primitives::Address;

use crate::{
    p_config::{V2Config, V3Config, V4Config},
    p_state::{V2State, V3State},
    p_ticks::PoolWords,
};

#[derive(Debug)]
pub enum AnyPoolShape {
    V2(FullV2Pool),
    V3(FullV3Pool),
    V4(FullV4Pool),
}

#[derive(Debug)]
pub struct FullV2Pool {
    pub chain: u64,
    pub address: Address,
    pub config: V2Config,
    pub state: V2State,
}

#[derive(Debug)]
pub struct FullV3Pool {
    pub chain: u64,
    pub address: Address,
    pub config: V3Config,
    pub state: V3State,
}

#[derive(Debug)]
pub struct FullV4Pool {
    pub chain: u64,
    pub address: Address,
    pub config: V4Config,
    pub state: V3State,
}
