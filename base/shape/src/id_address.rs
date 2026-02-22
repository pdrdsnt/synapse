use all_sol_types::sol_types::PoolKey;
use alloy::primitives::{Address, B256};

#[derive(Hash, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]

pub struct IdAddress {
    pub id: u64,
    pub address: Address,
}

#[derive(Hash, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct IdKey {
    pub id: u64,
    pub key: B256,
}
