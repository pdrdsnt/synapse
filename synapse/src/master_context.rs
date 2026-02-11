use all_sol_types::sol_types::IUniswapV2Pair;
use alloy::primitives::Log;
use dashmap::DashMap;

pub struct MasterContext {
    pools: DashMap<shape::id_address::IdAddress, shape::p_any::AnyPoolShape>,
}

impl MasterContext {
    pub fn handle_v2_swap(&self, log: Log<IUniswapV2Pair::Swap>) {}
    pub fn handle_v3_swap(&self) {}
    pub fn handle_v4_swap(&self) {}
}
