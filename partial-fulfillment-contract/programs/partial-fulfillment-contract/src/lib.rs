use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;

use instructions::*;

declare_id!("7PjmkG4xTrQfM3rtGu6cA7B27eLBy56qJZ5gAq7NpCYH");

#[program]
pub mod partial_fulfillment_escrow {
    use super::*;

    /// Maker initializes escrow and deposits token A
    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.deposit(deposit)?;
        ctx.accounts.save_escrow(seed, deposit, receive, &ctx.bumps)
    }

    /// Taker partially fulfills the order with token B and receives proportional token A
    pub fn take(ctx: Context<Take>, amount_b: u64) -> Result<()> {
        ctx.accounts.execute(amount_b)
    }

    /// Maker claims refund of remaining token A and closes the vault + escrow
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close()
    }
}
