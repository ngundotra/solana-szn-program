//! Does something important
use::{
    // arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs},
    solana_program::{
        entrypoint::{ProgramResult},
        account_info::{
            // next_account_info, 
            AccountInfo
        },
        pubkey::Pubkey,
        // program_pack::{Pack},
        // rent::Rent,
        // msg,
        // system_instruction,
        // program::{invoke, invoke_signed},
        // system_program,
        // sysvar::{Sysvar},
    },
    // std::{
    //     str::from_utf8,
    //     vec::Vec,
    // },
};
use crate::{
    // error::Sol2SolError,
    instruction::Sol2SolInstruction,
    state::{SolBox, SOL_BOX_NUM_SPOTS},
};

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
        let instruction = Sol2SolInstruction::unpack(instruction_data)?;
        Ok(match instruction {
            Sol2SolInstruction::InitializeSolBox {
                owner,
                num_spots,
                next_box,
                prev_box,
            } => {
                let _ = Self::process_init_sol_box(
                    program_id, 
                    accounts,
                    &owner,
                    num_spots,
                    &next_box,
                    &prev_box,
                );
            },
            Sol2SolInstruction::WriteMessage {
                sender,
                recipient,
                sol_box_id,
                msg_size,
                msg_string,
            } => {
                let _ = Self::process_write_message(
                    program_id, 
                    accounts,
                    &sender,
                    &recipient, 
                    &sol_box_id, 
                    msg_size,
                    &msg_string,
                );
            },
            Sol2SolInstruction::DeleteMessage {
                owner,
                message_id,
                sol_box_id,
            } => {
                let _ = Self::process_delete_message(
                    program_id, 
                    accounts,
                    &owner,
                    &message_id, 
                    &sol_box_id,
                );
            }
        })
    }

    fn process_init_sol_box<'a>(
        program_id: &'a Pubkey,
        accounts: &'a [AccountInfo],
        owner: &'a Pubkey,
        num_spots: u32,
        next_box: &'a Pubkey,
        prev_box: &'a Pubkey,
    ) -> ProgramResult {
        let message_slots: [Pubkey; SOL_BOX_NUM_SPOTS] = SolBox::get_empty_message_slots();
        let sol_box = SolBox {
            owner: *owner,
            next_box: *next_box,
            prev_box: *prev_box,
            num_spots,
            message_slots,
            is_initialized: true,
            num_in_use: 0,
        };
        Ok(())
    }

    fn process_write_message<'a>(
        program_id: &'a Pubkey,
        accounts: &'a [AccountInfo],
        sender: &'a Pubkey,
        recipient: &'a Pubkey,
        sol_box_id: &'a Pubkey,
        msg_size: u32,
        msg_string: &String,
    ) -> ProgramResult {
        Ok(())
    }

    fn process_delete_message<'a>(
        program_id: &'a Pubkey,
        accounts: &'a [AccountInfo],
        owner: &'a Pubkey,
        recipient: &'a Pubkey,
        sol_box_id: &'a Pubkey,
    ) -> ProgramResult {
        Ok(())
    }
}