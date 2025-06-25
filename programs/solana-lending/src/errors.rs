use anchor_lang::prelude::*;

#[error_code]
pub enum LendingError {
    #[msg("Invalid credit score")]
    InvalidCreditScore,
    #[msg("Insufficient stake amount")]
    InsufficientStake,
    #[msg("Stake lockup period not met")]
    StakeLockupActive,
    #[msg("Loan amount exceeds collateral value")]
    LoanExceedsCollateral,
    #[msg("Loan not eligible for liquidation")]
    LoanNotLiquidatable,
    #[msg("Oracle signature verification failed")]
    InvalidOracleSignature,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Invalid protocol parameters")]
    InvalidProtocolParams,
}
