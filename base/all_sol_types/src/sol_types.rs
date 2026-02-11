use serde::{Deserialize, Serialize};

alloy_sol_types::sol! {
type IHooks is address;
type Currency is address;
type PoolId is bytes32;

/// @notice Returns the key for identifying a pool
///
#[derive(Serialize,Deserialize,Debug)]
struct PoolKey {
    Currency currency0;
    Currency currency1;
    uint24 fee;
    int24 tickSpacing;
    IHooks hooks;

}

// Event signature from Uniswap V4 PoolManager
    event PoolInitialized(
        address currency0,
        address currency1,
        uint24 fee,
        int24 tickSpacing,
        address hooks,
        bytes32 poolId
    );



#[derive(Serialize,Deserialize,Debug)]
#[sol(rpc)]
interface IUniswapV2Factory {
        function getPair(address tokenA, address tokenB) external view returns (address pair);
        function createPair(address tokenA, address tokenB) external returns (address pair);
        function allPairs(uint) external view returns (address pair);
        function allPairsLength() external view returns (uint);

        event PairCreated(address indexed token0, address indexed token1, address pair, uint);
    }


#[derive(Serialize,Deserialize,Debug)]
#[sol(rpc)]
interface IUniswapV3Factory {
    event OwnerChanged(address indexed oldOwner, address indexed newOwner);
    event PoolCreated(
        address indexed token0,
        address indexed token1,
        uint24 indexed fee,
        int24 tickSpacing,
        address pool
    );

    event FeeAmountEnabled(uint24 indexed fee, int24 indexed tickSpacing);
    function owner() external view returns (address);
    function feeAmountTickSpacing(uint24 fee) external view returns (int24);

    function getPool(
        address tokenA,
        address tokenB,
        uint24 fee
    ) external view returns (address pool);

    function createPool(
        address tokenA,
        address tokenB,
        uint24 fee
    ) external returns (address pool);

    function setOwner(address _owner) external;

    function enableFeeAmount(uint24 fee, int24 tickSpacing) external;
}


#[derive(Serialize,Deserialize,Debug)]
#[sol(rpc)]
contract StateView{
    function getSlot0(PoolId poolId)
        external
        view
        returns (uint160 sqrtPriceX96, int24 tick, uint24 protocolFee, uint24 lpFee);

    function getTickInfo(PoolId poolId, int24 tick)
        external
        view
        returns (
            uint128 liquidityGross,
            int128 liquidityNet,
            uint256 feeGrowthOutside0X128,
            uint256 feeGrowthOutside1X128
        );

    function getTickLiquidity(PoolId poolId, int24 tick)
        external
        view
        returns (uint128 liquidityGross, int128 liquidityNet);
    function getLiquidity(PoolId poolId) external view returns (uint128 liquidity);
    function getTickBitmap(PoolId poolId, int16 tick) external view returns (uint256 tickBitmap);


    event ModifyLiquidity(
        PoolId indexed id, address indexed sender, int24 tickLower, int24 tickUpper, int256 liquidityDelta, bytes32 salt
    );

    event Initialize(
        PoolId indexed id,
        Currency indexed currency0,
        Currency indexed currency1,
        uint24 fee,
        int24 tickSpacing,
        IHooks hooks,
        uint160 sqrtPriceX96,
        int24 tick
    );

    event Swap(
        PoolId indexed id,
        address indexed sender,
        int128 amount0,
        int128 amount1,
        uint160 sqrtPriceX96,
        uint128 liquidity,
        int24 tick,
        uint24 fee
    );

    event Donate(PoolId indexed id, address indexed sender, uint256 amount0, uint256 amount1);
    function getPositionInfo(PoolId poolId, address owner, int24 tickLower, int24 tickUpper, bytes32 salt)
        external
        view
            returns (uint128 liquidity, uint256 feeGrowthInside0LastX128, uint256 feeGrowthInside1LastX128);
    function getPositionInfo(PoolId poolId, bytes32 positionId)
        external
        view
        returns (uint128 liquidity, uint256 feeGrowthInside0LastX128, uint256 feeGrowthInside1LastX128);    /// @inheritdoc IStateView
    function getPositionLiquidity(PoolId poolId, bytes32 positionId) external view returns (uint128 liquidity);
    function getFeeGrowthInside(PoolId poolId, int24 tickLower, int24 tickUpper)
        external
        view
        returns (uint256 feeGrowthInside0X128, uint256 feeGrowthInside1X128);
}

type BalanceDelta is int256;

using {add as +, sub as -, eq as ==, neq as !=} for BalanceDelta global;
using BalanceDeltaLibrary for BalanceDelta global;
using SafeCast for int256;

function toBalanceDelta(int128 _amount0, int128 _amount1) pure returns (BalanceDelta balanceDelta) {
    assembly ("memory-safe") {
        balanceDelta := or(shl(128, _amount0), and(sub(shl(128, 1), 1), _amount1))
    }
}

function add(BalanceDelta a, BalanceDelta b) pure returns (BalanceDelta) {
    int256 res0;
    int256 res1;
    assembly ("memory-safe") {
        let a0 := sar(128, a)
        let a1 := signextend(15, a)
        let b0 := sar(128, b)
        let b1 := signextend(15, b)
        res0 := add(a0, b0)
        res1 := add(a1, b1)
    }
    return toBalanceDelta(res0.toInt128(), res1.toInt128());
}

function sub(BalanceDelta a, BalanceDelta b) pure returns (BalanceDelta) {
    int256 res0;
    int256 res1;
    assembly ("memory-safe") {
        let a0 := sar(128, a)
        let a1 := signextend(15, a)
        let b0 := sar(128, b)
        let b1 := signextend(15, b)
        res0 := sub(a0, b0)
        res1 := sub(a1, b1)
    }
    return toBalanceDelta(res0.toInt128(), res1.toInt128());
}

function eq(BalanceDelta a, BalanceDelta b) pure returns (bool) {
    return BalanceDelta.unwrap(a) == BalanceDelta.unwrap(b);
}

function neq(BalanceDelta a, BalanceDelta b) pure returns (bool) {
    return BalanceDelta.unwrap(a) != BalanceDelta.unwrap(b);
}

/// @notice Library for getting the amount0 and amount1 deltas from the BalanceDelta type
library BalanceDeltaLibrary {
    /// @notice A BalanceDelta of 0
    BalanceDelta public constant ZERO_DELTA = BalanceDelta.wrap(0);

    function amount0(BalanceDelta balanceDelta) internal pure returns (int128 _amount0) {
        assembly ("memory-safe") {
            _amount0 := sar(128, balanceDelta)
        }
    }

    function amount1(BalanceDelta balanceDelta) internal pure returns (int128 _amount1) {
        assembly ("memory-safe") {
            _amount1 := signextend(15, balanceDelta)
        }
    }
}



#[derive(Serialize,Deserialize,Debug)]
interface IPoolManager is IProtocolFees, IERC6909Claims, IExtsload, IExttload {
    error CurrencyNotSettled();

    error PoolNotInitialized();
    error AlreadyUnlocked();

    error ManagerLocked();

    error TickSpacingTooLarge(int24 tickSpacing);

    error TickSpacingTooSmall(int24 tickSpacing);

    error CurrenciesOutOfOrderOrEqual(address currency0, address currency1);

    error UnauthorizedDynamicLPFeeUpdate();

    error SwapAmountCannotBeZero();

    error NonzeroNativeValue();

    error MustClearExactPositiveDelta();

    event Initialize(
        PoolId indexed id,
        Currency indexed currency0,
        Currency indexed currency1,
        uint24 fee,
        int24 tickSpacing,
        IHooks hooks,
        uint160 sqrtPriceX96,
        int24 tick
    );

    event ModifyLiquidity(
        PoolId indexed id, address indexed sender, int24 tickLower, int24 tickUpper, int256 liquidityDelta, bytes32 salt
    );

    event Swap(
        PoolId indexed id,
        address indexed sender,
        int128 amount0,
        int128 amount1,
        uint160 sqrtPriceX96,
        uint128 liquidity,
        int24 tick,
        uint24 fee
    );

    event Donate(PoolId indexed id, address indexed sender, uint256 amount0, uint256 amount1);

    function unlock(bytes calldata data) external returns (bytes memory);

    function initialize(PoolKey memory key, uint160 sqrtPriceX96) external returns (int24 tick);

    struct ModifyLiquidityParams {
        // the lower and upper tick of the position
        int24 tickLower;
        int24 tickUpper;
        // how to modify the liquidity
        int256 liquidityDelta;
        // a value to set if you want unique liquidity positions at the same range
        bytes32 salt;
    }

    function modifyLiquidity(PoolKey memory key, ModifyLiquidityParams memory params, bytes calldata hookData)
        external
        returns (BalanceDelta callerDelta, BalanceDelta feesAccrued);

    struct SwapParams {
        /// Whether to swap token0 for token1 or vice versa
        bool zeroForOne;
        /// The desired input amount if negative (exactIn), or the desired output amount if positive (exactOut)
        int256 amountSpecified;
        /// The sqrt price at which, if reached, the swap will stop executing
        uint160 sqrtPriceLimitX96;
    }

    function swap(PoolKey memory key, SwapParams memory params, bytes calldata hookData)
        external
        returns (BalanceDelta swapDelta);

    function donate(PoolKey memory key, uint256 amount0, uint256 amount1, bytes calldata hookData)
        external
        returns (BalanceDelta);

    function sync(Currency currency) external;

    function take(Currency currency, address to, uint256 amount) external;

    function settle() external payable returns (uint256 paid);
    function settleFor(address recipient) external payable returns (uint256 paid);
    function clear(Currency currency, uint256 amount) external;

    function mint(address to, uint256 id, uint256 amount) external;

    function burn(address from, uint256 id, uint256 amount) external;

    function updateDynamicLPFee(PoolKey memory key, uint24 newDynamicLPFee) external;
}


#[derive(Debug, Serialize, Deserialize)]
#[sol(rpc)]
interface IUniswapV2Pair {
        function name() external view returns (string);
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);
        function totalSupply() external view returns (uint256);
        function balanceOf(address owner) external view returns (uint256);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 value) external returns (bool);
        function transfer(address to, uint256 value) external returns (bool);
        function transferFrom(address from, address to, uint256 value) external returns (bool);

        function factory() external view returns (address);
        function token0() external view returns (address);
        function token1() external view returns (address);
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
        function mint(address to) external returns (uint liquidity);
        function burn(address to) external returns (uint amount0, uint amount1);
        function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external;
        function skim(address to) external;
        function sync() external;

        event Approval(address indexed owner, address indexed spender, uint value);
        event Transfer(address indexed from, address indexed to, uint value);
        event Mint(address indexed sender, uint amount0, uint amount1);
        event Burn(address indexed sender, uint amount0, uint amount1, address indexed to);
        event Swap(
            address indexed sender,
            uint amount0In,
            uint amount1In,
            uint amount0Out,
            uint amount1Out,
            address indexed to
        );
        event Sync(uint112 reserve0, uint112 reserve1);
    }

