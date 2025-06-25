use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, TokenAccount, Mint};
use crate::state::*;

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserState::INIT_SPACE,
        seeds = [b"user_state", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = staking_vault
    )]
    pub staking_vault: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = user
    )]
    pub user_usdc_account: Account<'info, TokenAccount>,
    
    pub usdc_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [b"global_state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Stake>, amount: u64) -> Result<()> {
    // Transfer USDC to staking vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_usdc_account.to_account_info(),
        to: ctx.accounts.staking_vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    token::transfer(
        CpiContext::new(cpi_program, cpi_accounts),
        amount,
    )?;

    // Update user state
    let user_state = &mut ctx.accounts.user_state;
    if user_state.staked_amount == 0 {
        user_state.wallet = ctx.accounts.user.key();
        user_state.stake_start_time = Clock::get()?.unix_timestamp;
    }
    user_state.staked_amount += amount;
    
    // Update global stats
    ctx.accounts.global_state.total_staked += amount;
    
    msg!("Staked {} USDC", amount);
    Ok(())
}
