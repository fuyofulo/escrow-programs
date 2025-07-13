use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Invalid amount provided")]
    InvalidAmount,
    #[msg("Amount exceeds remaining amount in escrow")]
    ExceedsRemainingAmount,
    #[msg("Calculation overflow occurred")]
    Overflow,
} 