#[derive(Debug, Serialize, Deserialize)]
#[sol(rpc)]
contract V3Pool {
   function slot0()
        external
        view
        returns (
            uint160 sqrtPriceX96,
            int24 tick,
            uint16 observationIndex,
            uint16 observationCardinality,
            uint16 observationCardinalityNext,
            uint8 feeProtocol,
            bool unlocked
        );

   function ticks(int24 tick)
        external
        view
        returns (
            uint128 liquidityGross,
            int128 liquidityNet,
            uint256 feeGrowthOutside0X128,
            uint256 feeGrowthOutside1X128,
            int56 tickCumulativeOutside,
            uint160 secondsPerLiquidityOutsideX128,
            uint32 secondsOutside,
            bool initialized
        );

    function liquidity() external view returns (uint128);
    function tickBitmap(int16 wordPosition) external view returns (uint256);
    function factory() external view returns (address);
    function token0() external view returns (address);
    function token1() external view returns (address);
    function fee() external view returns (uint24);
    function tickSpacing() external view returns (int24);
    function maxLiquidityPerTick() external view returns (uint128);

    event Swap(
        address indexed sender,
        address indexed recipient,
        int256 amount0,
        int256 amount1,
        uint160 sqrtPriceX96,
        uint128 liquidity,
        int24 tick
    );

    event Flash(
        address indexed sender,
        address indexed recipient,
        uint256 amount0,
        uint256 amount1,
        uint256 paid0,
        uint256 paid1
    );

    event Mint(
        address sender,
        address indexed owner,
        int24 indexed tickLower,
        int24 indexed tickUpper,
        uint128 amount,
        uint256 amount0,
        uint256 amount1
    );

    event Collect(
        address indexed owner,
        address recipient,
        int24 indexed tickLower,
        int24 indexed tickUpper,
        uint128 amount0,
        uint128 amount1
    );

    event Burn(
        address indexed owner,
        int24 indexed tickLower,
        int24 indexed tickUpper,
        uint128 amount,
        uint256 amount0,
        uint256 amount1
    );
}


