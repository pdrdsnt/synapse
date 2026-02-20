use std::{collections::HashMap, str::FromStr};

use alloy::providers::Provider;
use alloy::{
    primitives::{Address, U256, aliases::U24},
    rlp::Encodable,
};
use dashmap::{DashMap, DashSet};
use shape::{
    d_any::{AnyDexShape, DexId, FullV2Dex, FullV3Dex, FullV4Dex, V2Fees, V3Fees},
    p_any::{AnyPoolShape, FullV2Pool, FullV3Pool, FullV4Pool},
    p_config::{V2Config, V3Config, V4Config},
    p_state::{V2State, V3State},
    t_any::{AnyTokenShape, ECR20Shape, TokenSymbol},
};

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

use crate::{
    generate_fallback_provider,
    //integration::{decompose_json_dexes, decompose_json_pools, decompose_json_tokens},
    types::{
        PartialV2Dex, PartialV2Pool, PartialV3Dex, PartialV3Pool, PartialV4Dex, PartialV4Pool,
    },
};

pub struct Cortex {
    pub tokens: Vec<AnyTokenShape>,

    pub v2_pools: Vec<PartialV2Pool>,
    pub v3_pools: Vec<PartialV3Pool>,
    pub v4_pools: Vec<PartialV4Pool>,

    pub v2_dexes: Vec<PartialV2Dex>,
    pub v3_dexes: Vec<PartialV3Dex>,
    pub v4_dexes: Vec<PartialV4Dex>,

    pub http_providers_urls: HashMap<u64, Vec<String>>,
    pub ws_providers_urls: HashMap<u64, Vec<String>>,
}

impl Cortex {
    /*
    pub fn from_json() -> Self {
        let input: ChainsJsonInput = chains::ChainsJsonInput::default();

        let mut tokens: Vec<AnyTokenShape> = Vec::new();
        let mut v2_pools = Vec::new();
        let mut v3_pools = Vec::new();
        let mut v4_pools = Vec::new();

        let mut v2_dexes = Vec::new();
        let mut v3_dexes = Vec::new();
        let mut v4_dexes = Vec::new();

        let mut tokens = Vec::new();

        let mut http_providers: HashMap<u64, Vec<String>> = HashMap::new();
        let mut ws_providers: HashMap<u64, Vec<String>> = HashMap::new();

        for (id, data) in input.chains {
            let mut pools = decompose_json_pools(id, &data.pools);
            v2_pools.append(&mut pools.v2_pools);
            v3_pools.append(&mut pools.v3_pools);
            v4_pools.append(&mut pools.v4_pools);

            let mut dexes = decompose_json_dexes(id, &data.dexes);
            v2_dexes.append(&mut dexes.v2_dexes);
            v3_dexes.append(&mut dexes.v3_dexes);
            v4_dexes.append(&mut dexes.v4_dexes);

            let mut tks = decompose_json_tokens(id, &data.tokens);
            tokens.append(&mut tks);
            http_providers.insert(id, data.http_nodes_urls.clone());

            for ws_provider in data.ws_nodes_urls {}
        }

        Self {
            tokens: tokens,
            v2_pools: v2_pools,
            v3_pools: v3_pools,
            v4_pools: v4_pools,
            v2_dexes: v2_dexes,
            v3_dexes: v3_dexes,
            v4_dexes: v4_dexes,
            http_providers_urls: http_providers,
            ws_providers_urls: ws_providers,
        }
    }
    */

    pub fn create_http_providers(&self) -> HashMap<u64, WsProvider> {
        let mut providers = HashMap::new();
        for (id, urls) in &self.http_providers_urls {
            if let Some(p) = generate_fallback_provider(urls.to_vec()) {
                providers.insert(*id, p);
            }
        }
        providers
    }
}
