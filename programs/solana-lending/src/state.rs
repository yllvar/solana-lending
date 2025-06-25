use anchor_lang::prelude::*;
use anchor_lang::Space;

// Constants for account sizes
pub const MAX_LOANS_PER_USER: usize = 10;
pub const GLOBAL_STATE_SIZE: usize = 32 + 32 + 32 + 8 + 8 + 2 + 2 + 8 + 8;
pub const USER_STATE_SIZE: usize = 32 + 1 + 1 + 8 + 8 + 4 + (MAX_LOANS_PER_USER * 32);
pub const LOAN_SIZE: usize = 32 + 8 + 8 + 8 + 2 + 2 + 1;

#[account]
#[derive(Default)]
pub struct GlobalState {
    pub admin: Pubkey,
    pub treasury: Pubkey,
    pub oracle: Pubkey,          // Oracle pubkey for verification
    pub total_staked: u64,
    pub total_loans: u64,
    pub protocol_fee_rate: u16,  // basis points
    pub ltv_threshold: u16,      // 7000 = 70%
    pub min_stake_duration: i64, // 45 days in seconds
    pub oracle_fee: u64,         // Fee paid to oracle per verification
}

impl Space for GlobalState {
    const INIT_SPACE: usize = GLOBAL_STATE_SIZE;
}

#[account]
#[derive(Default)]
pub struct UserState {
    pub wallet: Pubkey,
    pub credit_score: u8,
    pub tier: u8,                // 0=Bronze, 1=Silver, 2=Gold
    pub staked_amount: u64,
    pub stake_start_time: i64,
    pub active_loans: Vec<Pubkey>,
}

impl Space for UserState {
    const INIT_SPACE: usize = USER_STATE_SIZE;
}

#[account]
#[derive(Default)]
pub struct Loan {
    pub borrower: Pubkey,
    pub amount: u64,
    pub collateral: u64,
    pub start_time: i64,
    pub interest_rate: u16,
    pub ltv_threshold: u16,
    pub status: LoanStatus,
}

impl Space for Loan {
    const INIT_SPACE: usize = LOAN_SIZE;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum LoanStatus {
    Active,
    Repaid,
    Liquidated,
    Defaulted,
}

impl Default for LoanStatus {
    fn default() -> Self {
        LoanStatus::Active
    }
}
