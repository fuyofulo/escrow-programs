use anchor_lang::prelude::*;

pub mod contexts;
use contexts::*;

pub mod state;
pub use state::*;

declare_id!("J1e4TfaFKrYvNM1EeyM1Pnh1XggW6HgFz9bFdNWuwcX3");

#[program]
pub mod time_based_escrow {
    use super::*;

    // creates a new escrow
    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64, expires_at: i64) -> Result<()> {
        ctx.accounts.deposit(deposit)?;
        ctx.accounts.save_escrow(seed, receive, expires_at, &ctx.bumps)
    }

    // taker fulfills the swap
    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close_vault()
    }

    // maker gets refund after expiry
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }
}
