//! Instructions for email
use {
    arrayref::{
        array_refs,
        array_ref,
    },
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_error::{ProgramError},
        pubkey::Pubkey,
        program_option::COption,
        sysvar,
    },
    std::{
        result::Result,
        convert::TryInto,
        mem::size_of,
        vec::Vec,
        str,
    }
};
use crate::{error::Sol2SolError};

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
/// Instructions supported by the email program.
pub enum Sol2SolInstruction {
    /// InitializeSolBox
    InitializeSolBox {
        /// owner of the sol box
        owner: Pubkey,
        /// How many messages will this box store
        num_spots: u32,
        /// Address of next box for extra entries
        next_box: Pubkey,
        /// Address of prev box, prevBox == self if root
        prev_box: Pubkey,
    },
    /// Send email from one wallet address to another
    WriteMessage {
        /// Which address is sending the email
        sender: Pubkey,
        /// Which address is the email for
        recipient: Pubkey,
        /// How large is the email
        msg_size: u32,
        /// What is the utf-8 data of the email
        msg_string: String,
    },
    /// Delete message & reclaim lamports
    DeleteMessage {
        /// To ensure we are deleting the right message 
        owner: Pubkey,
        /// Account of message to be deleted
        message_id: Pubkey,
        /// Where the message is located
        sol_box_id: Pubkey,
    }
}
impl Sol2SolInstruction {
    /// Unpack the given bytes into an email
    pub fn unpack<'a>(input: &'a [u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_at(1);
        Ok(match tag[0] {
            0 => {
                let (owner, rest) = Self::unpack_pubkey(rest)?;
                let (num_spots, rest) = Self::unpack_size(rest)?;
                let (next_box, rest) = Self::unpack_pubkey(rest)?;
                let (prev_box, _rest) = Self::unpack_pubkey(rest)?;
                Self::InitializeSolBox {
                    owner,
                    num_spots,
                    next_box,
                    prev_box
                }
            }
            1 => {
                let (sender, rest) = Self::unpack_pubkey(rest)?;
                let (recipient, rest) = Self::unpack_pubkey(rest)?;
                let (msg_size, rest) = Self::unpack_size(rest)?;
                let (msg_string, _rest) = Self::unpack_msg(rest, msg_size as usize)?;
                Self::WriteMessage {
                    sender,
                    recipient,
                    msg_size,
                    msg_string
                }
            }
            2 => {
                let (owner, rest) = Self::unpack_pubkey(rest)?;
                let (message_id, rest) = Self::unpack_pubkey(rest)?;
                let (sol_box_id, _rest) = Self::unpack_pubkey(rest)?;
                Self::DeleteMessage {
                    owner,
                    message_id,
                    sol_box_id,
                }
            }
            _ => return Err(Sol2SolError::InvalidInstructionData.into()),
        })
    }

    fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
        if input.len() >= 32 {
            let (key, rest) = input.split_at(32);
            let pk = Pubkey::new(key);
            Ok((pk, rest))
        } else {
            Err(Sol2SolError::InvalidInstructionData.into())
        }
    }

    fn unpack_size(input: &[u8]) -> Result<(u32, &[u8]), ProgramError> {
        if input.len() >= 4 {
            let (_, rest) = input.split_at(4);
            let bytes = array_ref![input, 0, 4];
            let size = u32::from_le_bytes(*bytes);
            Ok((size, rest))
        } else {
            Err(Sol2SolError::InvalidInstructionData.into())
        }
    }
    
    fn unpack_msg<'a>(input: &'a [u8], msg_size: usize) -> Result<(String, &'a [u8]), ProgramError> {
        if input.len() >= msg_size {
            let mut utf8_bytes = Vec::new();
            for i in 0..msg_size {
                utf8_bytes.push(input[i]);
            }
            let msg_decoded = String::from_utf8(utf8_bytes);
            match msg_decoded {
                Ok(msg_data) => {
                    let (_, rest) = input.split_at(msg_size);
                    Ok((msg_data, rest))
                },
                Err(_) => {
                    Err(Sol2SolError::InvalidAccountData.into())
                }
            }
        } else {
            Err(Sol2SolError::InvalidInstructionData.into())
        }
    }

    /// Packs a [EmailInstruction](enum.EmailInstruction.html) into a vec buffer
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::InitializeSolBox {
                owner,
                num_spots,
                next_box,
                prev_box,
            } => {
                buf.push(0);
                buf.extend_from_slice(&owner.to_bytes());
                buf.extend_from_slice(&num_spots.to_le_bytes());
                buf.extend_from_slice(&next_box.to_bytes());
                buf.extend_from_slice(&prev_box.to_bytes());
            }
            Self::WriteMessage {
                sender,
                recipient,
                msg_size,
                msg_string,
            } => {
                buf.push(1);
                buf.extend_from_slice(&sender.to_bytes());
                buf.extend_from_slice(&recipient.to_bytes());
                buf.extend_from_slice(&msg_size.to_le_bytes());
                Self::pack_msg(&msg_string, &mut buf);
            }
            Self::DeleteMessage {
                owner,
                message_id,
                sol_box_id,
            } => {
                buf.push(2);
                buf.extend_from_slice(&owner.to_bytes());
                buf.extend_from_slice(&message_id.to_bytes());
                buf.extend_from_slice(&sol_box_id.to_bytes());
            }
        };
        buf
    }

    fn pack_msg(msg: &String, buf: &mut Vec<u8>) {
        let msg_str: &[u8] = msg.as_bytes();
        // println!("\tMsg str is: {} and has size {}", msg, msg_str.len());
        buf.extend_from_slice(msg_str);
    }
}
