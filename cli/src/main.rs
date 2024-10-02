use {
	anyhow::{anyhow, Result},
	clap::{command, Args, Parser, Subcommand},
	sclr_token::{
		get_principal_mint_address, get_tokenizer_address, get_yield_mint_address, instruction,
		Expiry,
	},
	solana_cli_config,
	solana_client::rpc_client::RpcClient,
	solana_program::{instruction::Instruction, pubkey::Pubkey},
	solana_sdk::{
		commitment_config::CommitmentConfig,
		signature::{read_keypair_file, Signer},
		transaction::Transaction,
	},
};

#[derive(Parser, Debug)]
struct Cli {
	#[arg(short, long)]
	config: Option<String>,
	#[arg(short, long)]
	rpc: Option<String>,
	#[arg(short, long)]
	payer: Option<String>,
	#[command(subcommand)]
	cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
	#[command(subcommand)]
	Init(Initialize),
	#[command(subcommand)]
	Tokenize(Tokenize),
	#[command(subcommand)]
	Redeem(Redeem),
	#[command(subcommand)]
	Terminate(Terminate),
	#[command(subcommand)]
	Swap(Swap),
}

#[derive(Subcommand, Debug)]
enum Initialize {
	Tokenizer(InitializeCommonFields),
	Mints(InitializeCommonFields),
	TokenizerMints(InitializeCommonFields),
	Amm(InitializeCommonFields),
}

#[derive(Subcommand, Debug)]
enum Tokenize {
	Deposit(InstructionCommonFields),
	Principal(InstructionCommonFields),
	Yield(InstructionCommonFields),
	PrincipalYield(InstructionCommonFields),
}

#[derive(Subcommand, Debug)]
enum Redeem {
	Principal(InstructionCommonFields),
	Yield(InstructionCommonFields),
	PrincipalYield(InstructionCommonFields),
}

#[derive(Subcommand, Debug)]
enum Terminate {
	Terminate(TerminateCommonFields),
	TerminateTokenizer(TerminateCommonFields),
	TerminateMints(TerminateCommonFields),
}

#[derive(Subcommand, Debug)]
enum Swap {
	Principal(InstructionCommonFields),
	Yield(InstructionCommonFields),
	Underlying(InstructionCommonFields),
}

#[derive(Args, Debug)]
struct InitializeCommonFields {
	underlying_mint_address: Pubkey,
	expiry: i64,
}

#[derive(Args, Debug)]
struct InstructionCommonFields {
	lysergic_tokenizer_address: Pubkey,
	amount: u64,
	underlying_mint_address: Pubkey,
}

#[derive(Args, Debug)]
struct TerminateCommonFields {
	lysergic_tokenizer_address: Pubkey,
	underlying_mint_address: Pubkey,
}

