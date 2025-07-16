#![allow(deprecated)]


use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, TokenAccount, Token};

declare_id!("CTq9UQdmsbWZoKh1hsioRyC4wiqCrx76UPH9wiKUwcFh");

#[program]
pub mod ultra_node_program {
    use super::*;

    pub fn submit_proof(
        ctx: Context<SubmitProof>,
        tx_hash: [u8; 32],
        merkle_root: [u8; 32],
    ) -> Result<()> {
        let node = &mut ctx.accounts.node;
        node.wallet = ctx.accounts.user.key();
        node.tx_count += 1;
        node.last_tx = tx_hash;
        node.merkle_root = merkle_root;
        node.bump = ctx.bumps.node;
        Ok(())
    }

    pub fn record_uptime(ctx: Context<RecordUptime>) -> Result<()> {
        let node = &mut ctx.accounts.node;
        node.uptime += 1;
        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let node = &mut ctx.accounts.node;
        let total = node.tx_count as u64 + (node.uptime as u64 / 48) * 5;

        let signer_seeds: &[&[u8]] = &[b"auth", &[node.bump]];
        let signer: &[&[&[u8]]] = &[signer_seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.to.to_account_info(),
                authority: ctx.accounts.auth.to_account_info(),
            },
            signer,
        );

        token::mint_to(cpi_ctx, total)?;

        node.tx_count = 0;
        node.uptime = 0;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SubmitProof<'info> {
    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"node", user.key().as_ref()],
        bump,
        space = 8 + Node::LEN,
    )]
    pub node: Account<'info, Node>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RecordUptime<'info> {
    #[account(
        mut,
        seeds = [b"node", user.key().as_ref()],
        bump = node.bump
    )]
    pub node: Account<'info, Node>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(
        mut,
        seeds = [b"node", user.key().as_ref()],
        bump = node.bump
    )]
    pub node: Account<'info, Node>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    /// CHECK: PDA derived from [b"auth"]
    #[account(seeds = [b"auth"], bump)]
    pub auth: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub user: Signer<'info>,
}

#[account]
pub struct Node {
    pub wallet: Pubkey,
    pub tx_count: u32,
    pub uptime: u32,
    pub last_tx: [u8; 32],
    pub merkle_root: [u8; 32],
    pub bump: u8,
}

impl Node {
    pub const LEN: usize = 32 + 4 + 4 + 32 + 32 + 1;
}
