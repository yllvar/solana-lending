use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + GlobalState::INIT_SPACE,
        seeds = [b"global_state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Initialize>,
    protocol_fee_rate: u16,
    ltv_threshold: u16,
    min_stake_duration: i64,
    oracle: Pubkey,
    oracle_fee: u64
) -> Result<()> {
    // Validate protocol parameters
    require!(protocol_fee_rate < 1000, LendingError::InvalidProtocolParams);
    require!(ltv_threshold > 0 && ltv_threshold < 10000, LendingError::InvalidProtocolParams);
    require!(min_stake_duration > 0, LendingError::InvalidProtocolParams);
    require!(oracle_fee > 0, LendingError::InvalidProtocolParams);

    let global_state = &mut ctx.accounts.global_state;
    global_state.admin = ctx.accounts.admin.key();
    global_state.treasury = ctx.accounts.admin.key(); // Default treasury to admin
    global_state.oracle = oracle;
    global_state.protocol_fee_rate = protocol_fee_rate;
    global_state.ltv_threshold = ltv_threshold;
    global_state.min_stake_duration = min_stake_duration;
    global_state.oracle_fee = oracle_fee;
    global_state.total_staked = 0;
    global_state.total_loans = 0;

    msg!("Protocol initialized with fee rate: {} bps", protocol_fee_rate);
    Ok(())
}
