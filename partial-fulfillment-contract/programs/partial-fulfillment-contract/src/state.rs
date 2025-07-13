// state.rs

use anchor_lang::prelude::*;

/// This is the state of the escrow account.
/// It stores how much the maker deposited,
/// how much has been taken,
/// and how much token B is expected in return.

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub seed: u64,                // unique seed to derive PDA
    pub maker: Pubkey,            // escrow creator
    pub mint_a: Pubkey,           // token being offered by the maker
    pub mint_b: Pubkey,           // token expected from takers
    pub total_amount: u64,        // total amount deposited (token A)
    pub remaining_amount: u64,    // remaining token A not yet taken
    pub receive_per_token: u64,   // how much token B the maker wants for 1 token A
    pub bump: u8,                 // PDA bump
}
