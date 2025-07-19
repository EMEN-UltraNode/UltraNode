use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke_signed, system_instruction};
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("8XiTe11YjconcJWDPE7SEUCNmfWFpWZmNRXDKoHQcjeC");

#[program]
pub mod ultra_node_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn submit_verified_proof(
        ctx: Context<SubmitVerifiedProof>,
        proof_hash: [u8; 32],
        merkle_root: [u8; 32],
        validator_signature: [u8; 64],
    ) -> Result<()> {
        // Save proof commitment (for audit trail)
        let proof_account = &mut ctx.accounts.proof_account;
        proof_account.validator = ctx.accounts.validator.key();
        proof_account.submitter = ctx.accounts.submitter.key();
        proof_account.proof_hash = proof_hash;
        proof_account.merkle_root = merkle_root;

        // Transfer BONK reward from vault to submitter
        let seeds = &[b"vault".as_ref(), &[ctx.accounts.vault.bump]];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.submitter_token_account.to_account_info(),
            authority: ctx.accounts.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );

        token::transfer(cpi_ctx, 1_000_000)?; // reward: 0.001 BONK (assuming 6 decimals)

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
#[instruction(proof_hash: [u8; 32])]
pub struct SubmitVerifiedProof<'info> {
    #[account(init, payer = submitter, space = 8 + ProofAccount::SIZE, seeds = [b"proof", &proof_hash], bump)]
    pub proof_account: Account<'info, ProofAccount>,

    #[account(mut)]
    pub submitter: Signer<'info>,

    /// CHECK: Validator pubkey (used for indexing, not signing)
    pub validator: UncheckedAccount<'info>,

    #[account(mut, seeds = [b"vault"], bump = vault.bump)]
    pub vault: Account<'info, Vault>,

    #[account(mut, associated_token::mint = bonk_mint, associated_token::authority = vault)]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(mut, associated_token::mint = bonk_mint, associated_token::authority = submitter)]
    pub submitter_token_account: Account<'info, TokenAccount>,

    pub bonk_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,

    pub rent: Sysvar<'info, Rent>,

    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
}

#[account]
pub struct ProofAccount {
    pub validator: Pubkey,
    pub submitter: Pubkey,
    pub proof_hash: [u8; 32],
    pub merkle_root: [u8; 32],
}

impl ProofAccount {
    pub const SIZE: usize = 32 + 32 + 32 + 32;
}

#[account]
pub struct Vault {
    pub bump: u8,
}
