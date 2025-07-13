use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, Mint, TokenAccount, TokenInterface,
        CloseAccount, TransferChecked,
    },
};

use crate::Escrow;
use crate::contexts::errors::EscrowError;

#[derive(Accounts)]
pub struct Take<'info> {

    // taker's pubkey
    #[account(mut)]
    pub taker: Signer<'info>,

    // maker's pubkey
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    // mint of token A
    pub mint_a: InterfaceAccount<'info, Mint>,

    // mint of token B
    pub mint_b: InterfaceAccount<'info, Mint>,

    // taker's token A ATA (to receive token A)
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,

    // taker's token B ATA (to send token B)
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    // maker's token B ATA (to receive token B)
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    // escrow account
    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = mint_a,
        has_one = mint_b,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    // vault account (holds token A temporarily)
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,

    // token program
    pub token_program: Interface<'info, TokenInterface>,

    // system program
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {

    // transferring token B from taker to maker
    pub fn deposit(&mut self) -> Result<()> {

        // check if escrow has expired
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp < self.escrow.expires_at,
            EscrowError::EscrowExpired
        );

        // step 1: define all the accounts using the TransferChecked method
        let transfer_accounts = TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        // step 2: setup the cpi context for cpi to the token program
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        // step 3: call the cpi
        transfer_checked(cpi_ctx, self.escrow.receive, self.mint_b.decimals)
    }

    // transferring token A from vault to taker and closing the vault
    pub fn withdraw_and_close_vault(&mut self) -> Result<()> {

        // set up signer seeds for the escrow PDA
        let maker_key = self.maker.key();
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            maker_key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        // transfer token A from vault to taker's token A ATA
        let accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            &signer_seeds,
        );

        transfer_checked(ctx, self.vault.amount, self.mint_a.decimals)?;


        // close the vault account
        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.taker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close_accounts,
            &signer_seeds,
        );

        close_account(ctx)
    }
}
w

