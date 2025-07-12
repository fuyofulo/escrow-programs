use anchor_lang::prelude::*;       // Anchor basic types and macros

#[account]                         // This is the main escrow account
#[derive(InitSpace)]               // Auto-calculates space needed for account
pub struct Escrow {
    pub seed: u64,                 // Seed used to derive PDA
    pub maker: Pubkey,            // The one creating the escrow
    #[max_len(10)]                // Max 10 tokens offered
    pub offered: Vec<TokenData>,  // Tokens deposited into vault
    #[max_len(10)]                // Max 10 tokens expected
    pub expected: Vec<TokenData>, // Tokens the maker wants in return
    pub bump: u8,                 // PDA bump
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)] // Custom token struct
pub struct TokenData {
    pub mint: Pubkey,             // Mint of the token
    pub amount: u64,              // Amount of tokens
}
