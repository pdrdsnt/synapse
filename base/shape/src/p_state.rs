use alloy::primitives::{U160, aliases::I24};

#[derive(Debug)]
pub enum AnyPoolState {
    V2(V2State),
    V3(V3State),
}

#[derive(Debug)]
pub struct V3State {
    pub tick: I24,
    pub x96price: U160,
    pub liquidity: u128,
}

#[derive(Debug)]
pub struct V2State {
    pub r0: u128,
    pub r1: u128,
}
