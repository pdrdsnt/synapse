use all_sol_types::sol_types::PoolKey;
use alloy::primitives::{
    Address, B256,
    aliases::{I24, U24},
    hex::encode,
    keccak256,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum AnyPoolConfig {
    V2(V2Config),
    V3(V3Config),
    V4(V4Config),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]

pub struct V2Config {
    pub name: String,
    pub fee: U24,
    pub token0: Address,
    pub token1: Address,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct V3Config {
    pub name: String,
    pub fee: U24,
    pub tick_spacing: I24,
    pub token0: Address,
    pub token1: Address,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct V4Config {
    pub fee: U24,
    pub tick_spacing: I24,
    pub hooks: Address,
    pub token0: Address,
    pub token1: Address,
}

impl V4Config {
    pub fn to_key(&self) -> PoolKey {
        PoolKey {
            currency0: self.token0,
            currency1: self.token1,
            fee: self.fee,
            tickSpacing: self.tick_spacing,
            hooks: self.hooks,
        }
    }
}

impl From<PoolKey> for V4Config {
    fn from(value: PoolKey) -> Self {
        Self {
            token0: value.currency0,
            token1: value.currency1,
            fee: value.fee,
            tick_spacing: value.tickSpacing,
            hooks: value.hooks,
        }
    }
}
