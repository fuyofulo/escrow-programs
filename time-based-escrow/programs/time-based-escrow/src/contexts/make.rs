use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {

    // person who is creating an escrow -> maker
    #[account(mut)]
    pub maker: Signer<'info>,

    // token A mint address
    #[account(
        mint::token_program = token_program,
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    // token B mint address
    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    // maker's token A ATA
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // new escrow account that is going to be created, here we are also defining the seeds and the bump is calculated automatically
    #[account(
        init,
        payer = maker,
        space = 8 + Escrow::INIT_SPACE,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,

    // new vault account that is going to be created 
    #[account(
        init, 
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // associated token program 
    pub associated_token_program: Program<'info, AssociatedToken>,

    // token program
    pub token_program: Interface<'info, TokenInterface>,

    // system program
    pub system_program: Program<'info, System>,

}

impl<'info> Make<'info> {

    // creating a new escrow
    pub fn save_escrow(
        &mut self,
        seed: u64,
        receive: u64,
        duration: i64,
        bumps: &MakeBumps,
    ) -> Result<()> {
        let clock = Clock::get()?;                           // get the current timestamp from the Solana clock sysvar
        let expires_at = clock.unix_timestamp + duration;    // set the expiry by adding duration in seconds

        self.escrow.set_inner(Escrow {                       // create a new escrow using set_inner
            seed,                                             // seed from which PDA is derived
            maker: self.maker.key(),                          // pubkey of escrow creator
            mint_a: self.mint_a.key(),                        // token being deposited
            mint_b: self.mint_b.key(),                        // token expected in return
            receive,                                          // amount of token B to receive
            expires_at,                                       // unix timestamp when escrow expires
            bump: bumps.escrow,                               // bump of escrow PDA
        });

        Ok(())
    }

    // depositing token A into the vault
    pub fn deposit(&mut self, deposit: u64) -> Result<()> {
        let transfer_accounts = TransferChecked {
            from: self.maker_ata_a.to_account_info(),        // source: maker's token A ATA
            mint: self.mint_a.to_account_info(),             // mint of token A
            to: self.vault.to_account_info(),                // destination: vault PDA
            authority: self.maker.to_account_info(),         // authority: maker
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        transfer_checked(cpi_ctx, deposit, self.mint_a.decimals)
    }
}
