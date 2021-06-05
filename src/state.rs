//! State for messaging
//! Todo(ngundotra): design a whitelist system + transferring lamports

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
// use num_enum::TryFromPrimitive;
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use std::{convert::TryFrom, str::FromStr};
use crate::error::Sol2SolError;

const NULL_PUBKEY_STR: &'static str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
/// Hard coded until post-hackathon
pub const SOL_BOX_NUM_SPOTS: usize = 20;

// Todo(ngundotra): remove assumption that box size is 20
/// SolBox
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SolBox {
    /// Who owns this SolBox
    pub owner: Pubkey,
    /// Where to go looking for more messages
    pub next_box: Pubkey,
    /// Where to go looking for more messages
    pub prev_box: Pubkey,
    /// How many messages this box stores
    pub num_spots: u32,
    /// How many messages have been used
    pub num_in_use: u32,
    /// Has been initialized?
    pub is_initialized: bool,
    /// The message pubkeys (const # only 40)
    pub message_slots: [Pubkey; SOL_BOX_NUM_SPOTS],
}
impl SolBox {
    /// Convenience function to initialize the message slots
    pub fn get_empty_message_slots() -> [Pubkey; SOL_BOX_NUM_SPOTS] {
        let null_pubkey: Pubkey = Pubkey::from_str(NULL_PUBKEY_STR).unwrap();
        return [null_pubkey; SOL_BOX_NUM_SPOTS];
    }

    fn pack_keys_into_ref(message_slots: &[Pubkey; SOL_BOX_NUM_SPOTS], message_slots_dst: &mut [u8]) {
        // Pack the keys into an array
        let key_bytes: &mut [u8; SOL_BOX_NUM_SPOTS*32] = &mut [0; SOL_BOX_NUM_SPOTS*32];
        for i in 0..SOL_BOX_NUM_SPOTS {
            let bytes = message_slots[i].to_bytes();
            for j in 0..32 {
                key_bytes[i*32+j] = bytes[j];
            }
        }
        message_slots_dst.copy_from_slice(key_bytes.as_ref());
    }
}
impl Sealed for SolBox {}
impl IsInitialized for SolBox {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Pack for SolBox {
    const LEN: usize = 745; //20*32+32+32+32+4+4+1;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, SolBox::LEN];
        let (owner, next_box, prev_box, num_spots, num_in_use, is_initialized, message_slots_src) =
            array_refs![src, 32, 32, 32, 4, 4, 1, SOL_BOX_NUM_SPOTS*32];

        let owner = Pubkey::new(owner);
        let next_box = Pubkey::new(next_box);
        let prev_box = Pubkey::new(prev_box);

        let num_spots = u32::from_le_bytes(*num_spots);
        if usize::try_from(num_spots).unwrap() != SOL_BOX_NUM_SPOTS {
            return Err(ProgramError::InvalidAccountData)
        }

        let num_in_use = u32::from_le_bytes(*num_in_use);

        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData)
        };

        let null_pubkey = Pubkey::from_str(NULL_PUBKEY_STR).unwrap();
        let message_slots: &mut [Pubkey; SOL_BOX_NUM_SPOTS] = &mut [null_pubkey; SOL_BOX_NUM_SPOTS];
        let mut i = 0;
        for chunk in message_slots_src.chunks(32) {
            let message_pubkey = Pubkey::new(chunk);
            message_slots[i] = message_pubkey;
            i += 1;
        }

        Ok(Self {
            owner,
            next_box,
            prev_box,
            num_spots,
            num_in_use,
            is_initialized,
            message_slots: *message_slots
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, SolBox::LEN];
        let (
            owner_dst,
            next_box_dst,
            prev_box_dst,
            num_spots_dst,
            num_in_use_dst,
            is_initialized_dst,
            message_slots_dst,
        ) = mut_array_refs![dst, 32, 32, 32, 4, 4, 1, (SOL_BOX_NUM_SPOTS as usize)*32];
        let &SolBox {
            ref owner,
            ref next_box,
            ref prev_box,
            num_spots,
            num_in_use,
            is_initialized,
            ref message_slots,
        } = self;
        owner_dst.copy_from_slice(owner.as_ref());
        next_box_dst.copy_from_slice(next_box.as_ref());
        prev_box_dst.copy_from_slice(prev_box.as_ref());
        *num_spots_dst = num_spots.to_le_bytes();
        *num_in_use_dst = num_in_use.to_le_bytes();
        is_initialized_dst[0] = is_initialized as u8;

        Self::pack_keys_into_ref(&message_slots, message_slots_dst);
    }
}



#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_sol_box_state() {
        println!("Penis!");
        let owner = Pubkey::new_unique();
        let num_spots = 20;
        let num_in_use = 3;
        let is_initialized = true;
        let next_box = Pubkey::new_unique();
        let prev_box = Pubkey::new_unique();
        let address1 = Pubkey::new_unique();
        let address2 = Pubkey::new_unique();
        let address3 = Pubkey::new_unique();
        let null_pubkey = Pubkey::from_str(NULL_PUBKEY_STR).unwrap();
        let message_slots = [
            address1, address2, address3, null_pubkey, null_pubkey,
            null_pubkey, null_pubkey, null_pubkey, null_pubkey, null_pubkey,
            null_pubkey, null_pubkey, null_pubkey, null_pubkey, null_pubkey,
            null_pubkey, null_pubkey, null_pubkey, null_pubkey, null_pubkey,
        ];
        let init_box = SolBox {
            owner,
            next_box,
            prev_box,
            num_in_use,
            num_spots,
            is_initialized,
            message_slots
        };
        
        let dst: &mut [u8; SolBox::LEN] = &mut [0; SolBox::LEN];
        init_box.pack_into_slice(dst);

        let recreated_box = SolBox::unpack_from_slice(dst).unwrap();
        assert_eq!(init_box, recreated_box);
    }
}