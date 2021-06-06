//! Instructions for email
use {
    arrayref::{
        array_ref,
    },
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_error::{ProgramError},
        pubkey::Pubkey,
        // program_option::COption,
        sysvar,
    },
    std::{
        result::Result,
        // convert::TryInto,
        mem::size_of,
        vec::Vec,
        // str,
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
        /// Which box to send it to
        sol_box_id: Pubkey,
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
                let (sol_box_id, rest) = Self::unpack_pubkey(rest)?;
                let (msg_size, rest) = Self::unpack_size(rest)?;
                let (msg_string, _rest) = Self::unpack_msg(rest, msg_size as usize)?;
                Self::WriteMessage {
                    sender,
                    recipient,
                    sol_box_id,
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

    /// Packs instruction into a vec buffer
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
                sol_box_id,
                msg_size,
                msg_string,
            } => {
                buf.push(1);
                buf.extend_from_slice(&sender.to_bytes());
                buf.extend_from_slice(&recipient.to_bytes());
                buf.extend_from_slice(&sol_box_id.to_bytes());
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

/// Creates an InitializeSolBox instruction
pub fn init_sol_box(
    program_id: &Pubkey,
    payer_pubkey: &Pubkey,
    sol_box_pubkey: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let data: Vec<u8> = Sol2SolInstruction::InitializeSolBox {
        owner: *payer_pubkey,
        num_spots: 20 as u32,
        next_box: *sol_box_pubkey,
        prev_box: *sol_box_pubkey
    }.pack();

    let accounts = vec![
        // AccountMeta::new(*program_id, false),
        AccountMeta::new(*sol_box_pubkey, true),
        AccountMeta::new(*payer_pubkey, true),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data
    })
}

// /// Creates an InitializeSolBox instruction
// pub fn init_sol_box(
//     program_id: &Pubkey,
//     payer_pubkey: &Pubkey,
//     sol_box_pubkey: &Pubkey,
// ) -> Result<Instruction, ProgramError> {
//     let data: Vec<u8> = Sol2SolInstruction::InitializeSolBox {
//         owner: *payer_pubkey,
//         num_spots: 20 as u32,
//         next_box: *sol_box_pubkey,
//         prev_box: *sol_box_pubkey
//     }.pack();

//     let accounts = vec![
//         // AccountMeta::new(*program_id, false),
//         AccountMeta::new(*sol_box_pubkey, true),
//         AccountMeta::new(*payer_pubkey, true),
//         AccountMeta::new_readonly(sysvar::rent::id(), false),
//     ];

//     Ok(Instruction {
//         program_id: *program_id,
//         accounts,
//         data
//     })
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_parsing() {
        let msg_string: String = "hello world!".to_owned();     // 12
        let init_msg_string = msg_string.clone();
        let msg_size: u32 = msg_string.len() as u32;            // 4
        let sender = Pubkey::new_unique();                      // 32
        let recipient = Pubkey::new_unique();                   // 32
        let sol_box_id = Pubkey::new_unique();                  // 32
        // 12 + 4 + 32 + 32 + 1 (tag) = 113
        let instruction = Sol2SolInstruction::WriteMessage {
            sender,
            recipient,
            sol_box_id,
            msg_size,
            msg_string,
        };
        let packed_vec = instruction.pack();
        assert_eq!(101 + msg_size as usize, packed_vec.len());
        assert_eq!(113, packed_vec.len());
        
        let recreated = Sol2SolInstruction::unpack(&packed_vec[..]).unwrap();
        assert_eq!(instruction, recreated);
        match instruction {
            Sol2SolInstruction::WriteMessage{ msg_string, .. } => {
                assert_eq!(init_msg_string, msg_string);
            }
            _ => {
                // Lol manually fail test
                assert_eq!(0, 1);
            }
        }
    }

    // #[test]
    // fn state_deserialize_invalid() {
    //     assert_eq!(
    //         FeatureProposalInstruction::unpack_from_slice(&[1]),
    //         Ok(FeatureProposalInstruction::Tally),
    //     );

    //     // Extra bytes (0xff) ignored...
    //     assert_eq!(
    //         FeatureProposalInstruction::unpack_from_slice(&[1, 0xff, 0xff, 0xff]),
    //         Ok(FeatureProposalInstruction::Tally),
    //     );

    //     assert_eq!(
    //         FeatureProposalInstruction::unpack_from_slice(&[2]),
    //         Err(ProgramError::InvalidInstructionData),
    //     );
    // }
}
