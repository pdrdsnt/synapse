use std::{collections::HashMap, str::FromStr, time::Duration};

use all_sol_types::sol_types::{IPoolManager, IUniswapV2Pair, StateView, V3Pool};
use alloy::{
    hex::HEX_DECODE_LUT,
    primitives::B256,
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::{client::RpcClient, types::Filter},
    signers::k256::U256,
    sol_types::SolEvent,
    transports::{RpcError, TransportErrorKind, http::reqwest::Url},
};
use chains_json::{chain::ChainJsonInput, chains::ChainsJsonInput};
//use chains_json::chains::ChainsJsonInput;
use dashmap::DashMap;
use shape::id_address::IdAddress;

use crate::{
    master_context::MasterContext,
    pool_event::{
        UnifiedPoolEvent, UnifiedPoolEventResponse, generate_pool_events, generate_pools_events_map,
    },
};

mod master_context;
mod pool_event;
mod token_event;

pub type WsProvider = alloy::providers::fillers::FillProvider<
    alloy::providers::fillers::JoinFill<
        alloy::providers::Identity,
        alloy::providers::fillers::JoinFill<
            alloy::providers::fillers::GasFiller,
            alloy::providers::fillers::JoinFill<
                alloy::providers::fillers::BlobGasFiller,
                alloy::providers::fillers::JoinFill<
                    alloy::providers::fillers::NonceFiller,
                    alloy::providers::fillers::ChainIdFiller,
                >,
            >,
        >,
    >,
    alloy::providers::RootProvider,
>;

fn main() {
    println!("Hello, world!");
}

pub async fn ws_sub<P: Provider + Clone>(
    provider: P,
    filter: Filter,
) -> Result<alloy::pubsub::Subscription<alloy::rpc::types::Log>, RpcError<TransportErrorKind>> {
    let r = provider.subscribe_logs(&filter).await?;
    return Ok(r);
}

pub async fn ws_provider(url: Url) -> Result<WsProvider, RpcError<TransportErrorKind>> {
    let ws_connect: WsConnect = WsConnect::new(url.clone());

    match RpcClient::connect_pubsub(ws_connect).await {
        Ok(rpc_client) => {
            println!("ws connection {:?}", &url);
            let provider: WsProvider = ProviderBuilder::new().connect_client(rpc_client);
            Ok(provider)
        }

        Err(err) => Err(err),
    }
}

async fn watch_chains() {
    let waiting_time = 2;
    let listening_time = 2;

    let chains = ChainsJsonInput::default();
    let pool_events: HashMap<B256, UnifiedPoolEvent> = generate_pools_events_map();

    let available_chains = chains.chains;

    let master_context = for (idx, x) in available_chains.iter() {
        for url_str in x.ws_nodes_urls.iter() {
            let url = Url::from_str(url_str).unwrap();
            let ws_provider = match ws_provider(url).await {
                Ok(ok) => ok,
                Err(err) => {
                    panic!("ws provider creation failed: {:?}", err)
                }
            };
        }
    };

    println!("exiting");
}

#[derive(Default)]
pub struct ChainState<P: Provider> {
    providers: P,
    id: u64,
    priority: u32,
    last_checked: Duration,
    liquidity: U256,
}

pub async fn decode_logs_listener_blocking<P: Provider + Clone>(
    chain_id: u64,
    provider: P,
    ctx: &MasterContext,
) {
    let map = generate_pools_events_map();
    let filter = Filter::new().events(generate_pool_events());
    if let Ok(mut ws) = provider.subscribe_logs(&filter).await {
        while let Ok(log) = ws.recv().await {
            if let Some(w) = log.topic0() {
                if let Some(res) = map.get(w) {
                    let response = match res {
                        // Uniswap V2
                        UnifiedPoolEvent::V2Mint() => log
                            .log_decode::<all_sol_types::sol_types::IUniswapV2Pair::Mint>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V2Mint),
                        UnifiedPoolEvent::V2Burn() => log
                            .log_decode::<all_sol_types::sol_types::IUniswapV2Pair::Burn>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V2Burn),
                        UnifiedPoolEvent::V2Swap() => log
                            .log_decode::<all_sol_types::sol_types::IUniswapV2Pair::Swap>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V2Swap),
                        UnifiedPoolEvent::V2Sync() => log
                            .log_decode::<IUniswapV2Pair::Sync>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V2Sync),
                        UnifiedPoolEvent::V2Approval() => log
                            .log_decode::<IUniswapV2Pair::Approval>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V2Approval),
                        UnifiedPoolEvent::V2Transfer() => log
                            .log_decode::<IUniswapV2Pair::Transfer>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V2Transfer),
                        // Uniswap V3
                        UnifiedPoolEvent::V3Mint() => log
                            .log_decode::<V3Pool::Mint>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V3Mint),
                        UnifiedPoolEvent::V3Swap() => log
                            .log_decode::<V3Pool::Swap>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V3Swap),
                        UnifiedPoolEvent::V3Collect() => log
                            .log_decode::<V3Pool::Collect>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V3Collect),
                        UnifiedPoolEvent::V3Burn() => log
                            .log_decode::<V3Pool::Burn>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V3Burn),
                        UnifiedPoolEvent::V3Flash() => log
                            .log_decode::<V3Pool::Flash>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V3Flash),
                        // Uniswap V4 (PoolManager Events)
                        UnifiedPoolEvent::V4Swap() => log
                            .log_decode::<IPoolManager::Swap>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V4Swap),
                        UnifiedPoolEvent::V4Modify() => log
                            .log_decode::<IPoolManager::ModifyLiquidity>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V4Modify),
                        UnifiedPoolEvent::V4Donate() => log
                            .log_decode::<IPoolManager::Donate>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V4Donate),
                        UnifiedPoolEvent::V4Initialize() => log
                            .log_decode::<IPoolManager::Initialize>()
                            .ok()
                            .map(UnifiedPoolEventResponse::V4Initialize),
                    };

                    if let Some(r) = response {
                        r.handle(ctx, chain_id);
                    }
                }
            }
        }
    }
}
