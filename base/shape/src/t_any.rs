use alloy::primitives::Address;

pub enum AnyTokenShape {
    ECR20(u64, ECR20Shape),
}

pub struct TokenSymbol {
    pub raw: String,
    pub symbol: String,
}

pub struct IsStable(bool);

pub enum TokenKind {
    Native,
    Wrapped,
    Derivative,
    Stable,
    Lp,
    Unknown,
}

pub struct ECR20Shape {
    pub address: Address,
    pub symbol: TokenSymbol,
    pub kind: TokenKind,
}
