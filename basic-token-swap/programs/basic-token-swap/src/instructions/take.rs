use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{close_account, transfer_checked, Mint, TokenAccount, TokenInterface, CloseAccount, TransferChecked}
};

use crate::Escrow;

#[derive(Accounts)]
pub struct Take<'info> {
    
    // taker's pubkey
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,                 // maker's pubkey
    pub mint_a: InterfaceAccount<'info, Mint>,       // mint of token A
    pub mint_b: InterfaceAccount<'info, Mint>,       // mint of token B

    // taker's token A ATA
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,

    // taker's token B ATA
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    // maker's token B ATA 
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
    escrow: Account<'info, Escrow>,

    // vault account 
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
    pub system_program: Program<'info, System>
}

impl<'info> Take<'info> {

    // transfering token B from taker to maker
    pub fn deposit(&mut self) -> Result<()> {

        // step 1: define all the accunts using the TransferChecked method
        let transfer_accounts = TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info()
        };

        // step 2: setup the cpi context for cpi to the token program
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        // step 3: call the cpi 
        transfer_checked(cpi_ctx, self.escrow.receive, self.mint_b.decimals)
    }

    pub fn withdraw_and_close_vault(&mut self) -> Result<()> {

        // setting up the seeds for the escrow account. this will be used in the functions below. 
        // the signer_seeds is a [&[&[u8]]] array. 
        // so we define it like this 
        // [&[ b"escrow", reference to the maker key, bytes of the escrow key, &[bump] ]] 
        let maker_key = self.maker.key();
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            maker_key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump]
        ]];
    
        
        // transfering token A from vault to taker's token A ATA

        // step 1: define all the accounts using the TransferChecked method
        let accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        // step 2: setup the cpi context for cpi to the token program
        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            &signer_seeds,
        );

        // step 3: call the cpi
        transfer_checked(ctx, self.vault.amount, self.mint_a.decimals)?;


        // here we are closing the vault account

        // step 1: define all the accounts related using the CLosedAccount method
        let accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.taker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        // step 2: setup the cpi context for the cpi to the token program
        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            &signer_seeds
        );

        // step 3: call the cpi
        close_account(ctx)
    }
}