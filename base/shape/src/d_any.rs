use alloy::primitives::{Address, aliases::U24};

#[derive(Debug)]
pub enum AnyDexShape {
    V2(FullV2Dex),
    V3(FullV3Dex),
    V4(FullV4Dex),
}

#[derive(Debug)]
pub struct V2Fees {
    pub crypto: U24,
    pub stable: Option<U24>,
}

#[derive(Debug)]
pub struct V3Fees {
    pub tiers: Vec<U24>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DexId {
    Uniswap,
    Sushiswap,
    Pancake,
    Curve,
    Balancer,
    Unknown,
}

#[derive(Debug)]
pub struct FullV2Dex {
    pub chain: u64,
    pub address: Address,
    pub id: DexId,
    pub fees: V2Fees,
}

#[derive(Debug)]
pub struct FullV3Dex {
    pub chain: u64,
    pub address: Address,
    pub id: DexId,
    pub fees: V3Fees,
}

#[derive(Debug)]
pub struct FullV4Dex {
    pub chain: u64,
    pub address: Address,
    pub id: DexId,
}
