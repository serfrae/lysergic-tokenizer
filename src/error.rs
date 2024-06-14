use {
    num_derive::FromPrimitive,
    num_traits::FromPrimitive as FromPrimitiveTrait,
    solana_program::{
        decode_error::DecodeError,
        msg,
        program_error::{PrintProgramError, ProgramError},
    },
    thiserror::Error,
};

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum LysergicTokenizerError {
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Tokenizer Already Initialized")]
    TokenizerAlreadyInitialized,
    #[error("Tokenizer Not Initialized")]
    TokenizerNotInitialized,
    #[error("Incorrect Account Address")]
    InvalidUserAccount,
    #[error("Incorrect Tokenizer Address")]
    IncorrectTokenizerAddress,
    #[error("Invalid Expiry Date")]
    InvalidExpiryDate,
    #[error("Incorrect Vault Address")]
    IncorrectVaultAddress,
    #[error("Incorrect Underlying Mint Address")]
    IncorrectUnderlyingMintAddress,
    #[error("Incorrect Principal Mint Address")]
    IncorrectPrincipalMintAddress,
    #[error("Incorrect Yield Mint Address")]
    IncorrectYieldMintAddress,
    #[error("Expiry Date Has Elapsed")]
    ExpiryDateElapsed,
    #[error("Expiry Date Has Not Elapsed")]
    ExpiryDateNotElapsed,
}


impl From<LysergicTokenizerError> for ProgramError {
    fn from(e: LysergicTokenizerError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for LysergicTokenizerError {
    fn type_of() -> &'static str {
        "Lysergic tokenizer error"
    }
}

impl PrintProgramError for LysergicTokenizerError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + FromPrimitiveTrait + PrintProgramError,
    {
        msg!(&self.to_string())
    }
}
