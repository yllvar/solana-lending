use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, TokenAccount};
use crate::state::*;

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"user_state", wallet.key().as_ref()],
        bump,
        has_one = wallet @ LendingError::Unauthorized
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"loan", wallet.key().as_ref(), &loan_index.to_le_bytes()],
        bump,
        constraint = loan.status == LoanStatus::Active @ LendingError::LoanNotLiquidatable
    )]
    pub loan: Account<'info, Loan>,
    
    #[account(mut)]
    pub wallet: AccountInfo<'info>,
    
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = liquidator
    )]
    pub liquidator_usdc_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = staking_vault
    )]
    pub staking_vault: Account<'info, TokenAccount>,
    
    pub usdc_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [b"global_state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,
    
    #[account(mut)]
    pub treasury: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<Liquidate>, loan_index: u64) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    
    // Add loan_index to loan account seeds
    let loan_account_info = loan.to_account_info();
    let loan_seeds = &[
        b"loan",
        ctx.accounts.wallet.key().as_ref(),
        &loan_index.to_le_bytes(),
        &[ctx.bumps.loan]
    ];
    let signer_seeds = &[&loan_seeds[..]];
    
    // Check if loan is eligible for liquidation
    let current_time = Clock::get()?.unix_timestamp;
    let loan_age = current_time - loan.start_time;
    let is_overdue = loan_age > 90 * 24 * 60 * 60; // 90 days
    
    // Simplified LTV check (in a real implementation, you'd use price oracle)
    let collateral_value = loan.collateral;
    let loan_value = loan.amount + (loan.amount * loan.interest_rate as u64 / 10000);
    let current_ltv = (loan_value * 10000) / collateral_value;
    
    require!(
        current_ltv > loan.ltv_threshold as u64 || is_overdue,
        LendingError::LoanNotLiquidatable
    );

    // Calculate liquidation amount (simplified: 50% of collateral)
    let liquidation_amount = loan.collateral / 2;
    
    // Transfer collateral from staking vault to liquidator
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.staking_vault.to_account_info(),
                to: ctx.accounts.liquidator_usdc_account.to_account_info(),
                authority: ctx.accounts.staking_vault.to_account_info(),
            },
        ),
        liquidation_amount,
    )?;

    // Update loan status
    loan.status = LoanStatus::Liquidated;
    
    // Update user state
    if let Some(pos) = ctx.accounts.user_state.active_loans.iter().position(|&x| x == loan.key()) {
        ctx.accounts.user_state.active_loans.remove(pos);
    }
    
    // Update global stats
    ctx.accounts.global_state.total_staked -= liquidation_amount;
    
    msg!("Loan liquidated: {} USDC transferred to liquidator", liquidation_amount);
    Ok(())
}
