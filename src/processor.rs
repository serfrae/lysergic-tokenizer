use {
    crate::{
        error::LysergicTokenizerError,
        get_principal_mint_address, get_tokenizer_address, get_yield_mint_address,
        instruction::LysergicTokenizerInstruction,
        state::{LysergicTokenizerState, LYSERGIC_TOKENIZER_STATE_SIZE},
        Expiry,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_instruction, system_program,
        sysvar::{rent, Sysvar},
    },
};

pub struct LysergicTokenizerProcessor;

impl LysergicTokenizerProcessor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        if program_id != &crate::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let instruction: LysergicTokenizerInstruction =
            LysergicTokenizerInstruction::try_from_slice(data)
                .map_err(|_| ProgramError::InvalidInstructionData)?;

        match instruction {
            LysergicTokenizerInstruction::InitializeLysergicTokenizer {
                underlying_vault,
                underlying_mint,
                principal_token_mint,
                yield_token_mint,
                expiry,
            } => Self::process_initialize_lysergic_tokenizer(
                accounts,
                underlying_vault,
                underlying_mint,
                principal_token_mint,
                yield_token_mint,
                &expiry,
            ),
            LysergicTokenizerInstruction::InitializeMints {
                underlying_mint,
                expiry,
            } => Self::process_initialize_mints(accounts, underlying_mint, &expiry),
            LysergicTokenizerInstruction::InitializeTokenizerAndMints {
                underlying_vault,
                underlying_mint,
                principal_token_mint,
                yield_token_mint,
                expiry,
            } => Self::process_initialize_tokenizer_and_mints(
                accounts,
                underlying_vault,
                underlying_mint,
                principal_token_mint,
                yield_token_mint,
                expiry,
            ),
            LysergicTokenizerInstruction::DepositUnderlying { amount } => {
                Self::process_deposit_underlying(accounts, amount)
            }
            LysergicTokenizerInstruction::TokenizePrincipal { amount } => {
                Self::process_tokenize_principal(accounts, amount)
            }
            LysergicTokenizerInstruction::TokenizeYield { amount } => {
                Self::process_tokenize_yield(accounts, amount)
            }
            LysergicTokenizerInstruction::DepositAndTokenize { amount } => {
                Self::process_deposit_and_tokenize(accounts, amount)
            }
            LysergicTokenizerInstruction::RedeemPrincipalAndYield { amount } => {
                Self::process_redeem_principal_and_yield(accounts, amount)
            }
            LysergicTokenizerInstruction::RedeemPrincipalOnly { principal_amount } => {
                Self::process_redeem_principal_only(accounts, principal_amount)
            }
            LysergicTokenizerInstruction::BurnPrincipalToken { amount } => {
                Self::process_burn_principal_token(accounts, amount)
            }
            LysergicTokenizerInstruction::ClaimYield { yield_amount } => {
                Self::process_claim_yield(accounts, yield_amount)
            }
            LysergicTokenizerInstruction::BurnYieldToken { amount } => {
                Self::process_burn_yield_token(accounts, amount)
            }
        }
    }

    fn process_initialize_lysergic_tokenizer(
        accounts: &[AccountInfo],
        principal_token_mint: Pubkey,
        yield_token_mint: Pubkey,
        underlying_mint: Pubkey,
        underlying_vault: Pubkey,
        expiry: &Expiry,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lysergic_tokenizer_account = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
        let underlying_vault_account = next_account_info(account_info_iter)?;
        let underlying_mint_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        let rent = rent::Rent::get()?;

        let expiry_date = match expiry.to_expiry_date() {
            Some(expiry_date) => expiry_date,
            None => return Err(LysergicTokenizerError::InvalidExpiryDate.into()),
        };

        // Check if lysergic tokenizer account address is correct
        if lysergic_tokenizer_account.key
            != &crate::get_tokenizer_address(authority.key, &underlying_mint, expiry_date)
        {
            return Err(LysergicTokenizerError::IncorrectTokenizerAddress.into());
        }

        if !authority.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check if the underlying vault account address is correct
        if underlying_vault_account.key
            != &spl_associated_token_account::get_associated_token_address(
                lysergic_tokenizer_account.key,
                &underlying_mint,
            )
        {
            return Err(LysergicTokenizerError::IncorrectVaultAddress.into());
        }

        // Check the underlying mint account
        if &underlying_mint != underlying_mint_account.key {
            return Err(LysergicTokenizerError::IncorrectUnderlyingMintAddress.into());
        }

        // Check principal token mint address
        if principal_token_mint
            != crate::get_principal_mint_address(lysergic_tokenizer_account.key, &underlying_mint)
        {
            return Err(LysergicTokenizerError::IncorrectPrincipalMintAddress.into());
        }

        // Check yield token mint address
        if yield_token_mint
            != crate::get_yield_mint_address(lysergic_tokenizer_account.key, &underlying_mint)
        {
            return Err(LysergicTokenizerError::IncorrectYieldMintAddress.into());
        }

        // Check token program
        if token_program.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Check system program
        if system_program.key != &system_program::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // The chance of address collision are negligible so we just check if the account is
        // owned by the program_id
        //
        // Check if the lysergic tokenizer account is already initialized
        if lysergic_tokenizer_account.owner != &crate::id() {
            let size = LYSERGIC_TOKENIZER_STATE_SIZE;
            let required_lamports = rent
                .minimum_balance(size)
                .max(1)
                .saturating_sub(lysergic_tokenizer_account.lamports());

            // Create lysergic tokenizer account
            invoke(
                &system_instruction::create_account(
                    authority.key,
                    lysergic_tokenizer_account.key,
                    required_lamports,
                    size as u64,
                    &crate::id(),
                ),
                &[
                    authority.clone(),
                    lysergic_tokenizer_account.clone(),
                    system_program.clone(),
                ],
            )?;

            // The chances of collision are low so we shouldn't need to check if the account is
            // initialized - will throw an error if it is.
            //
            // Create underlying vault account
            invoke(
                &spl_associated_token_account::instruction::create_associated_token_account(
                    authority.key,
                    underlying_vault_account.key,
                    &underlying_mint,
                    token_program.key,
                ),
                &[
                    authority.clone(),
                    underlying_vault_account.clone(),
                    lysergic_tokenizer_account.clone(),
                    underlying_mint_account.clone(),
                    system_program.clone(),
                    token_program.clone(),
                ],
            )?;

            let lysergic_tokenizer_state = LysergicTokenizerState {
                principal_token_mint,
                yield_token_mint,
                underlying_mint,
                underlying_vault,
                expiry_date,
            };

            lysergic_tokenizer_state
                .serialize(&mut &mut lysergic_tokenizer_account.data.borrow_mut()[..])?;

            Ok(())
        } else {
            return Err(LysergicTokenizerError::TokenizerAlreadyInitialized.into());
        }
    }

    fn process_initialize_mints(
        accounts: &[AccountInfo],
        underlying_mint: Pubkey,
        expiry: &Expiry,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lysergic_tokenizer_account = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
        let underlying_mint_account = next_account_info(account_info_iter)?;
        let principal_token_mint_account = next_account_info(account_info_iter)?;
        let yield_token_mint_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        let expiry_date = match expiry.to_expiry_date() {
            Some(expiry_date) => expiry_date,
            None => return Err(LysergicTokenizerError::InvalidExpiryDate.into()),
        };

        // General safety checks
        if lysergic_tokenizer_account.key
            != &get_tokenizer_address(token_program.key, &underlying_mint, expiry_date)
        {
            return Err(LysergicTokenizerError::IncorrectTokenizerAddress.into());
        }

        if !authority.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if token_program.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Run safety checks if the lysergic tokenizer account is initialized
        if lysergic_tokenizer_account.owner == &crate::id() {
            let lysergic_tokenizer_state = LysergicTokenizerState::try_from_slice(
                &lysergic_tokenizer_account.data.borrow()[..LYSERGIC_TOKENIZER_STATE_SIZE],
            )?;

            if &lysergic_tokenizer_state.principal_token_mint != principal_token_mint_account.key {
                return Err(LysergicTokenizerError::IncorrectPrincipalMintAddress.into());
            }

            if &lysergic_tokenizer_state.yield_token_mint != yield_token_mint_account.key {
                return Err(LysergicTokenizerError::IncorrectYieldMintAddress.into());
            }

            if &lysergic_tokenizer_state.underlying_mint != underlying_mint_account.key {
                return Err(LysergicTokenizerError::IncorrectUnderlyingMintAddress.into());
            }

            if lysergic_tokenizer_state.expiry_date != expiry_date {
                return Err(LysergicTokenizerError::InvalidExpiryDate.into());
            }

            if lysergic_tokenizer_state.underlying_vault
                != spl_associated_token_account::get_associated_token_address(
                    lysergic_tokenizer_account.key,
                    &lysergic_tokenizer_state.underlying_mint,
                )
            {
                return Err(LysergicTokenizerError::IncorrectVaultAddress.into());
            }
        } else if lysergic_tokenizer_account.owner != &crate::id() {
            // Safety checks if the lysergic tokenizer account is not initialized
            if principal_token_mint_account.key
                != &get_principal_mint_address(token_program.key, lysergic_tokenizer_account.key)
            {
                return Err(LysergicTokenizerError::IncorrectPrincipalMintAddress.into());
            }
            if yield_token_mint_account.key
                != &get_yield_mint_address(token_program.key, lysergic_tokenizer_account.key)
            {
                return Err(LysergicTokenizerError::IncorrectYieldMintAddress.into());
            }
        }

        // Initialize principal token mint
        invoke(
            &spl_token::instruction::initialize_mint(
                token_program.key,
                principal_token_mint_account.key,
                lysergic_tokenizer_account.key,
                None,
                6,
            )?,
            &[principal_token_mint_account.clone(), token_program.clone()],
        )?;

        // Initialize yield token mint
        invoke(
            &spl_token::instruction::initialize_mint(
                token_program.key,
                yield_token_mint_account.key,
                lysergic_tokenizer_account.key,
                None,
                6,
            )?,
            &[yield_token_mint_account.clone(), token_program.clone()],
        )?;

        Ok(())
    }

    fn process_initialize_tokenizer_and_mints(
        accounts: &[AccountInfo],
        underlying_vault: Pubkey,
        underlying_mint: Pubkey,
        principal_token_mint: Pubkey,
        yield_token_mint: Pubkey,
        expiry: Expiry,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lysergic_tokenizer_account = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
        let underlying_vault_account = next_account_info(account_info_iter)?;
        let underlying_mint_account = next_account_info(account_info_iter)?;
        let principal_token_mint_account = next_account_info(account_info_iter)?;
        let yield_token_mint_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let associated_token_account_program = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        let initialize_tokenizer_accounts = vec![
            lysergic_tokenizer_account.clone(),
            authority.clone(),
            underlying_vault_account.clone(),
            token_program.clone(),
            associated_token_account_program.clone(),
            system_program.clone(),
        ];

        let initialize_mint_accounts = vec![
            lysergic_tokenizer_account.clone(),
            underlying_mint_account.clone(),
            principal_token_mint_account.clone(),
            yield_token_mint_account.clone(),
            token_program.clone(),
        ];

        Self::process_initialize_lysergic_tokenizer(
            &initialize_tokenizer_accounts,
            principal_token_mint,
            yield_token_mint,
            underlying_mint,
            underlying_vault,
            &expiry,
        )?;

        Self::process_initialize_mints(&initialize_mint_accounts, underlying_mint, &expiry)?;

        Ok(())
    }

    fn process_deposit_underlying(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lysergic_tokenizer_account = next_account_info(account_info_iter)?;
        let underlying_vault_account = next_account_info(account_info_iter)?;
        let user_account = next_account_info(account_info_iter)?;
        let user_underlying_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        let lysergic_tokenizer_state = LysergicTokenizerState::try_from_slice(
            &lysergic_tokenizer_account.data.borrow()[..LYSERGIC_TOKENIZER_STATE_SIZE],
        )?;

        // Safety checks
        if lysergic_tokenizer_account.owner != &crate::id() {
            return Err(LysergicTokenizerError::TokenizerNotInitialized.into());
        }

        if underlying_vault_account.key != &lysergic_tokenizer_state.underlying_vault {
            return Err(LysergicTokenizerError::IncorrectVaultAddress.into());
        }

        if !user_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if user_underlying_token_account.key
            != &spl_associated_token_account::get_associated_token_address(
                user_account.key,
                &lysergic_tokenizer_state.underlying_mint,
            )
        {
            return Err(LysergicTokenizerError::IncorrectAccountAddress.into());
        }

        if token_program.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Transfer underlying token from user to lysergic tokenizer
        invoke(
            &spl_token::instruction::transfer(
                token_program.key,
                user_underlying_token_account.key,
                underlying_vault_account.key,
                user_account.key,
                &[],
                amount,
            )?,
            &[
                user_underlying_token_account.clone(),
                underlying_vault_account.clone(),
                user_account.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_tokenize_principal(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lysergic_tokenizer_account = next_account_info(account_info_iter)?;
        let principal_token_mint_account = next_account_info(account_info_iter)?;
        let user_account = next_account_info(account_info_iter)?;
        let user_principal_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        if lysergic_tokenizer_account.owner != &crate::id() {
            return Err(LysergicTokenizerError::TokenizerNotInitialized.into());
        }

        let lysergic_tokenizer_state = LysergicTokenizerState::try_from_slice(
            &lysergic_tokenizer_account.data.borrow()[..LYSERGIC_TOKENIZER_STATE_SIZE],
        )?;

        if principal_token_mint_account.key != &lysergic_tokenizer_state.principal_token_mint {
            return Err(LysergicTokenizerError::IncorrectPrincipalMintAddress.into());
        }

        if user_principal_token_account.key
            != &spl_associated_token_account::get_associated_token_address(
                user_account.key,
                &lysergic_tokenizer_state.principal_token_mint,
            )
        {
            return Err(LysergicTokenizerError::IncorrectAccountAddress.into());
        }

        if token_program.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Mint principal token to user
        invoke_signed(
            &spl_token::instruction::mint_to(
                token_program.key,
                principal_token_mint_account.key,
                user_principal_token_account.key,
                lysergic_tokenizer_account.key,
                &[],
                amount,
            )?,
            &[
                principal_token_mint_account.clone(),
                user_principal_token_account.clone(),
                lysergic_tokenizer_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"lysergic-tokenizer"[..], &[0u8]]],
        )?;

        Ok(())
    }

    fn process_tokenize_yield(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lysergic_tokenizer_account = next_account_info(account_info_iter)?;
        let yield_token_mint_account = next_account_info(account_info_iter)?;
        let user_account = next_account_info(account_info_iter)?;
        let user_yield_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        if lysergic_tokenizer_account.owner != &crate::id() {
            return Err(LysergicTokenizerError::TokenizerNotInitialized.into());
        }

        let lysergic_tokenizer_state = LysergicTokenizerState::try_from_slice(
            &lysergic_tokenizer_account.data.borrow()[..LYSERGIC_TOKENIZER_STATE_SIZE],
        )?;

        if yield_token_mint_account.key != &lysergic_tokenizer_state.yield_token_mint {
            return Err(LysergicTokenizerError::IncorrectYieldMintAddress.into());
        }

        if user_yield_token_account.key
            != &spl_associated_token_account::get_associated_token_address(
                user_account.key,
                &lysergic_tokenizer_state.yield_token_mint,
            )
        {
            return Err(LysergicTokenizerError::IncorrectAccountAddress.into());
        }

        if token_program.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Mint yield token to user
        invoke_signed(
            &spl_token::instruction::mint_to(
                token_program.key,
                yield_token_mint_account.key,
                user_yield_token_account.key,
                lysergic_tokenizer_account.key,
                &[],
                amount,
            )?,
            &[
                yield_token_mint_account.clone(),
                user_yield_token_account.clone(),
                lysergic_tokenizer_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"lysergic-tokenizer"[..], &[0u8]]],
        )?;

        Ok(())
    }

    fn process_deposit_and_tokenize(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lysergic_tokenizer_account = next_account_info(account_info_iter)?;
        let underlying_vault_account = next_account_info(account_info_iter)?;
        let principal_token_mint_account = next_account_info(account_info_iter)?;
        let yield_token_mint_account = next_account_info(account_info_iter)?;
        let user_account = next_account_info(account_info_iter)?;
        let user_underlying_token_account = next_account_info(account_info_iter)?;
        let user_principal_token_account = next_account_info(account_info_iter)?;
        let user_yield_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        let deposit_accounts = vec![
            lysergic_tokenizer_account.clone(),
            underlying_vault_account.clone(),
            user_account.clone(),
            user_underlying_token_account.clone(),
            token_program.clone(),
        ];

        let tokenize_prinicpal_accounts = vec![
            lysergic_tokenizer_account.clone(),
            principal_token_mint_account.clone(),
            user_account.clone(),
            user_principal_token_account.clone(),
            token_program.clone(),
        ];

        let tokenize_yield_accounts = vec![
            lysergic_tokenizer_account.clone(),
            yield_token_mint_account.clone(),
            user_account.clone(),
            user_yield_token_account.clone(),
            token_program.clone(),
        ];

        Self::process_deposit_underlying(&deposit_accounts, amount)?;
        Self::process_tokenize_principal(&tokenize_prinicpal_accounts, amount)?;
        Self::process_tokenize_yield(&tokenize_yield_accounts, amount)?;

        Ok(())
    }

    fn process_redeem_principal_and_yield(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lysergic_tokenizer_account = next_account_info(account_info_iter)?;
        let underlying_vault_account = next_account_info(account_info_iter)?;
        let underlying_mint_account = next_account_info(account_info_iter)?;
        let principal_token_mint_account = next_account_info(account_info_iter)?;
        let yield_token_mint_account = next_account_info(account_info_iter)?;
        let user_account = next_account_info(account_info_iter)?;
        let user_underlying_token_account = next_account_info(account_info_iter)?;
        let user_principal_token_account = next_account_info(account_info_iter)?;
        let user_yield_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        let redeem_principal_accounts = vec![
            lysergic_tokenizer_account.clone(),
            underlying_vault_account.clone(),
            underlying_mint_account.clone(),
            principal_token_mint_account.clone(),
            yield_token_mint_account.clone(),
            user_account.clone(),
            user_principal_token_account.clone(),
            user_yield_token_account.clone(),
            token_program.clone(),
        ];

        let claim_yield_accounts = vec![
            lysergic_tokenizer_account.clone(),
            underlying_vault_account.clone(),
            yield_token_mint_account.clone(),
            user_account.clone(),
            user_underlying_token_account.clone(),
            user_yield_token_account.clone(),
            token_program.clone(),
        ];

        let burn_yield_token_accounts = vec![
            lysergic_tokenizer_account.clone(),
            yield_token_mint_account.clone(),
            user_account.clone(),
            user_yield_token_account.clone(),
            token_program.clone(),
        ];

        Self::process_redeem_principal_only(&redeem_principal_accounts, amount)?;
        Self::process_claim_yield(&claim_yield_accounts, amount)?;
        Self::process_burn_yield_token(&burn_yield_token_accounts, amount)?;

        Ok(())
    }

    fn process_redeem_principal_only(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lysergic_tokenizer_account = next_account_info(account_info_iter)?;
        let underlying_vault_account = next_account_info(account_info_iter)?;
        let underlying_mint_account = next_account_info(account_info_iter)?;
        let principal_token_mint_account = next_account_info(account_info_iter)?;
        let user_account = next_account_info(account_info_iter)?;
        let user_underlying_token_account = next_account_info(account_info_iter)?;
        let user_principal_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        if lysergic_tokenizer_account.owner != &crate::id() {
            return Err(LysergicTokenizerError::TokenizerNotInitialized.into());
        }

        let lysergic_tokenizer_state = LysergicTokenizerState::try_from_slice(
            &lysergic_tokenizer_account.data.borrow()[..LYSERGIC_TOKENIZER_STATE_SIZE],
        )?;

        if underlying_vault_account.key != &lysergic_tokenizer_state.underlying_vault {
            return Err(LysergicTokenizerError::IncorrectVaultAddress.into());
        }

        if underlying_mint_account.key != &lysergic_tokenizer_state.underlying_mint {
            return Err(LysergicTokenizerError::IncorrectUnderlyingMintAddress.into());
        }

        if principal_token_mint_account.key != &lysergic_tokenizer_state.principal_token_mint {
            return Err(LysergicTokenizerError::IncorrectPrincipalMintAddress.into());
        }

        if user_underlying_token_account.key
            != &spl_associated_token_account::get_associated_token_address(
                user_account.key,
                &lysergic_tokenizer_state.underlying_mint,
            )
        {
            return Err(LysergicTokenizerError::IncorrectAccountAddress.into());
        }

        if user_principal_token_account.key
            != &spl_associated_token_account::get_associated_token_address(
                user_account.key,
                &lysergic_tokenizer_state.principal_token_mint,
            )
        {
            return Err(LysergicTokenizerError::IncorrectAccountAddress.into());
        }

        if token_program.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                underlying_vault_account.key,
                user_principal_token_account.key,
                lysergic_tokenizer_account.key,
                &[],
                amount,
            )?,
            &[
                underlying_vault_account.clone(),
                underlying_mint_account.clone(),
                user_underlying_token_account.clone(),
                lysergic_tokenizer_account.clone(),
            ],
            &[&[&b"lysergic-tokenizer"[..], &[0u8]]],
        )?;

        Self::process_burn_principal_token(&accounts, amount)?;
        Ok(())
    }

    fn process_burn_principal_token(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lysergic_tokenizer_account = next_account_info(account_info_iter)?;
        let principal_token_mint_account = next_account_info(account_info_iter)?;
        let user_account = next_account_info(account_info_iter)?;
        let user_principal_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        if lysergic_tokenizer_account.owner != &crate::id() {
            return Err(LysergicTokenizerError::TokenizerNotInitialized.into());
        }

        let lysergic_tokenizer_state = LysergicTokenizerState::try_from_slice(
            &lysergic_tokenizer_account.data.borrow()[..LYSERGIC_TOKENIZER_STATE_SIZE],
        )?;

        if principal_token_mint_account.key != &lysergic_tokenizer_state.principal_token_mint {
            return Err(LysergicTokenizerError::IncorrectPrincipalMintAddress.into());
        }

        if !user_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if user_principal_token_account.key
            != &spl_associated_token_account::get_associated_token_address(
                user_account.key,
                &lysergic_tokenizer_state.principal_token_mint,
            )
        {
            return Err(LysergicTokenizerError::IncorrectAccountAddress.into());
        }

        if token_program.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        invoke(
            &spl_token::instruction::burn(
                token_program.key,
                user_principal_token_account.key,
                principal_token_mint_account.key,
                user_account.key,
                &[],
                amount,
            )?,
            &[
                user_principal_token_account.clone(),
                principal_token_mint_account.clone(),
                user_account.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_claim_yield(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lysergic_tokenizer_account = next_account_info(account_info_iter)?;
        let underlying_vault_account = next_account_info(account_info_iter)?;
        let yield_token_mint_account = next_account_info(account_info_iter)?;
        let user_account = next_account_info(account_info_iter)?;
        let user_underlying_token_account = next_account_info(account_info_iter)?;
        let user_yield_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        if lysergic_tokenizer_account.owner != &crate::id() {
            return Err(LysergicTokenizerError::TokenizerNotInitialized.into());
        }

        let lysergic_tokenizer_state = LysergicTokenizerState::try_from_slice(
            &lysergic_tokenizer_account.data.borrow()[..LYSERGIC_TOKENIZER_STATE_SIZE],
        )?;

        if underlying_vault_account.key != &lysergic_tokenizer_state.underlying_vault {
            return Err(LysergicTokenizerError::IncorrectVaultAddress.into());
        }

        if yield_token_mint_account.key != &lysergic_tokenizer_state.yield_token_mint {
            return Err(LysergicTokenizerError::IncorrectYieldMintAddress.into());
        }

        if user_underlying_token_account.key
            != &spl_associated_token_account::get_associated_token_address(
                user_account.key,
                &lysergic_tokenizer_state.underlying_mint,
            )
        {
            return Err(LysergicTokenizerError::IncorrectAccountAddress.into());
        }

        if user_yield_token_account.key
            != &spl_associated_token_account::get_associated_token_address(
                user_account.key,
                &lysergic_tokenizer_state.yield_token_mint,
            )
        {
            return Err(LysergicTokenizerError::IncorrectAccountAddress.into());
        }

        if token_program.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                underlying_vault_account.key,
                user_yield_token_account.key,
                lysergic_tokenizer_account.key,
                &[],
                amount,
            )?,
            &[
                underlying_vault_account.clone(),
                yield_token_mint_account.clone(),
                user_underlying_token_account.clone(),
                lysergic_tokenizer_account.clone(),
            ],
            &[&[&b"lysergic-tokenizer"[..], &[0u8]]],
        )?;

        Ok(())
    }

    fn process_burn_yield_token(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lysergic_tokenizer_account = next_account_info(account_info_iter)?;
        let yield_token_mint_account = next_account_info(account_info_iter)?;
        let user_account = next_account_info(account_info_iter)?;
        let user_yield_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        if lysergic_tokenizer_account.owner != &crate::id() {
            return Err(LysergicTokenizerError::TokenizerNotInitialized.into());
        }

        let lysergic_tokenizer_state = LysergicTokenizerState::try_from_slice(
            &lysergic_tokenizer_account.data.borrow()[..LYSERGIC_TOKENIZER_STATE_SIZE],
        )?;

        if yield_token_mint_account.key != &lysergic_tokenizer_state.yield_token_mint {
            return Err(LysergicTokenizerError::IncorrectYieldMintAddress.into());
        }

        if !user_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if user_yield_token_account.key
            != &spl_associated_token_account::get_associated_token_address(
                user_account.key,
                &lysergic_tokenizer_state.yield_token_mint,
            )
        {
            return Err(LysergicTokenizerError::IncorrectAccountAddress.into());
        }

        if token_program.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        invoke(
            &spl_token::instruction::burn(
                token_program.key,
                user_yield_token_account.key,
                yield_token_mint_account.key,
                user_account.key,
                &[],
                amount,
            )?,
            &[
                user_yield_token_account.clone(),
                yield_token_mint_account.clone(),
                user_account.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }
}