#[derive(Debug,Serialize, Deserialize)]
#[sol(rpc)]
interface IERC20 {
        function name() external view returns (string);
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);
        function totalSupply() external view returns (uint256);
        function balanceOf(address account) external view returns (uint256);
        function transfer(address to, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address from, address to, uint256 amount) external returns (bool);

        // Optional: ERC20 events
        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);

}

#[derive(Debug,Serialize, Deserialize)]
#[sol(rpc)]
interface IERC721 {
        function balanceOf(address owner) external view returns (uint256);
        function ownerOf(uint256 tokenId) external view returns (address);
        function safeTransferFrom(address from, address to, uint256 tokenId) external;
        function transferFrom(address from, address to, uint256 tokenId) external;
        function approve(address to, uint256 tokenId) external;
        function getApproved(uint256 tokenId) external view returns (address);
        function setApprovalForAll(address operator, bool approved) external;
        function isApprovedForAll(address owner, address operator) external view returns (bool);

        event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
        event Approval(address indexed owner, address indexed approved, uint256 indexed tokenId);
        event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
    }


#[derive(Debug,Serialize,Deserialize)]
#[sol(rpc)]
interface IERC165 {

    function supportsInterface(bytes4 interfaceId) public view virtual returns (bool) {
        return interfaceId == type(IERC165).interfaceId;
    }
}
interface ICurveMetaRegistry {
        // Registry Discovery
        function registry_length() external view returns (uint256);
        function pool_list(uint256 index) external view returns (address);
        function get_pool_name(address pool) external view returns (string);
        function is_meta(address pool) external view returns (bool);

