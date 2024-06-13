use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

declare_id!("EyUB9d1nWYR3tUZg7cNmfzN2q6wEXSk917ip1NfPtiGS");
pub mod constants {
    pub const VAULT_SEED: &[u8] = b"vault";
    pub const TOKEN_SEED: &[u8] = b"token";
    pub const STAKE_INFO_SEED: &[u8] = b"stake_info";
}

#[program]
pub mod checking {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        // msg!(ctx);
        Ok(())
    }
    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        Ok(())
    }
    // pub fn unstake(ctx: Context) -> Result<()> {
    //     Ok(())
    // }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        seeds=[constants::VAULT_SEED],
        bump,
        payer=signer,
        token::mint=mint_account,
        token::authority=token_vault_account,//vault shoul dhave an authority of its own to sign ,hence
    )]
    pub token_vault_account: Account<'info, TokenAccount>,
    pub mint_account: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        seeds=[constants::STAKE_INFO_SEED,signer.key.as_ref()],
        bump,
        payer=signer,
        token::mint=mint_account,
        token::authority=stake_info_account
    )]
    pub stake_info_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        seeds=[constants::TOKEN_SEED,signer.key.as_ref()],
        bump,
        payer=signer,
        token::mint=mint_account,
        token::authority=stake_account
    )]
    pub stake_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint=mint_account,
        associated_token::authority=signer,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    pub mint_account: Account<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[account()]
pub struct StakeInfo {
    pub stake_at_slot: u64,
    pub is_staked: bool,
}
