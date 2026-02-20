use std::str::FromStr;

use alloy::{
    primitives::{
        Address,
        aliases::{I24, U24},
    },
    transports::http::Http,
};
/*
use chains_json::{
    chain_json_model::{DexJsonModel, PoolJsonModel, TokenJsonModel},
    chains::ChainsJsonInput,
};
*/

use shape::{
    d_any::{AnyDexShape, DexId, FullV2Dex, FullV3Dex, FullV4Dex, V2Fees, V3Fees},
    p_any::{AnyPoolShape, FullV2Pool, FullV3Pool, FullV4Pool},
    p_config::{V2Config, V3Config, V4Config},
    p_state::{V2State, V3State},
    t_any::{AnyTokenShape, ECR20Shape, TokenSymbol},
};
use url::Url;

use crate::types::{
    PartialV2Dex, PartialV2Pool, PartialV3Dex, PartialV3Pool, PartialV4Dex, PartialV4Pool,
};

/*
pub fn decompose_json_input(data: &ChainsJsonInput) {
    let mut tokens: Vec<AnyTokenShape> = Vec::new();
    let mut v2_pools: Vec<PartialV2Pool> = Vec::new();
    let mut v3_pools: Vec<PartialV3Pool> = Vec::new();
    let mut v4_pools: Vec<PartialV4Pool> = Vec::new();

    let mut v2_dexes: Vec<PartialV2Dex> = Vec::new();
    let mut v3_dexes: Vec<PartialV3Dex> = Vec::new();
    let mut v4_dexes: Vec<PartialV4Dex> = Vec::new();

    for (id, data) in data.chains.iter() {
        data.tokens.iter().for_each(|t| {
            if let Ok(addr) = Address::from_str(&t.address) {
                tokens.push(AnyTokenShape::ECR20(
                    *id,
                    ECR20Shape {
                        address: addr,
                        symbol: TokenSymbol {
                            raw: t.symbol.clone(),
                            symbol: "".to_string(),
                        },
                        kind: shape::t_any::TokenKind::Unknown,
                    },
                ))
            }
        });
    }
}

pub fn decompose_json_pools(id: u64, pools: &Vec<PoolJsonModel>) -> DecomposePoolsReturn {
    let mut v2_pools = Vec::<PartialV2Pool>::new();
    let mut v3_pools = Vec::<PartialV3Pool>::new();
    let mut v4_pools = Vec::<PartialV4Pool>::new();

    pools.iter().for_each(|p| match p {
        chains_json::chain_json_model::PoolJsonModel::V2 {
            address,
            token0,
            token1,
            fee,
        } => {
            if let (Ok(addr), fe, Ok(t0), Ok(t1)) = (
                Address::from_str(address),
                U24::from(*fee),
                Address::from_str(token0),
                Address::from_str(token1),
            ) {
                let new_v2 = PartialV2Pool {
                    chain: id,
                    address: addr,
                    config: Some(V2Config {
                        name: "".to_string(),
                        fee: fe,
                        token0: t0,
                        token1: t1,
                    }),
                    state: None,
                };
                v2_pools.push(new_v2);
            }
        }

        chains_json::chain_json_model::PoolJsonModel::V3 {
            address,
            token0,
            token1,
            fee,
        } => {
            if let (Ok(addr), fe, Ok(t0), Ok(t1)) = (
                Address::from_str(address),
                U24::from(*fee),
                Address::from_str(token0),
                Address::from_str(token1),
            ) {
                let mut tick_spacing = I24::from_be_bytes(1_i8.to_be_bytes());
                if *fee != 100 {
                    tick_spacing = (I24::from(fe) / I24::from_be_bytes(100_i8.to_be_bytes()))
                        * I24::from_be_bytes(2_i8.to_be_bytes());
                };

                let new_v3 = PartialV3Pool {
                    chain: id,
                    address: addr,
                    config: Some(V3Config {
                        name: "".to_string(),
                        fee: fe,
                        token0: t0,
                        token1: t1,
                        tick_spacing,
                    }),
                    state: None,
                };
                v3_pools.push(new_v3);
            }
        }
        chains_json::chain_json_model::PoolJsonModel::V4 {
            pool_manager,
            state_view,
            token0,
            token1,
            fee,
            spacing,
            hooks,
        } => {
            let mut hook = Address::ZERO;
            if let Some(hks) = hooks {
                if let Ok(h) = Address::from_str(&hks) {
                    hook = h;
                };
            };

            if let (Ok(sv), Ok(t0), Ok(t1), Ok(ts)) = (
                Address::from_str(&state_view),
                Address::from_str(token0),
                Address::from_str(token1),
                I24::try_from(*spacing),
            ) {
                let new_v4 = PartialV4Pool {
                    chain: id,
                    // In V4, the entry point address is the PoolManager
                    address: sv,
                    config: Some(V4Config {
                        fee: U24::from(*fee),
                        token0: t0,
                        token1: t1,
                        // V4 has explicit spacing, unlike early V3
                        tick_spacing: ts,
                        hooks: hook,
                    }),
                    state: None,
                };
                v4_pools.push(new_v4);
            }
        }
    });

    DecomposePoolsReturn {
        v2_pools,
        v3_pools,
        v4_pools,
    }
}

pub fn decompose_json_tokens(id: u64, tokens: &Vec<TokenJsonModel>) -> Vec<AnyTokenShape> {
    let mut parsed_tokens = Vec::new();

    tokens.iter().for_each(|t| {
        if let Ok(addr) = Address::from_str(&t.address) {
            parsed_tokens.push(AnyTokenShape::ECR20(
                id,
                ECR20Shape {
                    address: addr,
                    symbol: TokenSymbol {
                        raw: t.symbol.clone(),
                        symbol: "".to_string(), // You might want to populate this if available
                    },
                    kind: shape::t_any::TokenKind::Unknown,
                },
            ))
        }
    });
    parsed_tokens
}

pub fn decompose_json_dexes(id: u64, dexes: &Vec<DexJsonModel>) -> DecomposeDexesReturn {
    let mut v2_dexes = Vec::<PartialV2Dex>::new();
    let mut v3_dexes = Vec::<PartialV3Dex>::new();
    let mut v4_dexes = Vec::<PartialV4Dex>::new();

    dexes.iter().for_each(|p| match p {
        DexJsonModel::V2 {
            address,
            fee,        // unused
            stable_fee, // unused
        } => {
            if let Ok(addr) = Address::from_str(address) {
                let sf = match stable_fee {
                    Some(f) => {
                        let c = U24::from(*f);

                        Some(c)
                    }
                    None => None,
                };

                v2_dexes.push(PartialV2Dex {
                    chain: id,
                    address: addr,
                    id: None,
                    fees: Some(V2Fees {
                        crypto: U24::from(*fee),
                        stable: sf,
                    }),
                })
            }
        }

        DexJsonModel::V3 { address, fee: _ } => {
            if let Ok(addr) = Address::from_str(address) {
                v3_dexes.push(PartialV3Dex {
                    chain: id,
                    address: addr,
                    id: None,
                    fees: None,
                })
            }
        }

        DexJsonModel::V4 {
            state_view,
            pool_manager: _, // unused
        } => {
            if let Ok(addr) = Address::from_str(state_view) {
                v4_dexes.push(PartialV4Dex {
                    chain: id,
                    state_view: addr,
                    id: None,
                })
            }
        }
    });

    DecomposeDexesReturn {
        v2_dexes,
        v3_dexes,
        v4_dexes,
    }
}
*/

pub fn dex_id_from_string(string: String) -> DexId {
    match string.as_str() {
        "uniswap" => DexId::Uniswap,
        "sushiswap" => DexId::Sushiswap,
        "pancake" => DexId::Pancake,
        "curve" => DexId::Curve,
        "balancer" => DexId::Balancer,
        _ => DexId::Unknown,
    }
}

pub struct DecomposePoolsReturn {
    pub v2_pools: Vec<PartialV2Pool>,
    pub v3_pools: Vec<PartialV3Pool>,
    pub v4_pools: Vec<PartialV4Pool>,
}

pub struct DecomposeDexesReturn {
    pub v2_dexes: Vec<PartialV2Dex>,
    pub v3_dexes: Vec<PartialV3Dex>,
    pub v4_dexes: Vec<PartialV4Dex>,
}

pub fn contact(ws_provider: Http<Url>) {}
