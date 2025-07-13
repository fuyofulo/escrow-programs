use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::Escrow;

#[derive(Accounts)]
pub struct Refund<'info> {

    // maker's pubkey
    #[account(mut)]
    maker: Signer<'info>,

    // mint of token A
    mint_a: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // escrow account
    #[account(
        mut,
        close = maker,
        has_one = mint_a,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    escrow: Account<'info, Escrow>,

    // vault account
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // assoociated token program
    associated_token_program: Program<'info, AssociatedToken>,

    // token program
    token_program: Interface<'info, TokenInterface>,

    // system program
    system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund_and_close_vault(&mut self) -> Result<()> {

        // setting up the seeds for the escrow account. this will be used in the functions below. 
        // the signer_seeds is a [&[&[u8]]] array. 
        // so we define it like this 
        // [&[ b"escrow", reference to the maker key, bytes of the escrow key, &[bump] ]] 
        let maker_key = self.maker.key();
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            maker_key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        // refunding token A back to the maker

        // step 1: define all the accounts using the TransferChecked method
        let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        // step 2: setup the cpi context for the cpi to the token program
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            &signer_seeds,
        );
        
        // step 3: call the cpi
        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;


        // closing the vault account

        // step 1: define all the accounts using the CloseAccount method
        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        // step 2: setup the cpi context fr the cpi to the token program
        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close_accounts,
            &signer_seeds,
        );

        // step 3: call the cpi
        close_account(ctx)
    }
}