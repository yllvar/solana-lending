use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;

declare_id!("DsReGnD5oqLFcYbzWZ36Esnb1mJybEYUiKHXhAXRmbDB");

#[program]
pub mod lending_program {
    use super::*;

    pub fn initialize(
        ctx: Context<instructions::initialize::Initialize>,
        protocol_fee_rate: u16,
        ltv_threshold: u16,
        min_stake_duration: i64,
        oracle: Pubkey,
        oracle_fee: u64
    ) -> Result<()> {
        instructions::initialize::handler(ctx, protocol_fee_rate, ltv_threshold, min_stake_duration, oracle, oracle_fee)
    }
}