        // Pool Introspection via RegistryHandler APIs
        function get_n_coins(address pool) external view returns (uint256);
        function get_coins(address pool) external view returns (address[8]);
        function get_underlying_coins(address pool) external view returns (address[8]);
        function get_balances(address pool) external view returns (uint256[8]);
        function get_underlying_balances(address pool) external view returns (uint256[8]);
        function get_coin_indices(address pool, address from, address to) external view returns (int128, int128, bool);
    }
    // StableSwap v1 (plain pool)
    interface ICurveV1PlainPool {
        // Core quoting / swapping
        function get_dy(int128 i, int128 j, uint256 dx) external view returns (uint256);
        function exchange(int128 i, int128 j, uint256 dx, uint256 min_dy) external;

        // Pool state
        function A() external view returns (uint256);                // amplification
        function fee() external view returns (uint256);             // swap fee (1e10 or 1e8 style, pool-dependent)
        function get_virtual_price() external view returns (uint256);

        // Balances & coins
        function balances(uint256 index) external view returns (uint256);
        function coins(uint256 index) external view returns (address);

        // Optional but common
        function admin_fee() external view returns (uint256);
        // Some pools expose N_COINS as an immut/const; not always callable.
    }
    interface ICurveV1Underlying {
        // Underlying coins (e.g., DAI/USDC/USDT beneath cTokens or meta setup)
        function underlying_coins(uint256 index) external view returns (address);

        // Underlying quoting / swapping
        function get_dy_underlying(int128 i, int128 j, uint256 dx) external view returns (uint256);
        function exchange_underlying(int128 i, int128 j, uint256 dx, uint256 min_dy) external;

        // Optional convenience for meta pools
        function base_pool() external view returns (address);
    }
interface ICurveV2CryptoPool {
        // Core quoting / swapping
        function get_dy(int128 i, int128 j, uint256 dx) external view returns (uint256);
        function exchange(int128 i, int128 j, uint256 dx, uint256 min_dy) external;

        // Coins & balances
        function coins(uint256 index) external view returns (address);
        function balances(uint256 index) external view returns (uint256);

        // Dynamic fee & parameters
        function fee() external view returns (uint256);        // current effective fee (computed)
        function gamma() external view returns (uint256);
        function mid_fee() external view returns (uint256);
        function out_fee() external view returns (uint256);
        function fee_gamma() external view returns (uint256);

