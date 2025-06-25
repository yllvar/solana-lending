use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, TokenAccount, Mint};
use crate::state::*;

#[derive(Accounts)]
pub struct RequestLoan<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
    
    #[account(
        mut,
        constraint = user_state.wallet == borrower.key() @ LendingError::Unauthorized,
        seeds = [b"user_state", borrower.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        init,
        payer = borrower,
        space = 8 + Loan::INIT_SPACE,
        seeds = [b"loan", borrower.key().as_ref(), &user_state.active_loans.len().to_le_bytes()],
        bump
    )]
    pub loan: Account<'info, Loan>,
    
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = borrower
    )]
    pub borrower_usdc_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = lending_pool
    )]
    pub lending_pool: Account<'info, TokenAccount>,
    
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
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<RequestLoan>,
    amount: u64,
    credit_score: u8,
    oracle_signature: [u8; 64],
) -> Result<()> {
    // Verify oracle signature using configured oracle
    let oracle_pubkey = ctx.accounts.global_state.oracle;
    let message = [
        ctx.accounts.borrower.key().as_ref(),
        &amount.to_le_bytes(),
        &credit_score.to_le_bytes(),
        &ctx.accounts.global_state.oracle_fee.to_le_bytes(),
    ]
    .concat();
    
    // Verify signature length
    require!(oracle_signature.len() == 64, LendingError::InvalidOracleSignature);
    
    // Verify signature
    let is_valid = anchor_lang::solana_program::ed25519_program::check(
        &oracle_pubkey,
        &message,
        &oracle_signature,
    ).map_err(|_| LendingError::InvalidOracleSignature)?;
    require!(is_valid, LendingError::InvalidOracleSignature);
    
    // Deduct oracle fee from borrower
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.borrower_usdc_account.to_account_info(),
                to: ctx.accounts.treasury.to_account_info(),
                authority: ctx.accounts.borrower.to_account_info(),
            },
        ),
        ctx.accounts.global_state.oracle_fee,
    )?;

    // Calculate max loan amount based on credit score and stake
    let max_loan = match credit_score {
        0..=50 => ctx.accounts.user_state.staked_amount / 2,
        51..=75 => ctx.accounts.user_state.staked_amount,
        76..=100 => ctx.accounts.user_state.staked_amount * 2,
        _ => return Err(LendingError::InvalidCreditScore.into()),
    };
    require!(amount <= max_loan, LendingError::LoanExceedsCollateral);

    // Transfer USDC from lending pool to borrower
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.lending_pool.to_account_info(),
                to: ctx.accounts.borrower_usdc_account.to_account_info(),
                authority: ctx.accounts.lending_pool.to_account_info(),
            },
        ),
        amount,
    )?;

    // Create loan account
    let loan = &mut ctx.accounts.loan;
    loan.borrower = ctx.accounts.borrower.key();
    loan.amount = amount;
    loan.collateral = ctx.accounts.user_state.staked_amount;
    loan.start_time = Clock::get()?.unix_timestamp;
    loan.interest_rate = match credit_score {
        0..=50 => 1500,  // 15%
        51..=75 => 1000, // 10%
        76..=100 => 500, // 5%
        _ => unreachable!(),
    };
    loan.ltv_threshold = ctx.accounts.global_state.ltv_threshold;
    loan.status = LoanStatus::Active;

    // Update user state
    ctx.accounts.user_state.active_loans.push(loan.key());
    ctx.accounts.user_state.credit_score = credit_score;
    
    // Update global stats
    ctx.accounts.global_state.total_loans += 1;
    
    msg!("Loan requested: {} USDC", amount);
    Ok(())
}
