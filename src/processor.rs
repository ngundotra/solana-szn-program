//! Does something important
use::{
    arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs},
    solana_program::{
        entrypoint::{ProgramResult},
        account_info::{next_account_info, AccountInfo},
        pubkey::Pubkey,
        program_pack::{Pack},
        rent::Rent,
        msg,
        system_instruction,
        program::{invoke, invoke_signed},
        system_program,
        sysvar::{Sysvar},
    },
    std::{
        str::from_utf8,
        vec::Vec,
    },
};
// use crate::{
//     error::MsgError,
//     instruction::EmailInstruction,
// };

/// Directs how message instructions will be handled
pub struct Processor {
}
impl Processor {
    /// Handle the different instructions
    pub fn process_instruction<'a>(
        program_id: &'a Pubkey,
        accounts: &'a [AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        Ok(())
    }
}