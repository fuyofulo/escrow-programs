// state.rs: this is the state of the program, it is the data that is stored in the account

use anchor_lang::prelude::*;  // this is the prelude of the anchor lang, it contains the basic types and macros

#[account]                 // declares that this is going to be an account
#[derive(InitSpace)]       // auto-calculates the space needed for the account 
pub struct Escrow {        // this is the struct of the account, we are defining the fields of the account
    pub seed: u64,         // this is the seed of the account
    pub maker: Pubkey,     // this is the maker of the account
    pub mint_a: Pubkey,    // this is the mint of the token A
    pub mint_b: Pubkey,    // this is the mint of the token B
    pub receive: u64,      // this is the amount of token B that the maker is receiving
    pub expires_at: i64,   // this is the expiry timestamp (unix seconds)
    pub bump: u8,          // this is the bump of the account
}
