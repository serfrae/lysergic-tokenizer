use {
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    crate::Expiry,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
    spl_associated_token_account, spl_token,
};

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, Debug, PartialEq)]
pub enum LysergicTokenizerInstruction {
    /// Initializes the LysergicTokenizer
    ///
    /// Accounts expected:
    ///
    /// 0. `[writable]` LysergicTokenizer account
    /// 1. `[writable, signer]` Authority
    /// 2. `[writable]` Underlying vault account
    /// 3. `[]` Underlying mint account
    /// 4. `[]` Token program
    /// 5. `[]` System program
    InitializeLysergicTokenizer {
        /// The public key of the underlying vault
        underlying_vault: Pubkey,
        /// The public key of the underlying mint
        underlying_mint: Pubkey,
        /// The public key of the principal token mint
        principal_token_mint: Pubkey,
        /// The public key of the yield token mint
        yield_token_mint: Pubkey,
        /// The expiry of the LysergicTokenizer
        expiry: Expiry,
    },

    /// Initializes the principal and yield token mints
    ///
    /// Accounts expected:
    ///
    /// 0. `[writable]` LysergicTokenizer account
    /// 1. `[writable, signer]` Authority
    /// 2. `[writable]` Principal token mint account
    /// 3. `[writable]` Yield token mint account
    /// 4. `[]` Token program
    InitializeMints {
        /// The public key of the underlying mint
        underlying_mint: Pubkey,
        /// The expiry of the LysergicTokenizer
        expiry: Expiry,
    },

    /// Helper function to initialize the LysergicTokenizer and the mints
    ///
    /// Accounts expected:
    ///
    /// 0. `[writable]` LysergicTokenizer account
    /// 1. `[writable,signer]` Authority
    /// 2. `[writable]` Underlying vault account
    /// 3. `[writable]` Principal token mint account
    /// 4. `[writable]` Yield token mint account
    /// 5. `[]` Token program
    /// 6. `[]` System program
    InitializeTokenizerAndMints {
        /// The public key of the underlying vault
        underlying_vault: Pubkey,
        /// The public key of the underlying mint
        underlying_mint: Pubkey,
        /// The public key of the principal token mint
        principal_token_mint: Pubkey,
        /// The public key of the yield token mint
        yield_token_mint: Pubkey,
        /// The expiry of the LysergicTokenizer
        expiry: Expiry,
    },

    /// Deposits the underlying token into the LysergicTokenizer
    ///
    /// Accounts expected:
    ///
    /// 0. `[writable]` LysergicTokenizer account
    /// 1. `[writable]` Underlying vault account
    /// 2. `[writable, signer]` User account
    /// 3. `[writable]` User underlying token account
    /// 4. `[]` Token program
    DepositUnderlying {
        /// The amount of the underlying token to deposit
        amount: u64,
    },

    /// Tokenizes the underlying token into principal tokens
    ///
    /// Accounts expected:
    ///
    /// 0. `[writable]` LysergicTokenizer account
    /// 1. `[writable]` Principal token mint account
    /// 2. `[writable, signer]` User account
    /// 3. `[writable]` User principal token account
    /// 4. `[]` Token program
    /// 5. `[]` Associated token account program
    TokenizePrincipal {
        /// The amount of the underlying token to tokenize
        amount: u64,
    },
    /// Tokenizes the underlying token into yield tokens
    ///
    /// Accounts expected:
    ///
    /// 0. `[writable]` LysergicTokenizer account
    /// 1. `[writable]` Yield token mint account
    /// 2. `[writable, signer]` User account
    /// 3. `[writable]` User yield token account
    /// 4. `[]` Token program
    TokenizeYield {
        /// The amount of the underlying token to tokenize
        amount: u64,
    },

    /// Helper function to deposit and tokenize the underlying token
    ///
    /// Accounts expected:
    ///
    /// 0. `[writable]` LysergicTokenizer account
    /// 1. `[writable]` Underlying vault account
    /// 2. `[writable]` Principal token mint account
    /// 3. `[writable]` Yield token mint account
    /// 4. `[writable, signer]` User account
    /// 5. `[writable]` User underlying token account
    /// 6. `[writable]` User principal token account
    /// 7. `[writable]` User yield token account
    /// 8. `[]` Token program
    /// 9. `[]` Associated token account program
    DepositAndTokenize {
        /// The amount of the underlying token to deposit
        amount: u64,
    },

