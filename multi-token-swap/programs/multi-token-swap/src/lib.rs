use anchor_lang::prelude::*;

pub mod contexts;
use contexts::*;

pub mod state;
pub use state::*;

declare_id!("Bxkzkxfovwu1PUn2xTfKZ2dCwPZXsxosimEh2t7ndQ1B");

#[program]
pub mod multi_token_swap {
    use super::*;

    pub fn make<'info>(
        ctx: Context<'_, '_, '_, 'info, Make<'info>>,
        seed: u64,
        offered: Vec<TokenData>,
        expected: Vec<TokenData>,
    ) -> Result<()> {
        let _escrow_key = ctx.accounts.escrow.key();
        let remaining = &ctx.remaining_accounts;

        let mut account_cursor = 0;

        for token in &offered {
            // mint
            let mint_account = &remaining[account_cursor];
            account_cursor += 1;

            // maker ATA (source)
            let from_account = &remaining[account_cursor];
            account_cursor += 1;

            // vault ATA (destination)
            let to_account = &remaining[account_cursor];
            account_cursor += 1;

            // Load mint to get decimals
            let mint_data = anchor_spl::token::Mint::try_deserialize(&mut &mint_account.data.borrow()[..])?;
            let decimals = mint_data.decimals;

            ctx.accounts.deposit_single_token(
                mint_account,
                from_account,
                to_account,
                token.amount,
                decimals,
            )?;
        }

        ctx.accounts.save_escrow(seed, offered, expected, ctx.bumps.escrow)
    }

    pub fn take<'info>(ctx: Context<'_, '_, '_, 'info, Take<'info>>) -> Result<()> {
        ctx.accounts.execute(&ctx.remaining_accounts)
    }

    pub fn refund<'info>(ctx: Context<'_, '_, '_, 'info, Refund<'info>>) -> Result<()> {
        ctx.accounts.refund_and_close_all(&ctx.remaining_accounts)
    }


}
