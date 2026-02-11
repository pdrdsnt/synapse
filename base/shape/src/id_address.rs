use alloy::primitives::Address;

#[derive(Hash, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]

pub struct IdAddress {
    pub id: u64,
    pub address: Address,
}
