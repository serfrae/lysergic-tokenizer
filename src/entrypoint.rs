use {
	crate::{error::TokenizerError, processor::TokenizerProcessor},
	solana_program::{
		account_info::AccountInfo, entrypoint::ProgramResult, program_error::PrintProgramError,
		pubkey::Pubkey,
	},
};

solana_program::entrypoint!(process_instruction);
pub fn process_instruction(
	program_id: &Pubkey,
	accounts: &[AccountInfo],
	instruction_data: &[u8],
) -> ProgramResult {
	if let Err(e) = TokenizerProcessor::process(program_id, accounts, instruction_data) {
		e.print::<TokenizerError>();
		return Err(e);
	}

	Ok(())
}
