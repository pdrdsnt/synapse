use all_sol_types::sol_types::PoolKey;
use alloy::primitives::Address;

#[derive(Hash, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]

pub struct IdAddress {
    pub id: u64,
    pub address: Address,
}

pub struct IdKey {
    pub id: u64,
    pub key: PoolKey,
}
