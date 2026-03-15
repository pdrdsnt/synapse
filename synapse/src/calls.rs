use std::collections::BTreeMap;

use all_sol_types::sol_types::{
    IERC1155::balanceOfReturn,
    IPoolManager::IPoolManagerCalls,
    IPositionManager::IPositionManagerInstance,
    IUniswapV2Pair::{IUniswapV2PairInstance, getReservesReturn},
    PoolKey,
    StateView::{StateViewInstance, getSlot0Return},
    V3Pool::{V3PoolInstance, slot0Return},
};
use alloy::primitives::{B256, aliases::I24};
use alloy::providers::Provider;
use alloy_sol_types::SolValue;
use futures::{
    future::join_all,
    stream::{FuturesOrdered, FuturesUnordered, StreamExt},
};
use shape::p_ticks::{TickData, TicksBitMap};
use v3::v3_base::bitmap_math;

use crate::master_context::MasterContext;

async fn get_v2_reserves<P: Provider + Clone>(
    pool: IUniswapV2PairInstance<P>,
) -> Option<getReservesReturn> {
    if let Ok(reserves) = pool.getReserves().call().await {
        return Some(reserves);
    };
    None
}

async fn get_v3_slot0<P: Provider + Clone>(pool: V3PoolInstance<P>) -> Option<slot0Return> {
    if let Ok(slot0) = pool.slot0().call().await {
        return Some(slot0);
    };
    None
}

async fn get_v3_liquidity<P: Provider + Clone>(
    pool: V3PoolInstance<P>,
    word: i16,
    tick_spacing: I24,
) -> Option<u128> {
    if let Ok(liq) = pool.liquidity().call().await {
        return Some(liq);
    };
    None
}

async fn get_v3_word_ticks<P: Provider + Clone>(
    pool: V3PoolInstance<P>,
    word: i16,
    tick_spacing: I24,
) -> Option<TicksBitMap> {
    if let Ok(bitmap) = pool.tickBitmap(word).call().await {
        let tks = bitmap_math::extract_ticks_from_bitmap(bitmap, word, tick_spacing);
        let mut new_ticks_map = BTreeMap::<I24, TickData>::new();
        let r: Option<TicksBitMap> = None;
        {
            let mut calls = FuturesOrdered::new();

            for tick in &tks {
                let pc = pool.clone();
                calls.push_back(async move { pc.ticks(*tick).call().await });
            }

            let mut idx = 0;
            while let Some(res) = calls.next().await {
                let tick = tks[idx];
                new_ticks_map.insert(
                    tick,
                    TickData {
                        liquidity_net: res.ok().map(|x| x.liquidityNet),
                    },
                );
            }
        };

        return Some(TicksBitMap {
            bitmap,
            ticks: new_ticks_map,
        });
    };
    None
}

async fn get_v4_word_ticks<P: Provider + Clone>(
    pool: &StateViewInstance<P>,
    word: i16,
    pool_id: B256,
    tick_spacing: I24,
) -> Option<TicksBitMap> {
    if let Ok(bitmap) = pool.getTickBitmap(pool_id, word).call().await {
        let tks = bitmap_math::extract_ticks_from_bitmap(bitmap, word, tick_spacing);
        let r: Option<TicksBitMap> = None;

        let calls = FuturesUnordered::new();

        for tick in tks {
            calls.push(pool.getTickInfo(pool_id, tick).call().await);
        }

        return r;
    };

    None
}

pub async fn get_v4_slot0<P: Provider + Clone>(
    pool: StateViewInstance<P>,
    id: B256,
) -> Option<getSlot0Return> {
    if let Ok(slot0) = pool.getSlot0(id).call().await {
        return Some(slot0);
    };
    None
}

pub async fn get_v4_config<P: Provider + Clone>(
    pool: StateViewInstance<P>,
    id: B256,
) -> Option<getSlot0Return> {
    if let Ok(slot0) = pool.getSlot0(id).call().await {
        return Some(slot0);
    };
    None
}

pub async fn get_v4_liquidity<P: Provider + Clone>(
    pool: StateViewInstance<P>,
    id: B256,
) -> Option<u128> {
    if let Ok(liq) = pool.getLiquidity(id).call().await {
        return Some(liq);
    };
    None
}

pub async fn get_v4_key<P: Provider + Clone>(
    id: B256,
    ps: IPositionManagerInstance<P>,
) -> Result<PoolKey, alloy::contract::Error> {
    let compacted_id: [u8; 25] = id[0..25]
        .try_into()
        .map_err(|_| alloy::contract::Error::NotADeploymentTransaction)?;

    ps.poolKeys(alloy::primitives::FixedBytes(compacted_id))
        .call()
        .await
}
