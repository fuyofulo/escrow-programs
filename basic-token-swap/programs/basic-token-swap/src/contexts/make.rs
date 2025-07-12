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

    // new escrow account that is going to be created, here we are also defining the seeds and the bmp is calculated automatically
    // the vault is actually a PDA of the escrow contract for token A
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
    pub fn save_escrow(&mut self, seed: u64, receive: u64, bumps: &MakeBumps) -> Result<()> {
        self.escrow.set_inner(Escrow {                         // here we are creating a new escrow , set_inner is an anchor method to set the inner data of an account
            seed,                                              // here we pass the seed from which PDAs will be derived
            maker: self.maker.key(),                           // pubkey of the person creating an escrow
            mint_a: self.mint_a.key(),                         // the token mint being deposited   
            mint_b: self.mint_b.key(),                         // the token mint to be received 
            receive,                                           // the amount of token b to be received
            bump:bumps.escrow,                                 // bumps for the escrow being created
        });
        Ok(())
    }

    // taking the token and storing it in a vault account
    pub fn deposit(&mut self, deposit: u64) -> Result<()> {
        let transfer_accounts = TransferChecked {              // TransferChecked is a function to perform token transfer. the function ensures decimals and amounts are valid
            from: self.maker_ata_a.to_account_info(),          // source wallet
            mint: self.mint_a.to_account_info(),               // token to be transfered 
            to: self.vault.to_account_info(),                  // destination wallet
            authority: self.maker.to_account_info(),           // who has the authority to move the tokens
        };

        // CpiContext -> creates a CPI context for cross program invocation
        // CpiContext::new -> builds a context for CPI, specifying programs and accounts
        // arguments:
        // self.token_program.to_account_info() -> the program to invoke (Token Program as account info)
        // transfer_accounts -> the struct of accounts for the transfer

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);      
        
        // transfer_checked -> calls the cpi to perform token transfer
        // invokes the token program instruction with decimal checks. this is the CPI call
        // arguments:
        // cpi_ctx -> the cpi context we created above 
        // deposit -> the token a amont being deposited (u64)
        // self.mint_a.decimals -> ensures the decimals are correct (u8)
        
        transfer_checked(cpi_ctx, deposit, self.mint_a.decimals)
    }
}