        // Pricing / oracles
        function get_virtual_price() external view returns (uint256);
        function price_scale(uint256 index) external view returns (uint256);
        function price_oracle(uint256 index) external view returns (uint256);
        function last_prices(uint256 index) external view returns (uint256);
    }
 interface ICurveFactory {
        function pool_count() external view returns (uint256);
        function pool_list(uint256 index) external view returns (address);
    }
interface IERC1155 is IERC165 {
    /**
     * @dev Emitted when `value` amount of tokens of type `id` are transferred from `from` to `to` by `operator`.
     */
    event TransferSingle(address indexed operator, address indexed from, address indexed to, uint256 id, uint256 value);

    /**
     * @dev Equivalent to multiple {TransferSingle} events, where `operator`, `from` and `to` are the same for all
     * transfers.
     */
    event TransferBatch(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256[] ids,
        uint256[] values
    );

    /**
     * @dev Emitted when `account` grants or revokes permission to `operator` to transfer their tokens, according to
     * `approved`.
     */
    event ApprovalForAll(address indexed account, address indexed operator, bool approved);

    /**
     * @dev Emitted when the URI for token type `id` changes to `value`, if it is a non-programmatic URI.
     *
     * If an {URI} event was emitted for `id`, the standard
     * https://eips.ethereum.org/EIPS/eip-1155#metadata-extensions[guarantees] that `value` will equal the value
     * returned by {IERC1155MetadataURI-uri}.
     */
    event URI(string value, uint256 indexed id);

    /**
     * @dev Returns the value of tokens of token type `id` owned by `account`.
     */
    function balanceOf(address account, uint256 id) external view returns (uint256);

    /**
     * @dev xref:ROOT:erc1155.adoc#batch-operations[Batched] version of {balanceOf}.
     *
     * Requirements:
     *
     * - `accounts` and `ids` must have the same length.
     */
    function balanceOfBatch(
        address[] calldata accounts,
        uint256[] calldata ids
    ) external view returns (uint256[] memory);

    /**
     * @dev Grants or revokes permission to `operator` to transfer the caller's tokens, according to `approved`,
     *
     * Emits an {ApprovalForAll} event.
     *
     * Requirements:
     *
     * - `operator` cannot be the zero address.
     */
    function setApprovalForAll(address operator, bool approved) external;

    /**
     * @dev Returns true if `operator` is approved to transfer ``account``'s tokens.
     *
     * See {setApprovalForAll}.
     */
    function isApprovedForAll(address account, address operator) external view returns (bool);

    /**
     * @dev Transfers a `value` amount of tokens of type `id` from `from` to `to`.
     *
     * WARNING: This function can potentially allow a reentrancy attack when transferring tokens
     * to an untrusted contract, when invoking {IERC1155Receiver-onERC1155Received} on the receiver.
     * Ensure to follow the checks-effects-interactions pattern and consider employing
     * reentrancy guards when interacting with untrusted contracts.
     *
     * Emits a {TransferSingle} event.
     *
     * Requirements:
     *
     * - `to` cannot be the zero address.
     * - If the caller is not `from`, it must have been approved to spend ``from``'s tokens via {setApprovalForAll}.
     * - `from` must have a balance of tokens of type `id` of at least `value` amount.
     * - If `to` refers to a smart contract, it must implement {IERC1155Receiver-onERC1155Received} and return the
     * acceptance magic value.
     */
    function safeTransferFrom(address from, address to, uint256 id, uint256 value, bytes calldata data) external;

    /**
     * @dev xref:ROOT:erc1155.adoc#batch-operations[Batched] version of {safeTransferFrom}.
     *
     * WARNING: This function can potentially allow a reentrancy attack when transferring tokens
     * to an untrusted contract, when invoking {IERC1155Receiver-onERC1155BatchReceived} on the receiver.
     * Ensure to follow the checks-effects-interactions pattern and consider employing
     * reentrancy guards when interacting with untrusted contracts.
     *
     * Emits either a {TransferSingle} or a {TransferBatch} event, depending on the length of the array arguments.
     *
     * Requirements:
     *
     * - `ids` and `values` must have the same length.
     * - If `to` refers to a smart contract, it must implement {IERC1155Receiver-onERC1155BatchReceived} and return the
     * acceptance magic value.
     */
    function safeBatchTransferFrom(
        address from,
        address to,
        uint256[] calldata ids,
        uint256[] calldata values,
        bytes calldata data
    ) external;
}
}
