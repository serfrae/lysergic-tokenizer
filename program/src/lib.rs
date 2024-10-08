pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

use {
	borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
	solana_program::{declare_id, program_error::ProgramError, pubkey::Pubkey},
};

declare_id!("LSDjBzV1CdC4zeXETyLnoUddeBeQAvXXRo49j8rSguH");

// Generate the tokenizer address
pub fn get_tokenizer_address(underlying_mint: &Pubkey, expiry_date: i64) -> (Pubkey, u8) {
	let seeds = &[
		b"tokenizer",
		&underlying_mint.to_bytes()[..],
		&expiry_date.to_le_bytes(),
	];
	Pubkey::find_program_address(seeds, &crate::id())
}

// Generate the principal mint address
pub fn get_principal_mint_address(tokenizer_address: &Pubkey) -> (Pubkey, u8) {
	let seeds = &[b"principal", &tokenizer_address.to_bytes()[..]];
	Pubkey::find_program_address(seeds, &crate::id())
}

// Generate the yield mint address
pub fn get_yield_mint_address(tokenizer_address: &Pubkey) -> (Pubkey, u8) {
	let seeds = &[b"yield", &tokenizer_address.to_bytes()[..]];
	Pubkey::find_program_address(seeds, &crate::id())
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum Expiry {
	TwelveMonths,
	EighteenMonths,
	TwentyFourMonths,
}

impl Expiry {
	pub fn to_seconds(&self) -> i64 {
		match self {
			Expiry::TwelveMonths => 31536000,
			Expiry::EighteenMonths => 47304000,
			Expiry::TwentyFourMonths => 63072000,
		}
	}
	pub fn from_i64(expiry: i64) -> Result<Self, ProgramError> {
		match expiry {
			12 => Ok(Expiry::TwelveMonths),
			18 => Ok(Expiry::EighteenMonths),
			24 => Ok(Expiry::TwentyFourMonths),
			_ => Err(ProgramError::InvalidArgument),
		}
	}

	// We set the expiry date to the beginning of the day of the expiry date
	// Handling a `None` expiry date is the responsibility of the calling program
	// since this function is used both on-chain and off-chain and thus requires different
	// methods to handle the `None` case in each context.
	pub fn to_expiry_date(&self, ts: i64) -> Option<i64> {
		let expiry_seconds = self.to_seconds();
		let expiry_timestamp = ts + expiry_seconds;
		let days = expiry_timestamp / (24 * 60 * 60);
		Some(days * 24 * 60 * 60)
	}
}
