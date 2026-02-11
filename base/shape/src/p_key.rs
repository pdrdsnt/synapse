use alloy::primitives::{Address, B256};
use std::collections::HashSet;

use crate::id_address::IdAddress;

#[derive(Hash, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum AnyPoolKey {
    V2(IdAddress),
    V3(IdAddress),
    V4(u64, B256),
}