    /// Redeems the principal and yield tokens for the underlying token
    ///
    /// Accounts expected:
    ///
    /// 0. `[writable]` LysergicTokenizer account
    /// 1. `[writable]` Underlying vault account
    /// 2. `[]` Underlying mint account
    /// 3. `[writable]` Principal token mint account
    /// 4. `[writable]` Yield token mint account
    /// 5. `[writable, signer]` User account
    /// 6. `[writable]` User underlying token account
    /// 7. `[writable]` User principal token account
    /// 8. `[writable]` User yield token account
    /// 9. `[]` Token program
    RedeemPrincipalAndYield { amount: u64 },

    /// Redeems the principal token for the underlying token
    ///
    /// Accounts expected:
    ///
    /// 0. `[writable]` LysergicTokenizer account
    /// 1. `[writable]` Underlying vault account
    /// 2. `[]` Underlying mint account
    /// 3. `[writable]` Principal token mint account
    /// 4. `[writable, signer]` User account
    /// 5. `[writable]` User underlying token account
    /// 7. `[writable]` User principal token account
    /// 8. `[]` Token program
    /// 9. `[]` Associated token account program
    RedeemPrincipalOnly {
        /// The amount of the principal token to redeem
        principal_amount: u64,
    },

    /// Burns the principal token
    ///
    /// Accounts expected:
    ///
    /// 0. `[writable]` LysergicTokenizer account
    /// 1. `[writable]` Principal token mint account
    /// 2. `[writable, signer]` User account
    /// 3. `[writable]` User principal token account
    /// 4. `[]` Token program
    BurnPrincipalToken {
        /// The amount of the principal token to burn
        amount: u64,
    },

    /// Claims the yield
    ///
    /// Accounts expected:
    ///
    /// 0. `[writable]` LysergicTokenizer account
    /// 1. `[writable]` Yield token mint account
    /// 2. `[writable]` Underlying vault account
    /// 3. `[writable, signer]` User account
    /// 4. `[writable]` User underlying token account
    /// 5. `[writable]` User yield token account
    /// 6. `[]` Token program
    /// 7. `[]` Associated token account program
    ClaimYield {
        /// The amount of the underlying token to claim
        yield_amount: u64,
    },

    /// Burns the yield token
    ///
    /// Accounts expected:
    ///
    /// 0. `[writable]` LysergicTokenizer account
    /// 1. `[writable]` Yield token mint account
    /// 2. `[writable, signer]` User account
    /// 3. `[writable]` User yield token account
    /// 4. `[]` Token program
    BurnYieldToken {
        /// The amount of the yield token to burn
        amount: u64,
    },
}

