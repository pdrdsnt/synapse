use std::{
    hash::Hash,
    sync::{Arc, Mutex, RwLock},
};

use all_sol_types::sol_types::{IUniswapV2Pair, V3Pool};
use alloy::{
    primitives::{Log, map::HashMap},
    rpc::types::state,
};
use cortex::{
    cortex::Cortex,
    types::{AnyPartialPool, PartialV2Pool, PartialV3Pool, PartialV4Pool, PoolEvaluation},
};
use dashmap::{DashMap, Map};
use shape::{
    id_address::{IdAddress, IdKey},
    p_config::V3Config,
    p_key::AnyPoolKey,
    p_state::V3State,
};

pub struct MasterContext {
    v2_pools: DashMap<IdAddress, PartialV2Pool>,
    v3_pools: DashMap<IdAddress, PartialV3Pool>,
    v4_pools: DashMap<IdKey, PartialV4Pool>,

    pools_by_token: DashMap<IdAddress, Vec<EvaluatedPool>>,

    v2_reserves_queue: Arc<Mutex<Vec<IdAddress>>>,
}

pub struct EvaluatedPool {
    pool: AnyPartialPool,
    eval: Option<PoolEvaluation>,
}

impl MasterContext {
    pub fn handle_v2_swap(
        &self,
        log: Log<all_sol_types::sol_types::IUniswapV2Pair::Swap>,
        chain_id: u64,
    ) {
        let key = IdAddress {
            id: chain_id,
            address: log.address,
        };

        let mut needs_update = false;

        self.v2_pools
            .entry(key)
            .and_modify(|x| {
                if let Some(mut stt) = x.state.as_mut() {
                    stt.r0 += log.amount0In.to::<u128>();
                    stt.r1 += log.amount1In.to::<u128>();
                    stt.r0 -= log.amount0Out.to::<u128>();
                    stt.r1 -= log.amount1Out.to::<u128>();
                } else {
                    needs_update = true;
                }
            })
            .or_insert_with(|| PartialV2Pool {
                chain: chain_id,
                address: log.address,
                config: None,
                state: None,
            });

        if needs_update {
            self.v2_reserves_queue.lock().unwrap().push(IdAddress {
                id: chain_id,
                address: log.address,
            });
        }
    }

    pub fn handle_v3_swap(&self, log: &Log<V3Pool::Swap>, chain_id: u64) {
        let key = IdAddress {
            id: chain_id,
            address: log.address,
        };

        self.v3_pools
            .entry(key)
            .and_modify(|x| match x.state.as_mut() {
                Some(stt) => {
                    stt.liquidity = log.liquidity;
                    stt.tick = log.tick;
                    stt.x96price = log.sqrtPriceX96;
                }
                None => {
                    let p = PartialV3Pool {
                        chain: chain_id,
                        address: log.address,
                        config: x.config.clone(),
                        state: Some(V3State {
                            tick: log.tick,
                            x96price: log.sqrtPriceX96,
                            liquidity: log.liquidity,
                        }),
                    };
                }
            })
            .or_insert_with(|| PartialV3Pool {
                chain: chain_id,
                address: log.address,
                config: None,
                state: None,
            });
    }

    pub fn handle_v4_swap(&self) {}
}
