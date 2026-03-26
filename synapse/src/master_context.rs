use std::{
    hash::Hash,
    sync::{Arc, Mutex, RwLock},
};

use all_sol_types::sol_types::{
    IPoolManager::{self, IPoolManagerInstance},
    IPositionManager::IPositionManagerInstance,
    IUniswapV2Pair, PoolKey,
    StateView::StateViewInstance,
    V3Pool,
};
use alloy::{
    primitives::{B256, U256, map::HashMap},
    providers::{Provider, RootProvider, fillers::FillProvider},
    rpc::types::{Log, state},
};
use chains_json::chain_json_model::ChainDataJsonModel;
use cortex::{
    cortex::{Cortex, WsProvider},
    types::{AnyPartialPool, PartialV2Pool, PartialV3Pool, PartialV4Pool, PoolEvaluation},
};
use dashmap::{DashMap, Map};
use futures::{SinkExt, channel::mpsc::Receiver, executor::block_on, stream::FuturesOrdered};
use shape::{
    id_address::{IdAddress, IdKey},
    p_config::{V3Config, V4Config},
    p_key::AnyPoolKey,
    p_state::V3State,
};

use crate::{
    calls::{self, get_v4_key},
    v4_fetcher::{self, V4Contracts, V4FetchArgs, V4Fetcher},
};

pub struct MasterContext {
    chains_providers: DashMap<u64, WsProvider>,
    v2_pools: DashMap<IdAddress, PartialV2Pool>,
    v3_pools: DashMap<IdAddress, PartialV3Pool>,
    v4_pools: DashMap<IdKey, PartialV4Pool>,
    v4_fetch_worker: Arc<RwLock<V4Fetcher>>,
    pools_by_token: DashMap<IdAddress, Vec<EvaluatedPool>>,
    v4_contracts: DashMap<u64, V4Contracts<WsProvider>>,
    v2_reserves_queue: Arc<RwLock<Vec<IdAddress>>>,
}

pub struct EvaluatedPool {
    pool: AnyPartialPool,
    eval: Option<PoolEvaluation>,
}

impl MasterContext {
    async fn new() -> Self {
        let v4_fetcher_worker = V4Fetcher {
            contracts: todo!(),
            pools: todo!(),
            not_found: todo!(),
            sender: todo!(),
        };

        Self {
            chains_providers: DashMap::new(),
            v2_pools: DashMap::new(),
            v3_pools: DashMap::new(),
            v4_pools: DashMap::new(),
            v4_fetch_worker,
            pools_by_token: DashMap::new(),
            v4_contracts: DashMap::new(),
            v2_reserves_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn v4_fetch(args: V4FetchArgs, ctx: &MasterContext) {
        let (id, chain) = (args.id, args.chain);
        if let Some(v4_contracts) = ctx.v4_contracts.get(&chain) {
            let pos_man = v4_contracts.value().position_manager.clone();
            if let Ok(key) = get_v4_key(id, pos_man).await {
                let ik = IdKey { id: chain, key: id };
                ctx.v4_pools
                    .entry(ik)
                    .and_modify(|x| x.config = Some(key.clone().into()))
                    .or_insert_with(|| PartialV4Pool {
                        chain,
                        state_view: v4_contracts.state_view.address().clone(),
                        config: Some(key.into()),
                        state: None,
                    });
            }
        };
    }

    async fn spaw_v4_fetch_worker(ctx: &MasterContext, mut rx: Receiver<V4FetchArgs>) {
        let worker = std::thread::scope(|_| async move {
            while let Ok(r) = rx.recv().await {
                let Some(v4_contracts) = ctx.v4_contracts.get(&r.chain) else {
                    continue;
                };

                let state_view = *v4_contracts.state_view.address();
                let log = r.log.inner;

                let Some(v4_contracts) = ctx.v4_contracts.get(&r.chain) else {
                    continue;
                };

                let Ok(call) = get_v4_key(r.id, v4_contracts.position_manager.clone()).await else {
                    continue;
                };

                let config = V4Config {
                    fee: call.fee,
                    tick_spacing: call.tickSpacing,
                    hooks: call.hooks,
                    token0: call.currency0,
                    token1: call.currency1,
                };

                let state = V3State {
                    tick: log.tick,
                    x96price: log.sqrtPriceX96,
                    liquidity: log.liquidity,
                };

                PartialV4Pool {
                    chain: r.chain,
                    state_view,
                    config: Some(config),
                    state: Some(state),
                };
            }
        })
        .await;
    }

    pub async fn handle_v4_swap(
        &self,
        log: Log<all_sol_types::sol_types::IPoolManager::Swap>,
        chain_id: u64,
    ) {
        let key = IdKey {
            id: chain_id,
            key: log.inner.data.id,
        };

        let args = V4FetchArgs {
            id: log.inner.id,
            chain: chain_id,
            log: log,
        };

        let write_result = self
            .v4_fetch_worker
            .write()
            .unwrap()
            .handle_v4_swap(args, chain_id)
            .await;
    }

    pub fn handle_v2_swap(
        &self,
        log: Log<all_sol_types::sol_types::IUniswapV2Pair::Swap>,
        chain_id: u64,
    ) {
        let key = IdAddress {
            id: chain_id,
            address: log.address(),
        };

        let mut needs_update = false;

        self.v2_pools
            .entry(key)
            .and_modify(|x| {
                if let Some(stt) = x.state.as_mut() {
                    stt.r0 += log.inner.amount0In.to::<u128>();
                    stt.r1 += log.inner.amount1In.to::<u128>();
                    stt.r0 -= log.inner.amount0Out.to::<u128>();
                    stt.r1 -= log.inner.amount1Out.to::<u128>();
                } else {
                    needs_update = true;
                }
            })
            .or_insert_with(|| PartialV2Pool {
                chain: chain_id,
                address: log.address(),
                config: None,
                state: None,
            });

        if needs_update {
            if let Ok(mut lock) = self.v2_reserves_queue.write() {
                lock.push(IdAddress {
                    id: chain_id,
                    address: log.address(),
                });
            }
        }
    }

    pub fn handle_v3_swap(&self, log: Log<V3Pool::Swap>, chain_id: u64) {
        let key = IdAddress {
            id: chain_id,
            address: log.address(),
        };

        self.v3_pools
            .entry(key)
            .and_modify(|x| match x.state.as_mut() {
                Some(stt) => {
                    stt.liquidity = log.inner.liquidity;
                    stt.tick = log.inner.tick;
                    stt.x96price = log.inner.sqrtPriceX96;
                }
                None => {
                    let p = PartialV3Pool {
                        chain: chain_id,
                        address: log.address(),
                        config: x.config.clone(),
                        state: Some(V3State {
                            tick: log.inner.tick,
                            x96price: log.inner.sqrtPriceX96,
                            liquidity: log.inner.liquidity,
                        }),
                    };
                }
            })
            .or_insert_with(|| PartialV3Pool {
                chain: chain_id,
                address: log.address(),
                config: None,
                state: None,
            });
    }
}

impl From<ChainDataJsonModel> for MasterContext {
    fn from(value: ChainDataJsonModel) -> Self {}
}
