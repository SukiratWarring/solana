use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount,Transfer,transfer},
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
        Ok(())
    }
    pub fn stake(_ctx: Context<Stake>,amount:u64) -> Result<()> {
        let stake_info_acc = &mut _ctx.accounts.stake_info_account;
        let user_stake_account = &mut _ctx.accounts.stake_account;
        if stake_info_acc.is_staked {
            return Err(Errors::IsStaked.into());
        }
        if amount<=0{
            return Err(Errors::NoTokens.into());
        }
        let clock=Clock::get()?;
        //update STAKE INFO
        stake_info_acc.stake_at_slot=clock.slot;
        stake_info_acc.is_staked=true;
        //update USER STAKE AMT
        let staking_amt=amount.checked_mul(10u64.pow(_ctx.accounts.mint_account.decimals as u32)).unwrap();
        transfer(
            CpiContext::new(
                _ctx.accounts.token_program.to_account_info(),
                Transfer{
                    from:_ctx.accounts.user_token_account.to_account_info(),
                    to:user_stake_account.to_account_info(),
                    authority:_ctx.accounts.signer.to_account_info()
                }
            ),
            staking_amt
        )?;
        Ok(())
    }
    pub fn unstake(ctx: Context<Unstake>,un_stake_amt:u64) -> Result<()> {
        let stake_info_acc = &mut ctx.accounts.stake_info_account;
        if !stake_info_acc.is_staked{
            return Err(Errors::NotStaked.into());
        }
        if un_stake_amt<=0{
            return Err(Errors::NoTokens.into());
        }
        let user_stake_account = &mut ctx.accounts.stake_account;
        let vault_account=&mut ctx.accounts.token_vault_account;
        let current_slot=Clock::get()?.slot;
        let slots_passed=current_slot-stake_info_acc.stake_at_slot;
        let calculate_amt=slots_passed.checked_mul(10u64.pow(ctx.accounts.mint_account.decimals as u32)).unwrap();
        //1) In Solana, when you need a PDA to sign a transaction, you provide the seeds 
        // used to derive the PDA along with the bump value. The runtime can then recreate 
        // the PDA's address and use it as a signer.

        //2) In the context of Solana programs, when a Program Derived Address (PDA) is used as a signer,
        //  the actual signature is provided by the program itself. 
        // let bump=ctx.accounts.stake_account.bump;
        let bump=*ctx.bumps.get("stake_account").unwrap();
        let signer: &[&[&[u8]]]=&[&[constants::VAULT_SEED,&[bump]]];
        //send the reward token
        transfer(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer{
            from:vault_account.to_account_info(),
            to:ctx.accounts.user_token_account.to_account_info(),
            authority:vault_account.to_account_info()
            },
            signer
        ),
        calculate_amt)?;   
        let staker=ctx.accounts.signer.key();
        let bump=*ctx.bumps.get("token_vault_account").unwrap();
        let signer: &[&[&[u8]]]=&[&[constants::TOKEN_SEED,staker.as_ref(),&[bump]]];
        //return the staked token
        transfer(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer{
            from:user_stake_account.to_account_info(),
            to:ctx.accounts.user_token_account.to_account_info(),
            authority:user_stake_account.to_account_info()
            },
            signer
        ),
        un_stake_amt)?;
     
        //update stake_info
        if user_stake_account.amount==0{
            stake_info_acc.is_staked=false;
        }
        stake_info_acc.stake_at_slot=Clock::get()?.slot;
        Ok(())
    }
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
        space=8+std::mem::size_of::<StakeInfo>()  
    )]
    pub stake_info_account: Account<'info, StakeInfo>,

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
    //this the user who is comming to stake's token account
    pub user_token_account: Account<'info, TokenAccount>,
    pub mint_account: Account<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[account]

pub struct StakeInfo {
    pub stake_at_slot: u64,
    pub is_staked: bool,
}

#[derive(Accounts)]
pub struct Unstake<'info>{

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds=[constants::VAULT_SEED],
        bump
    )]
    pub token_vault_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds=[constants::STAKE_INFO_SEED,signer.key.as_ref()],
        bump,
    )]
    //Account which holds the time slot info
    pub stake_info_account: Account<'info, StakeInfo>,

    #[account(
        mut,
        seeds=[constants::TOKEN_SEED,signer.key.as_ref()],
        bump,
    )]
    //Account that holds the token balance 
    pub stake_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint=mint_account,
        associated_token::authority=signer,
    )]
    //this the user who is comming to unstake token account
    pub user_token_account: Account<'info, TokenAccount>,
    // This is the account which holds all the info like decimals,tokensupply of the token being exchanged
    pub mint_account: Account<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[error_code]

pub enum Errors {
    #[msg("The user has already staked")]
    IsStaked,
    #[msg("The user has not staked")]
    NotStaked,
    #[msg("No token to stake")]
    NoTokens,
}
