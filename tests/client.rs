
#![cfg(feature = "test-bpf")]

use {
    arrayref::{array_ref, array_refs},
    solana_program_test::*,
    solana_sdk::{ 
        account::Account, 
        signature::Signer,
        transaction::Transaction, 
        signer::keypair::Keypair,
    },
    solana_program::{
        system_program,
        pubkey::Pubkey,
        rent::Rent,
        sysvar,
        system_instruction,
        program_pack::{Pack},
        instruction::{AccountMeta, Instruction}
    },
    sol2sol::{
        processor::Processor,
        instruction::Sol2SolInstruction,
        state::SolBox,
    },
    std::{convert::TryInto, str::FromStr},
};

#[cfg(test)]
// pub mod tests {
    #[tokio::test]
    async fn test_init_sol_box() {
        let program_id = Pubkey::from_str(&"invoker111111111111111111111111111111111111").unwrap();
        let sol_box_pair = Keypair::new();

        let mut program_test = ProgramTest::new(
            &"sol2sol",
            program_id,
            processor!(Processor::process_instruction),
        );
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        let rent = banks_client.get_rent().await.unwrap();
        let create_account_ix = system_instruction::create_account(
            &payer.pubkey(),
            &sol_box_pair.pubkey(),
            rent.minimum_balance(SolBox::get_packed_len()) + 2,
            SolBox::get_packed_len().try_into().unwrap(),
            &program_id,
        );

        let init_sol_box_ix = Sol2SolInstruction::InitializeSolBox {
            owner: payer.pubkey(),
            num_spots: 20 as u32,
            next_box: sol_box_pair.pubkey(),
            prev_box: sol_box_pair.pubkey(),
        }.pack();

        let mut transaction = Transaction::new_with_payer(
            &[
                create_account_ix,
                Instruction {
                    program_id,
                    data: init_sol_box_ix,
                    accounts: vec![
                        AccountMeta::new(program_id, false),
                        AccountMeta::new(sol_box_pair.pubkey(), true),
                        AccountMeta::new(payer.pubkey(), true),
                        AccountMeta::new(sysvar::rent::id(), false),
                    ],
                },
            ],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &sol_box_pair], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
        let sol_box_acct = banks_client.get_account(sol_box_pair.pubkey())
            .await
            .expect("get_account")
            .expect("associated_account not none");
        
        assert_eq!(sol_box_acct.data.len(), SolBox::get_packed_len());
        let null_messages = SolBox::get_empty_message_slots();
        let sol_box_state = SolBox {
            owner: payer.pubkey(),
            next_box: sol_box_pair.pubkey(),
            prev_box: sol_box_pair.pubkey(),
            num_spots: 20 as u32,
            num_in_use: 0 as u32,
            is_initialized: true,
            message_slots: null_messages,
        };

        let data_src = array_ref![&sol_box_acct.data[..], 0, 96];
        let (owner_src, next_box_src, prev_box_src) 
            = array_refs![data_src, 32, 32, 32];

        let owner = Pubkey::new(owner_src);
        assert_eq!(payer.pubkey(), owner);

        let next_box = Pubkey::new(next_box_src);
        assert_eq!(sol_box_pair.pubkey(), next_box);

        let prev_box = Pubkey::new(prev_box_src);
        assert_eq!(sol_box_pair.pubkey(), prev_box);

        let recreated_data = SolBox::unpack_from_slice(&sol_box_acct.data[..]).unwrap();
        assert_eq!(sol_box_state, recreated_data);
    }
// }