/// Creates an `InitializeLysergicTokenizer` instruction
pub fn init_lysergic_tokenizer(
    lysergic_tokenizer: &Pubkey,
    authority: &Pubkey,
    underlying_vault: &Pubkey,
    underlying_mint: &Pubkey,
    prinicpal_token_mint: &Pubkey,
    yield_token_mint: &Pubkey,
    expiry: Expiry,
) -> Result<Instruction, ProgramError> {
    Ok(Instruction::new_with_borsh(
        crate::id(),
        &LysergicTokenizerInstruction::InitializeLysergicTokenizer {
            underlying_vault: *underlying_vault,
            underlying_mint: *underlying_mint,
            principal_token_mint: *prinicpal_token_mint,
            yield_token_mint: *yield_token_mint,
            expiry,
        },
        vec![
            AccountMeta::new(*lysergic_tokenizer, false),
            AccountMeta::new(*authority, true),
            AccountMeta::new(*underlying_vault, false),
            AccountMeta::new_readonly(*underlying_mint, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    ))
}

/// Creates an `InitializeMints` instruction
pub fn init_mints(
    lysergic_tokenizer: &Pubkey,
    authority: &Pubkey,
    principal_token_mint: &Pubkey,
    yield_token_mint: &Pubkey,
    underlying_mint: &Pubkey,
    expiry: Expiry,
) -> Result<Instruction, ProgramError> {
    Ok(Instruction::new_with_borsh(
        crate::id(),
        &LysergicTokenizerInstruction::InitializeMints {
            underlying_mint: *underlying_mint,
            expiry,
        },
        vec![
            AccountMeta::new(*lysergic_tokenizer, false),
            AccountMeta::new(*authority, true),
            AccountMeta::new(*principal_token_mint, false),
            AccountMeta::new(*yield_token_mint, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    ))
}

/// Creates an `InitializeTokenizerAndMints` instruction
pub fn init_tokenizer_and_mints(
    lysergic_tokenizer: &Pubkey,
    authority: &Pubkey,
    underlying_vault: &Pubkey,
    underlying_mint: &Pubkey,
    principal_token_mint: &Pubkey,
    yield_token_mint: &Pubkey,
    expiry: Expiry,
) -> Result<Instruction, ProgramError> {
    Ok(Instruction::new_with_borsh(
        crate::id(),
        &LysergicTokenizerInstruction::InitializeTokenizerAndMints {
            underlying_vault: *underlying_vault,
            underlying_mint: *underlying_mint,
            principal_token_mint: *principal_token_mint,
            yield_token_mint: *yield_token_mint,
            expiry,
        },
        vec![
            AccountMeta::new(*lysergic_tokenizer, false),
            AccountMeta::new(*authority, true),
            AccountMeta::new(*underlying_vault, false),
            AccountMeta::new_readonly(*underlying_mint, false),
            AccountMeta::new(*principal_token_mint, false),
            AccountMeta::new(*yield_token_mint, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    ))
}

/// Creates a `DepositUnderlying` instruction
pub fn deposit_underlying(
    lysergic_tokenizer: &Pubkey,
    underlying_vault: &Pubkey,
    user: &Pubkey,
    user_underlying_token_account: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    Ok(Instruction::new_with_borsh(
        crate::id(),
        &LysergicTokenizerInstruction::DepositUnderlying { amount },
        vec![
            AccountMeta::new(*lysergic_tokenizer, false),
            AccountMeta::new(*underlying_vault, false),
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_underlying_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
    ))
}

/// Creates a `TokenizePrincipal` instruction
pub fn tokenize_principal(
    lysergic_tokenizer: &Pubkey,
    principal_token_mint: &Pubkey,
    user: &Pubkey,
    user_principal_token_account: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    Ok(Instruction::new_with_borsh(
        crate::id(),
        &LysergicTokenizerInstruction::TokenizePrincipal { amount },
        vec![
            AccountMeta::new(*lysergic_tokenizer, false),
            AccountMeta::new(*principal_token_mint, false),
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_principal_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
    ))
}

/// Creates a `TokenizeYield` instruction
pub fn tokenize_yield(
    lysergic_tokenizer: &Pubkey,
    yield_token_mint: &Pubkey,
    user: &Pubkey,
    user_yield_token_account: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    Ok(Instruction::new_with_borsh(
        crate::id(),
        &LysergicTokenizerInstruction::TokenizeYield { amount },
        vec![
            AccountMeta::new(*lysergic_tokenizer, false),
            AccountMeta::new(*yield_token_mint, false),
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_yield_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    ))
}

/// Creates a `DepositAndTokenize` instruction
pub fn deposit_and_tokenize(
    lysergic_tokenizer: &Pubkey,
    underlying_vault: &Pubkey,
    principal_token_mint: &Pubkey,
    yield_token_mint: &Pubkey,
    user: &Pubkey,
    user_underlying_token_account: &Pubkey,
    user_principal_token_account: &Pubkey,
    user_yield_token_account: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    Ok(Instruction::new_with_borsh(
        crate::id(),
        &LysergicTokenizerInstruction::DepositAndTokenize { amount },
        vec![
            AccountMeta::new(*lysergic_tokenizer, false),
            AccountMeta::new(*underlying_vault, false),
            AccountMeta::new(*principal_token_mint, false),
            AccountMeta::new(*yield_token_mint, false),
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_underlying_token_account, false),
            AccountMeta::new(*user_principal_token_account, false),
            AccountMeta::new(*user_yield_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
    ))
}

/// Creates a `RedeemPrincipalOnly` instruction
pub fn redeem_principal_only(
    lysergic_tokenizer: &Pubkey,
    underlying_vault: &Pubkey,
    underlying_mint: &Pubkey,
    principal_token_mint: &Pubkey,
    user: &Pubkey,
    user_underlying_token_account: &Pubkey,
    user_principal_token_account: &Pubkey,
    principal_amount: u64,
) -> Result<Instruction, ProgramError> {
    Ok(Instruction::new_with_borsh(
        crate::id(),
        &LysergicTokenizerInstruction::RedeemPrincipalOnly { principal_amount },
        vec![
            AccountMeta::new(*lysergic_tokenizer, false),
            AccountMeta::new(*underlying_vault, false),
            AccountMeta::new_readonly(*underlying_mint, false),
            AccountMeta::new(*principal_token_mint, false),
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_underlying_token_account, false),
            AccountMeta::new(*user_principal_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
    ))
}

/// Creates a `RedeemPrincipalAndYield` instruction
pub fn redeem_principal_and_yield(
    lysergic_tokenizer: &Pubkey,
    yield_token_mint: &Pubkey,
    underlying_vault: &Pubkey,
    underlying_mint: &Pubkey,
    principal_token_mint: &Pubkey,
    user: &Pubkey,
    user_underlying_token_account: &Pubkey,
    user_principal_token_account: &Pubkey,
    user_yield_token_account: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    Ok(Instruction::new_with_borsh(
        crate::id(),
        &LysergicTokenizerInstruction::RedeemPrincipalAndYield { amount },
        vec![
            AccountMeta::new(*lysergic_tokenizer, false),
            AccountMeta::new(*underlying_vault, false),
            AccountMeta::new_readonly(*underlying_mint, false),
            AccountMeta::new(*principal_token_mint, false),
            AccountMeta::new(*yield_token_mint, false),
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_underlying_token_account, false),
            AccountMeta::new(*user_principal_token_account, false),
            AccountMeta::new(*user_yield_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    ))
}

/// Creates a `ClaimYield` instruction
pub fn claim_yield(
    lysergic_tokenizer: &Pubkey,
    underlying_vault: &Pubkey,
    yield_token_mint: &Pubkey,
    user: &Pubkey,
    user_underlying_token_account: &Pubkey,
    user_yield_token_account: &Pubkey,
    yield_amount: u64,
) -> Result<Instruction, ProgramError> {
    Ok(Instruction::new_with_borsh(
        crate::id(),
        &LysergicTokenizerInstruction::ClaimYield { yield_amount },
        vec![
            AccountMeta::new(*lysergic_tokenizer, false),
            AccountMeta::new(*underlying_vault, false),
            AccountMeta::new(*yield_token_mint, false),
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_underlying_token_account, false),
            AccountMeta::new(*user_yield_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    ))
}

/// Creates a `BurnPrincipalToken` instruction
pub fn burn_principal_token(
    lysergic_tokenizer: &Pubkey,
    principal_token_mint: &Pubkey,
    user: &Pubkey,
    user_principal_token_account: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    Ok(Instruction::new_with_borsh(
        crate::id(),
        &LysergicTokenizerInstruction::BurnPrincipalToken { amount },
        vec![
            AccountMeta::new(*lysergic_tokenizer, false),
            AccountMeta::new(*principal_token_mint, false),
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_principal_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
    ))
}

/// Creates a `BurnYieldToken` instruction
pub fn burn_yield_token(
    lysergic_tokenizer: &Pubkey,
    yield_token_mint: &Pubkey,
    user: &Pubkey,
    user_yield_token_account: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    Ok(Instruction::new_with_borsh(
        crate::id(),
        &LysergicTokenizerInstruction::BurnYieldToken { amount },
        vec![
            AccountMeta::new(*lysergic_tokenizer, false),
            AccountMeta::new(*yield_token_mint, false),
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_yield_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    ))
}
