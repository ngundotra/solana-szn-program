//! Does something important
use::{
    // arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs},
    solana_program::{  
        entrypoint::{ProgramResult},
        program_error::{ProgramError},
        account_info::{
            next_account_info, 
            AccountInfo,
        },
        pubkey::Pubkey,
        program_pack::{Pack},
        rent::Rent,
        // msg,
        // system_instruction,
        // program::{invoke, invoke_signed},
        // system_program,
        sysvar::{Sysvar},
        msg,
    },
    // std::{
    //     str::from_utf8,
    //     vec::Vec,
    // },
};
use crate::{
    // error::Sol2SolError,
    instruction::Sol2SolInstruction,
    state::{
        SolBox, 
        // Message,
        pack_message_into,
        SOL_BOX_NUM_SPOTS
    },
    error::Sol2SolError,
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
        match instruction {
            Sol2SolInstruction::InitializeSolBox {
                owner,
                num_spots,
                next_box,
                prev_box,
            } => {
                Self::process_init_sol_box(
                    program_id, 
                    accounts,
                    &owner,
                    num_spots,
                    &next_box,
                    &prev_box,
                )
            },
            Sol2SolInstruction::WriteMessage {
                sender,
                recipient,
                message_pubkey,
                sol_box_pubkey,
                msg_size,
                msg_string,
            } => {
                Self::process_write_message(
                    program_id, 
                    accounts,
                    &sender,
                    &recipient, 
                    &message_pubkey,
                    &sol_box_pubkey, 
                    msg_size,
                    &msg_string,
                )
            },
            Sol2SolInstruction::DeleteMessage {
                owner,
                message_id,
                sol_box_id,
            } => {
                Self::process_delete_message(
                    program_id, 
                    accounts,
                    &owner,
                    &message_id, 
                    &sol_box_id,
                )
            }
        }
    }

    fn process_init_sol_box<'a>(
        program_id: &'a Pubkey,
        accounts: &'a [AccountInfo],
        owner: &'a Pubkey,
        num_spots: u32,
        next_box: &'a Pubkey,
        prev_box: &'a Pubkey,
    ) -> ProgramResult {

        // <------Accounts Check------
        let account_info_iter = &mut accounts.iter();
        let sol_box_info = next_account_info(account_info_iter)?;
        let payer_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        let sol_box_data_len = sol_box_info.data_len();

        // Check that this was created properly
        msg!("Checking system owner");
        if sol_box_info.owner != program_id {
            return Err(Sol2SolError::OwnerMismatch.into());
        }
        // Check that account data is zero'd
        // msg!("Checking data is uninitialized");
        // let sol_box = SolBox::unpack_unchecked(&sol_box_info.data.borrow())?;
        // if sol_box.is_initialized {
        //     return Err(Sol2SolError::SolBoxAlreadyInUse.into());
        // }
        // Check that payer will be user-space owner
        msg!("Checking user space owner");
        if owner != payer_info.key {
            return Err(Sol2SolError::OwnerMismatch.into());
        }
        // Check that the solbox is rent-exempt
        if !rent.is_exempt(sol_box_info.lamports(), sol_box_data_len) {
            return Err(Sol2SolError::InsufficientFunds.into());
        }
        // -------End Account Check----->

        // <---------Init Sol Box-------
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

        SolBox::pack(sol_box, &mut sol_box_info.data.borrow_mut())?;
        // -------End Init Sol Box------>

        Ok(())
    }

    fn process_write_message<'a>(
        program_id: &'a Pubkey,
        accounts: &'a [AccountInfo],
        sender: &'a Pubkey,
        recipient: &'a Pubkey,
        message_pubkey: &'a Pubkey,
        sol_box_pubkey: &'a Pubkey,
        msg_size: u32,
        msg_string: &String,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let message_account_info = next_account_info(account_info_iter)?;
        let sol_box_info = next_account_info(account_info_iter)?;
        let payer_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        msg!("Checking owner of message field matches");
        if message_account_info.owner != program_id {
            return Err(Sol2SolError::OwnerMismatch.into());
        }
        msg!("Checking message account id matches");
        if message_account_info.key != message_pubkey {
            return Err(ProgramError::InvalidInstructionData);
        }
        msg!("Checking owner of sol box field matches program id");
        if sol_box_info.owner != program_id {
            return Err(Sol2SolError::OwnerMismatch.into());
        }
        msg!("Checking owner of sol box field matches payer");
        let mut sol_box = SolBox::unpack_unchecked(&sol_box_info.data.borrow())?;
        if sol_box.owner != *payer_info.key {
            return Err(Sol2SolError::OwnerMismatch.into());
        }

        msg!("Writing to message state");
        pack_message_into(msg_size, msg_string, &mut message_account_info.data.borrow_mut());

        msg!("Writing to sol box");
        SolBox::add_message_to_sol_box(&mut sol_box.message_slots, &message_account_info.key)?;
        SolBox::pack(sol_box, &mut sol_box_info.data.borrow_mut())?;

        msg!("Writing message succeeded!");
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

