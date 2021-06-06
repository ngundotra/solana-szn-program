
#![cfg(feature = "test-bpf")]

use {
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
        state::SolBox,
    },
    std::{convert::TryInto, str::FromStr},
};

#[cfg(test)]
// pub mod tests {
    #[tokio::test]
    async fn test_init_sol_box() {
        let program_id = Pubkey::from_str(&"invoker111111111111111111111111111111111111").unwrap();
        let mut program_test = ProgramTest::new(
            &"sol2sol",
            program_id,
            processor!(Processor::process_instruction),
        );
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let sol_box_pair = Keypair::new();
        let idk = Keypair::new();
        let wtf = Keypair::new();

        let rent = banks_client.get_rent().await.unwrap();
        let create_account_ix = system_instruction::create_account(
            &payer.pubkey(),
            &sol_box_pair.pubkey(),
            rent.minimum_balance(SolBox::get_packed_len()) + 2,
            SolBox::get_packed_len().try_into().unwrap(),
            &payer.pubkey(),
        );
        // assert_eq!(create_account_ix.)
        // let pre_existing_acct = banks_client.get_account(sol_box_pair.pubkey())
        //     .await
        //     .expect("get_account")
        //     .expect("associated_account not none");
        // println!("Found account!");

        let mut transaction = Transaction::new_with_payer(
            // &[Instruction::new_with_bincode(
            //     system_program::id(),
            //     &create_account_ix,
            //     vec![
            //         AccountMeta::new(system_program::id(), false),
            //         // Account::new(lamports: 10000, space:0, owner:)
            //         AccountMeta::new(payer.pubkey(), true),
            //         AccountMeta::new(idk.pubkey(), true),
            //         // AccountMeta::new(sysvar::rent::id(), false),
            //         // AccountMeta::new(nft_factory_pair.pubkey(), true),
            //     ],
            // )],
            &[create_account_ix],
            Some(&payer.pubkey()),
        );
        // // transaction.sign(&[&payer, &nft_factory_pair, &nft_holder_pair], recent_blockhash);
        transaction.sign(&[&payer, &sol_box_pair], recent_blockhash);
        // banks_client.process_transaction(transaction).await.unwrap();
    }
// }