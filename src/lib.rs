pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

use {
    solana_program::{
        pubkey::Pubkey,
        declare_id,
        program_error::ProgramError,
    },
    borsh::{BorshSerialize, BorshSchema, BorshDeserialize},
    chrono::{Utc, Duration, NaiveDateTime},
};

declare_id!("LSDjBzV1CdC4zeXETyLnoUddeBeQAvXXRo49j8rSguH");

// Generate the tokenizer address
pub fn get_tokenizer_address(program_id: &Pubkey, underlying_mint: &Pubkey, expiry_date: i64) -> Pubkey {
    let seeds = &[&underlying_mint.to_bytes()[..], &expiry_date.to_le_bytes()];
    let (tokenizer_key, _) = Pubkey::find_program_address(seeds, program_id);
    tokenizer_key
}

// Generate the principal mint address
pub fn get_principal_mint_address(
    program_id: &Pubkey,
    tokenizer_address: &Pubkey,
) -> Pubkey {
    let seeds = &[
        &tokenizer_address.to_bytes()[..],
        b"principal",
    ];
    let (principal_mint_key, _) = Pubkey::find_program_address(seeds, program_id);
    principal_mint_key
}

// Generate the yield mint address
pub fn get_yield_mint_address(
    program_id: &Pubkey,
    tokenizer_address: &Pubkey,
) -> Pubkey {
    let seeds = &[
        &tokenizer_address.to_bytes()[..],
        b"yield",
    ];
    let (yield_mint_key, _) = Pubkey::find_program_address(seeds, program_id);
    yield_mint_key
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
    pub fn to_expiry_date(&self) -> Option<i64> {
        let now = Utc::now();
        let expiry_seconds = self.to_seconds();
        let expiry_duration = Duration::seconds(expiry_seconds);
        let expiry_date = now + expiry_duration;
        let expiry_date: Option<NaiveDateTime> = expiry_date.date_naive().and_hms_opt(0,0,0);
        Some(expiry_date?.and_utc().timestamp())
    }
}

