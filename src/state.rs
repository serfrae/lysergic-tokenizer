use {
	borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
	solana_program::pubkey::Pubkey,
};

pub const LYSERGIC_TOKENIZER_STATE_SIZE: usize = 1 + 32 + 32 + 32 + 32 + 32 + 8 + 8; // 184 bytes

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, Debug, PartialEq)]
pub struct LysergicTokenizerState {
    pub bump: u8,
	pub authority: Pubkey,
	pub principal_token_mint: Pubkey,
	pub yield_token_mint: Pubkey,
	pub underlying_mint: Pubkey,
	pub underlying_vault: Pubkey,
	pub expiry_date: i64,
	pub fixed_apy: u64,
}
