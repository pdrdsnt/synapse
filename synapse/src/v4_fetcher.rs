use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use all_sol_types::sol_types::{
    IPoolManager::IPoolManagerInstance, IPositionManager::IPositionManagerInstance,
    StateView::StateViewInstance,
};
use alloy::{primitives::B256, providers::Provider, rpc::types::Log};
use chains_json::{chain_json_model::DexJsonModel, chains::ChainsJsonInput};
use cortex::{cortex::WsProvider, types::PartialV4Pool};
use dashmap::DashMap;
use futures::{SinkExt, channel::mpsc::Receiver, executor::block_on, future, lock::Mutex};
use shape::{id_address::IdKey, p_config::V4Config, p_state::V3State};

use crate::calls::get_v4_key;

pub struct V4Fetcher {
    contracts: DashMap<u64, V4Contracts<WsProvider>>,
    pools: DashMap<IdKey, PartialV4Pool>,
    not_found: Arc<RwLock<Vec<IdKey>>>,
    sender: Arc<Mutex<futures::channel::mpsc::Sender<V4FetchArgs>>>,
}

impl V4Fetcher {
    async fn new(data: ChainsJsonInput) -> Self {
        let (tx, rx) = futures::channel::mpsc::channel(10000);

        let mut providers: HashMap<u64, String>;

        let mut s = Self {
            contracts: todo!(),
            pools: todo!(),
            not_found: todo!(),
            sender: todo!(),
        };

        for (id, chain) in data.chains {
            chain.dexes.iter().map(|dex| {});
        }

        let worker = std::thread::spawn(move || {
            Self::blocking_receive(&s, rx);
        });

        s
    }

    async fn update_v4(
        &mut self,
        dex: chains_json::chain_json_model::DexJsonModel,
        contracts: Ordered
    ) -> Option<chains_json::chain_json_model::DexJsonModel> {
        match dex {
            chains_json::chain_json_model::DexJsonModel::V2 {
                address,
                fee,
                stable_fee,
            } => return None,
            chains_json::chain_json_model::DexJsonModel::V3 { address, fee } => return None
            chains_json::chain_json_model::DexJsonModel::V4 {
                state_view,
                pool_manager,
                postion_descriptor,
                position_manager,
                quoter,
                universal_router,
                permit2,
            } => {
                return Some(DexJsonModel::V4 { state_view: (), pool_manager: (), postion_descriptor: (), position_manager: (), quoter: (), universal_router: (), permit2: () } )
            },
        };
    }

    async fn fetch_and_update(&self, args: V4FetchArgs) {
        let (id, chain) = (args.id, args.chain);
        if let Some(v4_contracts) = self.contracts.get(&chain) {
            let pos_man = v4_contracts.value().position_manager.clone();
            if let Ok(key) = get_v4_key(id, pos_man).await {
                let ik = IdKey { id: chain, key: id };
                self.pools
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

    pub fn spaw_fetch_worker(&self) {}

    pub fn blocking_receive(&self, mut rx: Receiver<V4FetchArgs>) {
        block_on(async move {
            while let Ok(r) = rx.recv().await {
                let Some(v4_contracts) = self.contracts.get(&r.chain) else {
                    continue;
                };

                let state_view = *v4_contracts.state_view.address();
                let log = r.log.inner;

                let Some(v4_contracts) = self.contracts.get(&r.chain) else {
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
        });
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

        if let Err(err) = self.sender.lock().await.send(args).await {
            println!("error {}", err);
        }
    }
}

pub struct V4FetchArgs {
    id: B256,
    chain: u64,
    log: Log<all_sol_types::sol_types::IPoolManager::Swap>,
}

pub struct V4Contracts<P: Provider + Clone + Send + Sync> {
    state_view: StateViewInstance<P>,
    position_manager: IPositionManagerInstance<P>,
    pools_manager: IPoolManagerInstance<P>,
}
