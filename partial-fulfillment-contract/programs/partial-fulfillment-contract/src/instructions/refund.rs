use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::state::Escrow;

#[derive(Accounts)]
pub struct Refund<'info> {
    // Maker of the escrow
    #[account(mut)]
    pub maker: Signer<'info>,

    // Mint of token A (escrowed token)
    pub mint_a: InterfaceAccount<'info, Mint>,

    // Maker's token A ATA (to receive refund)
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // Escrow account
    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = mint_a,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    // Vault account (holds token A)
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // SPL program interfaces
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund_and_close(&mut self) -> Result<()> {
        // Build signer seeds for the escrow PDA
        let maker_key = self.maker.key();
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            maker_key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        // 1. Transfer remaining token A from vault to maker
        let transfer = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer,
            &signer_seeds,
        );

        // Transfer only the remaining amount
        transfer_checked(ctx, self.escrow.remaining_amount, self.mint_a.decimals)?;

        // 2. Close the vault account
        let close = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close,
            &signer_seeds,
        );

        close_account(ctx)
    }
}
