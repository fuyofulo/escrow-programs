use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, TokenInterface, TransferChecked, CloseAccount,
    },
};

use crate::{Escrow};

#[derive(Accounts)]
pub struct Refund<'info> {

    // maker's pubkey (escrow creator)
    #[account(mut)]
    pub maker: Signer<'info>,

    // escrow account being closed
    #[account(
        mut,
        close = maker,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    // associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,

    // token program
    pub token_program: Interface<'info, TokenInterface>,

    // system program
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund_and_close_all(&self, remaining: &[AccountInfo<'info>]) -> Result<()> {
        let maker_key = self.maker.key();

        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            maker_key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        let mut cursor = 0;

        for token in &self.escrow.offered {
            let mint = &remaining[cursor];
            cursor += 1;

            let vault = &remaining[cursor];
            cursor += 1;

            let maker_ata = &remaining[cursor];
            cursor += 1;

            // load mint to get decimals
            let mint_data = anchor_spl::token::Mint::try_deserialize(&mut &mint.data.borrow()[..])?;
            let decimals = mint_data.decimals;

            // refund transfer from vault → maker
            let transfer_accounts = TransferChecked {
                from: vault.clone(),
                mint: mint.clone(),
                to: maker_ata.clone(),
                authority: self.escrow.to_account_info(),
            };

            let transfer_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                transfer_accounts,
                &signer_seeds,
            );

            transfer_checked(transfer_ctx, token.amount, decimals)?;

            // close vault
            let close_accounts = CloseAccount {
                account: vault.clone(),
                destination: self.maker.to_account_info(),
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
