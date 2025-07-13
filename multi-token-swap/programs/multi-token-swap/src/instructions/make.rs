use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, TokenInterface, TransferChecked},
};

use crate::{Escrow, TokenData};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {

    // the person who is creating the escrow -> maker
    #[account(mut)]
    pub maker: Signer<'info>,

    // new escrow account that is going to be created
    #[account(
        init,
        payer = maker,
        space = 8 + Escrow::INIT_SPACE,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,

    // system program
    pub system_program: Program<'info, System>,

    // token program
    pub token_program: Interface<'info, TokenInterface>,

    // associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Make<'info> {

    // create the escrow state
    pub fn save_escrow(
        &mut self,
        seed: u64,
        offered: Vec<TokenData>,
        expected: Vec<TokenData>,
        bump: u8,
    ) -> Result<()> {
        self.escrow.set_inner(Escrow {
            seed,                                      // seed for PDA
            maker: self.maker.key(),                   // pubkey of escrow creator
            offered,                                   // list of tokens being deposited
            expected,                                  // list of tokens expected
            bump,                                      // bump for escrow PDA
        });
        Ok(())
    }

    // helper function to transfer token from maker to vault
    pub fn deposit_single_token(
        &self,
        mint: &AccountInfo<'info>,
        from: &AccountInfo<'info>,
        to: &AccountInfo<'info>,
        amount: u64,
        decimals: u8,
    ) -> Result<()> {
        let transfer_accounts = TransferChecked {
            from: from.clone(),
            mint: mint.clone(),
            to: to.clone(),
            authority: self.maker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        transfer_checked(cpi_ctx, amount, decimals)
    }
}
