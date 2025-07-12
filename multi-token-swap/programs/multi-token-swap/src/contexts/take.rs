use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, TokenInterface,
        TransferChecked, CloseAccount
    },
};

use crate::{Escrow};

#[derive(Accounts)]
pub struct Take<'info> {
    // the taker who is fulfilling the escrow
    #[account(mut)]
    pub taker: Signer<'info>,

    // the escrow account being closed
    #[account(
        mut,
        close = taker,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    // the original escrow creator (maker)
    /// CHECK: we're only transferring tokens to this address
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,

    // programs
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Take<'info> {
    pub fn execute(&self, remaining: &[AccountInfo<'info>]) -> Result<()> {
        let _escrow_key = self.escrow.key();
        let maker_key = self.maker.key();

        let mut account_cursor = 0;

        // Transfer expected tokens from taker → maker
        for token in &self.escrow.expected {
            let mint = &remaining[account_cursor];
            account_cursor += 1;

            let taker_ata = &remaining[account_cursor];
            account_cursor += 1;

            let maker_ata = &remaining[account_cursor];
            account_cursor += 1;

            let mint_data = anchor_spl::token::Mint::try_deserialize(&mut &mint.data.borrow()[..])?;
            let decimals = mint_data.decimals;

            let transfer_accounts = TransferChecked {
                from: taker_ata.clone(),
                mint: mint.clone(),
                to: maker_ata.clone(),
                authority: self.taker.to_account_info(),
            };

            let ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);
            transfer_checked(ctx, token.amount, decimals)?;
        }

        // Seeds for vault PDA authority
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            maker_key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        // Transfer offered tokens from vault → taker, then close vault
        for token in &self.escrow.offered {
            let mint = &remaining[account_cursor];
            account_cursor += 1;

            let vault = &remaining[account_cursor];
            account_cursor += 1;

            let taker_ata = &remaining[account_cursor];
            account_cursor += 1;

            let mint_data = anchor_spl::token::Mint::try_deserialize(&mut &mint.data.borrow()[..])?;
            let decimals = mint_data.decimals;

            // Transfer from vault to taker
            let transfer_accounts = TransferChecked {
                from: vault.clone(),
                mint: mint.clone(),
                to: taker_ata.clone(),
                authority: self.escrow.to_account_info(),
            };

            let transfer_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                transfer_accounts,
                &signer_seeds,
            );

            transfer_checked(transfer_ctx, token.amount, decimals)?;

            // Close the vault account
            let close_accounts = CloseAccount {
                account: vault.clone(),
                destination: self.taker.to_account_info(),
                authority: self.escrow.to_account_info(),
            };

            let close_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                close_accounts,
                &signer_seeds,
            );

            close_account(close_ctx)?;
        }

        Ok(())
    }
}