fn main() -> Result<()> {
	let args = Cli::parse();

	let solana_config_file = if let Some(ref config) = *solana_cli_config::CONFIG_FILE {
		solana_cli_config::Config::load(config).unwrap_or_default()
	} else {
		solana_cli_config::Config::default()
	};

	let wallet_keypair = read_keypair_file(&solana_config_file.keypair_path)
		.map_err(|err| anyhow!("Unable to read keypair file: {}", err))?;
	let wallet_pubkey = wallet_keypair.pubkey();

	let client = RpcClient::new_with_commitment(
		solana_config_file.json_rpc_url.to_string(),
		CommitmentConfig::confirmed(),
	);

	let slot = client.get_slot()?;
	let timestamp = client.get_block_time(slot)? as i64;

	let instruction: Instruction = match args.cmd {
		Commands::Init(init) => match init {
			Initialize::Tokenizer(common_fields) => {
				let expiry = Expiry::from_i64(common_fields.expiry).map_err(|err| {
					anyhow!("Unable to parse the given value to `Expiry`: {}", err)
				})?;

				let expiry_date = expiry.to_expiry_date(timestamp).expect("Invalid");

				let (lysergic_tokenizer_address, _) =
					get_tokenizer_address(&common_fields.underlying_mint_address, expiry_date);

				let underlying_vault_address =
					spl_associated_token_account::get_associated_token_address(
						&lysergic_tokenizer_address,
						&common_fields.underlying_mint_address,
					);

				let (principal_mint_address, _) =
					get_principal_mint_address(&lysergic_tokenizer_address);

				let (yield_mint_address, _) = get_yield_mint_address(&lysergic_tokenizer_address);

				//TODO: Calculation methodology for the fixed APY of the principal token
				//NOTE: placeholder
				let fixed_apy = 0;

				println!("Tokenizer Address: {}", lysergic_tokenizer_address);
				println!("Vault Address: {}", underlying_vault_address);
				println!("Principal Mint Address: {}", principal_mint_address);
				println!("Yield Mint Address: {}", yield_mint_address);

				instruction::init_tokenizer(
					&lysergic_tokenizer_address,
					&wallet_pubkey,
					&underlying_vault_address,
					&common_fields.underlying_mint_address,
					&principal_mint_address,
					&yield_mint_address,
					expiry,
					fixed_apy,
				)
				.map_err(|err| anyhow!("Unable to create init instruction: {}", err))?
			}
			Initialize::Mints(common_fields) => {
				let expiry = Expiry::from_i64(common_fields.expiry).map_err(|err| {
					anyhow!("Unable to parse the given value to `Expiry`: {}", err)
				})?;

				let expiry_date = expiry
					.to_expiry_date(timestamp)
					.expect("Unable to convert expiry to expiry date");

				let (lysergic_tokenizer_address, _) =
					get_tokenizer_address(&common_fields.underlying_mint_address, expiry_date);

				let (principal_mint_address, _) =
					get_principal_mint_address(&lysergic_tokenizer_address);

				let (yield_mint_address, _) = get_yield_mint_address(&lysergic_tokenizer_address);

				println!("Principal Mint Address: {}", principal_mint_address);
				println!("Yield Mint Address: {}", yield_mint_address);

				instruction::init_mints(
					&lysergic_tokenizer_address,
					&wallet_pubkey,
					&common_fields.underlying_mint_address,
					&principal_mint_address,
					&yield_mint_address,
					expiry,
				)
				.map_err(|err| anyhow!("Unable to create `Initialize` instruction: {}", err))?
			}
			Initialize::TokenizerMints(common_fields) => {
				let expiry = Expiry::from_i64(common_fields.expiry).map_err(|err| {
					anyhow!("Unable to parse the given value to `Expiry`: {}", err)
				})?;

				let expiry_date = expiry.to_expiry_date(timestamp).expect("Invalid");
				let (lysergic_tokenizer_address, _) =
					get_tokenizer_address(&common_fields.underlying_mint_address, expiry_date);

				let underlying_vault_address =
					spl_associated_token_account::get_associated_token_address(
						&lysergic_tokenizer_address,
						&common_fields.underlying_mint_address,
					);

				let (principal_mint_address, _) =
					get_principal_mint_address(&lysergic_tokenizer_address);

				let (yield_mint_address, _) = get_yield_mint_address(&lysergic_tokenizer_address);

				//TODO: Calculation methodology for the fixed APY of the principal token
				//NOTE: placeholder
				let fixed_apy = 0;

				println!("Tokenizer Address: {}", lysergic_tokenizer_address);
				println!("Vault Address: {}", underlying_vault_address);
				println!("Principal Mint Address: {}", principal_mint_address);
				println!("Yield Mint Address: {}", yield_mint_address);

				instruction::init_tokenizer_and_mints(
					&lysergic_tokenizer_address,
					&wallet_pubkey,
					&underlying_vault_address,
					&common_fields.underlying_mint_address,
					&principal_mint_address,
					&yield_mint_address,
					Expiry::from_i64(common_fields.expiry)?,
					fixed_apy,
				)
				.map_err(|err| {
					anyhow!(
						"Unable to create `InitializeTokenizerAndMints` instruction: {}",
						err
					)
				})?
			}
			Initialize::Amm(_common_fields) => unimplemented!(),
		},
		Commands::Tokenize(tokenize) => match tokenize {
			Tokenize::Deposit(common_fields) => {
				let underlying_vault = spl_associated_token_account::get_associated_token_address(
					&common_fields.lysergic_tokenizer_address,
					&common_fields.underlying_mint_address,
				);

				instruction::deposit_underlying(
					&common_fields.lysergic_tokenizer_address,
					&wallet_pubkey,
					&underlying_vault,
					&common_fields.underlying_mint_address,
					common_fields.amount,
				)
				.map_err(|err| anyhow!("Unable to create `Deposit` instruction: {}", err))?
			}
			Tokenize::Principal(common_fields) => {
				let (principal_mint_address, _) =
					get_principal_mint_address(&common_fields.lysergic_tokenizer_address);

				let user_principal_token_address =
					spl_associated_token_account::get_associated_token_address(
						&wallet_pubkey,
						&principal_mint_address,
					);

				instruction::tokenize_principal(
					&common_fields.lysergic_tokenizer_address,
					&principal_mint_address,
					&wallet_pubkey,
					&user_principal_token_address,
					common_fields.amount,
				)
				.map_err(|err| {
					anyhow!("Unable to create `TokenizePrincipal` instruction: {}", err)
				})?
			}
			Tokenize::Yield(common_fields) => {
				let (yield_mint_address, _) =
					get_yield_mint_address(&common_fields.lysergic_tokenizer_address);

				let user_yield_token_address =
					spl_associated_token_account::get_associated_token_address(
						&wallet_pubkey,
						&yield_mint_address,
					);

				instruction::tokenize_yield(
					&common_fields.lysergic_tokenizer_address,
					&yield_mint_address,
					&wallet_pubkey,
					&user_yield_token_address,
					common_fields.amount,
				)
				.map_err(|err| anyhow!("Unable to create `TokenizeYield` instruction: {}", err))?
			}
			Tokenize::PrincipalYield(common_fields) => {
				let underlying_vault = spl_associated_token_account::get_associated_token_address(
					&common_fields.lysergic_tokenizer_address,
					&common_fields.underlying_mint_address,
				);

				println!("Underlying vault: {}", underlying_vault);

				let (principal_mint_address, _) =
					get_principal_mint_address(&common_fields.lysergic_tokenizer_address);

				let (yield_mint_address, _) =
					get_yield_mint_address(&common_fields.lysergic_tokenizer_address);

				let user_underlying_token_address =
					spl_associated_token_account::get_associated_token_address(
						&wallet_pubkey,
						&common_fields.underlying_mint_address,
					);

				let user_principal_token_address =
					spl_associated_token_account::get_associated_token_address(
						&wallet_pubkey,
						&principal_mint_address,
					);

				let user_yield_token_address =
					spl_associated_token_account::get_associated_token_address(
						&wallet_pubkey,
						&yield_mint_address,
					);

				instruction::deposit_and_tokenize(
					&common_fields.lysergic_tokenizer_address,
					&underlying_vault,
					&principal_mint_address,
					&yield_mint_address,
					&wallet_pubkey,
					&user_underlying_token_address,
					&user_principal_token_address,
					&user_yield_token_address,
					common_fields.amount,
				)
				.map_err(|err| {
					anyhow!("Unable to create `DepositAndTokenize` instruction: {}", err)
				})?
			}
		},
		Commands::Redeem(redeem) => match redeem {
			Redeem::Principal(common_fields) => {
				let underlying_vault_address =
					spl_associated_token_account::get_associated_token_address(
						&common_fields.lysergic_tokenizer_address,
						&common_fields.underlying_mint_address,
					);

				let (principal_mint_address, _) =
					get_principal_mint_address(&common_fields.lysergic_tokenizer_address);

				let user_underlying_token_address =
					spl_associated_token_account::get_associated_token_address(
						&wallet_pubkey,
						&common_fields.underlying_mint_address,
					);

				let user_principal_token_address =
					spl_associated_token_account::get_associated_token_address(
						&wallet_pubkey,
						&principal_mint_address,
					);

				instruction::redeem_mature_principal(
					&common_fields.lysergic_tokenizer_address,
					&underlying_vault_address,
					&common_fields.underlying_mint_address,
					&principal_mint_address,
					&wallet_pubkey,
					&user_underlying_token_address,
					&user_principal_token_address,
					common_fields.amount,
				)
				.map_err(|err| {
					anyhow!(
						"Unable to create `RedeemPrincipalOnly` instruction: {}",
						err
					)
				})?
			}
			Redeem::Yield(common_fields) => {
				let (yield_mint_address, _) =
					get_yield_mint_address(&common_fields.lysergic_tokenizer_address);

				let user_underlying_token_address =
					spl_associated_token_account::get_associated_token_address(
						&wallet_pubkey,
						&common_fields.underlying_mint_address,
					);

				let user_yield_token_address =
					spl_associated_token_account::get_associated_token_address(
						&wallet_pubkey,
						&yield_mint_address,
					);

				instruction::claim_yield(
					&common_fields.lysergic_tokenizer_address,
					&common_fields.underlying_mint_address,
					&yield_mint_address,
					&wallet_pubkey,
					&user_underlying_token_address,
					&user_yield_token_address,
					common_fields.amount,
				)
				.map_err(|err| anyhow!("Unable to create `ClaimYield` instruction: {}", err))?
			}
			Redeem::PrincipalYield(common_fields) => {
				let underlying_vault_address =
					spl_associated_token_account::get_associated_token_address(
						&common_fields.lysergic_tokenizer_address,
						&common_fields.underlying_mint_address,
					);

				let (principal_mint_address, _) =
					get_principal_mint_address(&common_fields.lysergic_tokenizer_address);

				let (yield_mint_address, _) =
					get_yield_mint_address(&common_fields.lysergic_tokenizer_address);

				let user_underlying_token_address =
					spl_associated_token_account::get_associated_token_address(
						&wallet_pubkey,
						&common_fields.underlying_mint_address,
					);

				let user_principal_token_address =
					spl_associated_token_account::get_associated_token_address(
						&wallet_pubkey,
						&principal_mint_address,
					);

				let user_yield_token_address =
					spl_associated_token_account::get_associated_token_address(
						&wallet_pubkey,
						&yield_mint_address,
					);

				instruction::redeem_principal_and_yield(
					&common_fields.lysergic_tokenizer_address,
					&underlying_vault_address,
					&common_fields.underlying_mint_address,
					&principal_mint_address,
					&yield_mint_address,
					&wallet_pubkey,
					&user_underlying_token_address,
					&user_principal_token_address,
					&user_yield_token_address,
					common_fields.amount,
				)
				.map_err(|err| {
					anyhow!(
						"Unable to create `RedeemPrincipalAndYield` instruction: {}",
						err
					)
				})?
			}
		},
		Commands::Terminate(terminate) => match terminate {
			Terminate::Terminate(common_fields) => {
				let (principal_mint_address, _) =
					get_principal_mint_address(&common_fields.lysergic_tokenizer_address);
				let (yield_mint_address, _) =
					get_yield_mint_address(&common_fields.lysergic_tokenizer_address);
				instruction::terminate(
					&common_fields.lysergic_tokenizer_address,
					&wallet_pubkey,
					&spl_associated_token_account::get_associated_token_address(
						&common_fields.lysergic_tokenizer_address,
						&common_fields.underlying_mint_address,
					),
					&spl_associated_token_account::get_associated_token_address(
						&common_fields.lysergic_tokenizer_address,
						&principal_mint_address,
					),
					&spl_associated_token_account::get_associated_token_address(
						&common_fields.lysergic_tokenizer_address,
						&yield_mint_address,
					),
				)
				.map_err(|err| anyhow!("Unable to create `Terminate` instruction: {}", err))?
			}
			Terminate::TerminateTokenizer(common_fields) => instruction::terminate_tokenizer(
				&common_fields.lysergic_tokenizer_address,
				&wallet_pubkey,
				&spl_associated_token_account::get_associated_token_address(
					&common_fields.lysergic_tokenizer_address,
					&common_fields.underlying_mint_address,
				),
			)
			.map_err(|err| anyhow!("Unable to create `TerminateTokenizer` instruction: {}", err))?,
			Terminate::TerminateMints(common_fields) => {
				let (principal_mint_address, _) =
					get_principal_mint_address(&common_fields.lysergic_tokenizer_address);
				let (yield_mint_address, _) =
					get_yield_mint_address(&common_fields.lysergic_tokenizer_address);
				instruction::terminate_mints(
					&common_fields.lysergic_tokenizer_address,
					&wallet_pubkey,
					&principal_mint_address,
					&yield_mint_address,
				)
				.map_err(|err| anyhow!("Unable to create `TerminateMints` instruction: {}", err))?
			}
		},
		Commands::Swap(swap) => match swap {
			Swap::Principal(_common_fields) => unimplemented!(),
			Swap::Yield(_common_fields) => unimplemented!(),
			Swap::Underlying(_common_fields) => unimplemented!(),
		},
	};

	let mut transaction = Transaction::new_with_payer(&[instruction], Some(&wallet_pubkey));
	let latest_blockchash = client
		.get_latest_blockhash()
		.map_err(|err| anyhow!("Unable to get latest blockhash: {}", err))?;

	transaction.sign(&[&wallet_keypair], latest_blockchash);
	client
		.send_and_confirm_transaction_with_spinner(&transaction)
		.map_err(|err| anyhow!("Unable to send transaction: {}", err))?;

	Ok(())
}
