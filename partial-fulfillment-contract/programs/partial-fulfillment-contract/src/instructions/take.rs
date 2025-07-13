use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::state::Escrow;
use crate::instructions::EscrowError;

#[derive(Accounts)]
pub struct Take<'info> {
    // The person taking a portion of the escrow
    #[account(mut)]
    pub taker: Signer<'info>,

    // The escrow creator
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    // Mints of token A and token B
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,

    // Taker's token A ATA (to receive from vault)
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // Taker's token B ATA (to pay the maker)
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_b: InterfaceAccount<'info, TokenAccount>,

    // Maker's token B ATA (receives payment)
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,

    // Escrow account
    #[account(
        mut,
        has_one = maker,
        has_one = mint_a,
        has_one = mint_b,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    // Vault account holding token A
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {
    pub fn execute(&mut self, amount_to_take: u64) -> Result<()> {
        require!(amount_to_take > 0, EscrowError::InvalidAmount);
        require!(
            amount_to_take <= self.escrow.remaining_amount,
            EscrowError::ExceedsRemainingAmount
        );

        // Calculate token B amount: receive_per_token * amount_to_take
        let amount_b = self
            .escrow
            .receive_per_token
            .checked_mul(amount_to_take)
            .ok_or(EscrowError::Overflow)?;

        // 1. Transfer token B from taker to maker
        let transfer_b = TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        let cpi_ctx_b = CpiContext::new(self.token_program.to_account_info(), transfer_b);
        transfer_checked(cpi_ctx_b, amount_b, self.mint_b.decimals)?;

        // 2. Transfer token A from vault to taker
        let maker_key = self.maker.key();
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            maker_key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        let transfer_a = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_ctx_a = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_a,
            &signer_seeds,
        );
        transfer_checked(cpi_ctx_a, amount_to_take, self.mint_a.decimals)?;

        // 3. Update state
        self.escrow.remaining_amount = self
            .escrow
            .remaining_amount
            .checked_sub(amount_to_take)
            .ok_or(EscrowError::Overflow)?;

        // 4. If everything is taken, close vault and escrow
        if self.escrow.remaining_amount == 0 {
            // Close vault
            let close = CloseAccount {
                account: self.vault.to_account_info(),
                destination: self.taker.to_account_info(),
                authority: self.escrow.to_account_info(),
            };

            let cpi_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                close,
                &signer_seeds,
            );
            close_account(cpi_ctx)?;

            // Escrow account will be closed by the runtime once instruction ends
            self.escrow.close(self.taker.to_account_info())?;
        }

        Ok(())
    }
}
