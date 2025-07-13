use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::state::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    // Maker is the person creating the escrow
    #[account(mut)]
    pub maker: Signer<'info>,

    // Mint of the token being offered (token A)
    #[account(
        mint::token_program = token_program,
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    // Mint of the token to be received (token B)
    #[account(
        mint::token_program = token_program,
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    // Maker's token A ATA
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // New escrow account to be initialized
    #[account(
        init,
        payer = maker,
        space = 8 + Escrow::INIT_SPACE,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    // Vault account where maker's token A will be stored
    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // Programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Make<'info> {
    /// This function transfers tokens from the maker to the vault
    pub fn deposit(&mut self, deposit: u64) -> Result<()> {
        let accounts = TransferChecked {
            from: self.maker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), accounts);

        transfer_checked(cpi_ctx, deposit, self.mint_a.decimals)
    }

    /// This function saves the escrow state after deposit
    pub fn save_escrow(
        &mut self,
        seed: u64,
        deposit: u64,
        receive_per_token: u64,
        bumps: &MakeBumps,
    ) -> Result<()> {
        self.escrow.set_inner(Escrow {
            seed,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            total_amount: deposit,
            remaining_amount: deposit,
            receive_per_token,
            bump: bumps.escrow,
        });
        Ok(())
    }
}
