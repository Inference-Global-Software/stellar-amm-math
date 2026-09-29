//! The contract's 11 methods as data, so that one driver can call any of them
//! on any module and compare the result with the oracle.

use crate::oracle::{self, Refusal};

/// A contract method, in the order `src/main.inf` declares them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    MinimumLiquidity,
    GetAmountOut,
    GetAmountIn,
    Quote,
    MintInitial,
    MintProportional,
    BurnShare,
    KHolds,
    KHoldsWithFee,
    MulDiv,
    MulDivUp,
}

/// The pin ([`crate::source::review_pin`]) of the refusal table in
/// `src/main.inf` and the notes under it ([`crate::source::refusal_table`]),
/// as they read when [`Method::refusals`] and [`Refusal`]'s rows were last
/// checked against them. `repo.rs` fails on any edit to the table until both
/// are checked again and the pin follows.
pub const REFUSAL_TABLE_PIN: &str = "a42f43901b029440";

/// What a method returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value {
    U32(u32),
    Bool(bool),
}

impl Method {
    pub const ALL: [Method; 11] = [
        Method::MinimumLiquidity,
        Method::GetAmountOut,
        Method::GetAmountIn,
        Method::Quote,
        Method::MintInitial,
        Method::MintProportional,
        Method::BurnShare,
        Method::KHolds,
        Method::KHoldsWithFee,
        Method::MulDiv,
        Method::MulDivUp,
    ];

    /// The exported name, which is also the Soroban function name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Method::MinimumLiquidity => "minimum_liquidity",
            Method::GetAmountOut => "get_amount_out",
            Method::GetAmountIn => "get_amount_in",
            Method::Quote => "quote",
            Method::MintInitial => "mint_initial",
            Method::MintProportional => "mint_proportional",
            Method::BurnShare => "burn_share",
            Method::KHolds => "k_holds",
            Method::KHoldsWithFee => "k_holds_with_fee",
            Method::MulDiv => "mul_div",
            Method::MulDivUp => "mul_div_up",
        }
    }

    /// The parameter names, which are the contract's CLI flags (`contract.rs`
    /// checks them against the interface `out/main.wasm` declares).
    #[must_use]
    pub fn params(self) -> &'static [&'static str] {
        match self {
            Method::MinimumLiquidity => &[],
            Method::GetAmountOut => &["amount_in", "reserve_in", "reserve_out", "fee_bps"],
            Method::GetAmountIn => &["amount_out", "reserve_in", "reserve_out", "fee_bps"],
            Method::Quote => &["amount_a", "reserve_a", "reserve_b"],
            Method::MintInitial => &["amount0", "amount1"],
            Method::MintProportional => {
                &["amount0", "amount1", "reserve0", "reserve1", "total_supply"]
            }
            Method::BurnShare => &["liquidity", "balance", "total_supply"],
            Method::KHolds => &["amount_in", "amount_out", "reserve_in", "reserve_out"],
            Method::KHoldsWithFee => {
                &["amount_in", "amount_out", "reserve_in", "reserve_out", "fee_bps"]
            }
            Method::MulDiv | Method::MulDivUp => &["a", "b", "denominator"],
        }
    }

    #[must_use]
    pub fn arity(self) -> usize {
        self.params().len()
    }

    /// The position of `fee_bps`, which takes its own edge values.
    #[must_use]
    pub fn fee_param(self) -> Option<usize> {
        self.params().iter().position(|&param| param == "fee_bps")
    }

    #[must_use]
    pub fn returns_bool(self) -> bool {
        matches!(self, Method::KHolds | Method::KHoldsWithFee)
    }

    /// The method's rows of the refusal table in `src/main.inf`, in the
    /// table's order, with the `fee_bps < 10000` check the note under the
    /// table adds to every method that takes a fee. [`REFUSAL_TABLE_PIN`]
    /// pins the table these were read from.
    #[must_use]
    pub fn refusals(self) -> &'static [Refusal] {
        use Refusal::{
            AmountOutNotBelowReserve, FeeNotBelowDenominator, LiquidityAboveSupply, NoShareFits, NothingBurned,
            NothingMinted, ResultDoesNotFit, RootBelowMinimum, ZeroAmountA, ZeroAmountIn, ZeroAmountOut,
            ZeroDenominator, ZeroReserve, ZeroSupply,
        };
        match self {
            Method::MinimumLiquidity => &[],
            Method::GetAmountOut => &[ZeroAmountIn, ZeroReserve, FeeNotBelowDenominator],
            Method::GetAmountIn => {
                &[ZeroAmountOut, ZeroReserve, AmountOutNotBelowReserve, ResultDoesNotFit, FeeNotBelowDenominator]
            }
            Method::Quote => &[ZeroAmountA, ZeroReserve, ResultDoesNotFit],
            Method::MintInitial => &[RootBelowMinimum, NothingMinted],
            Method::MintProportional => &[ZeroReserve, ZeroSupply, NoShareFits, NothingMinted],
            Method::BurnShare => &[ZeroSupply, LiquidityAboveSupply, NothingBurned],
            Method::KHolds => &[ZeroReserve, AmountOutNotBelowReserve],
            Method::KHoldsWithFee => &[ZeroReserve, AmountOutNotBelowReserve, FeeNotBelowDenominator],
            Method::MulDiv | Method::MulDivUp => &[ZeroDenominator, ResultDoesNotFit],
        }
    }

    /// The oracle's result for `args`, one `u32` per parameter.
    ///
    /// # Panics
    ///
    /// When `args` does not have one value per parameter.
    pub fn oracle(self, args: &[u32]) -> Result<Value, Refusal> {
        use Value::{Bool, U32};
        match (self, args) {
            (Method::MinimumLiquidity, []) => Ok(U32(oracle::minimum_liquidity())),
            (Method::GetAmountOut, &[a, ri, ro, f]) => oracle::get_amount_out(a, ri, ro, f).map(U32),
            (Method::GetAmountIn, &[x, ri, ro, f]) => oracle::get_amount_in(x, ri, ro, f).map(U32),
            (Method::Quote, &[a, ra, rb]) => oracle::quote(a, ra, rb).map(U32),
            (Method::MintInitial, &[a0, a1]) => oracle::mint_initial(a0, a1).map(U32),
            (Method::MintProportional, &[a0, a1, r0, r1, s]) => {
                oracle::mint_proportional(a0, a1, r0, r1, s).map(U32)
            }
            (Method::BurnShare, &[l, b, s]) => oracle::burn_share(l, b, s).map(U32),
            (Method::KHolds, &[a, o, ri, ro]) => oracle::k_holds(a, o, ri, ro).map(Bool),
            (Method::KHoldsWithFee, &[a, o, ri, ro, f]) => {
                oracle::k_holds_with_fee(a, o, ri, ro, f).map(Bool)
            }
            (Method::MulDiv, &[a, b, d]) => oracle::mul_div(a, b, d).map(U32),
            (Method::MulDivUp, &[a, b, d]) => oracle::mul_div_up(a, b, d).map(U32),
            _ => panic!("{} takes {} arguments, got {args:?}", self.name(), self.arity()),
        }
    }
}

/// A call to show in a failure message: `get_amount_out(1000, 100000, ...)`.
#[must_use]
pub fn render_call(method: Method, args: &[u32]) -> String {
    let args: Vec<String> = args.iter().map(u32::to_string).collect();
    format!("{}({})", method.name(), args.join(", "))
}
