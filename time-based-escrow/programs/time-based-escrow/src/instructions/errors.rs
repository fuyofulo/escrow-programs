use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("The escrow has not yet expired.")]
    EscrowNotExpired,
    #[msg("The escrow has expired.")]
    EscrowExpired,
} 