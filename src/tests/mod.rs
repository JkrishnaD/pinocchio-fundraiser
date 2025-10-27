#[cfg(test)]
mod tests {
    use std::{io::Error, path::PathBuf, vec};

    use bytemuck::bytes_of;
    use litesvm::LiteSVM;
    use litesvm_token::{
        CreateAssociatedTokenAccount, CreateMint, MintTo,
        spl_token::{
            self,
            solana_program::{msg, rent::Rent, sysvar::SysvarId},
            state::Account,
        },
    };
    use pinocchio::program_error::ProgramError;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use spl_associated_token_account::solana_program::program_pack::Pack;

    use crate::instructions::{ContributeInstructions, InitializeInstructionData};

    const PROGRAM_ID: &str = "CG1q69YqagtgKi4G22pNM3WPYeqs1MEBe79qAZGU4FNc";
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

    fn program_id() -> Pubkey {
        Pubkey::from(crate::ID)
    }

    pub struct SetupState {
        pub maker: Keypair,
        pub user: Keypair,
        pub maker_ata: Pubkey,
        pub mint_to_raise: Pubkey,
        pub fundraiser: (Pubkey, u8),
        pub contributor: (Pubkey, u8),
        pub contributor_ata: Pubkey,
        pub vault: Pubkey,
        pub associated_token_program: Pubkey,
        pub token_program: Pubkey,
        pub system_program: Pubkey,
    }

    fn setup() -> (LiteSVM, SetupState) {
        let mut svm = LiteSVM::new();
        let maker = Keypair::new();
        let user = Keypair::new();

        svm.airdrop(&maker.pubkey(), 5 * LAMPORTS_PER_SOL)
            .expect("Airdrop Failed");

        svm.airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop Failed");

        let so_path = PathBuf::from(
            "/Users/jayakrishna/Documents/turbine/acc_builders/fundraiser-pinocchio/target/deploy/fundraiser_pinocchio.so",
        );

        let program_data = std::fs::read(so_path).expect("Failed to read the path");

        svm.add_program(program_id(), &program_data);

        let mint_to_raise = CreateMint::new(&mut svm, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        // msg!("Mint To Raise: {}", mint_to_raise);

        let fundraiser = Pubkey::find_program_address(
            &[b"fundraiser", maker.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );
        // msg!("Fundraiser PDA: {}\n", fundraiser.0);

        let contributor = Pubkey::find_program_address(
            &[
                b"contributor",
                fundraiser.0.as_ref(),
                user.pubkey().as_ref(),
            ],
            &PROGRAM_ID.parse().unwrap(),
        );
        // msg!("Contributor PDA: {}\n", contributor.0);

        let maker_ata = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_to_raise)
            .owner(&maker.pubkey())
            .send()
            .unwrap();
        // msg!("Maker ata: {}\n", maker_ata);

        let contributor_ata = CreateAssociatedTokenAccount::new(&mut svm, &user, &mint_to_raise)
            .owner(&user.pubkey())
            .send()
            .unwrap();
        // msg!("Contributor Ata: {}\n", contributor_ata);

        MintTo::new(
            &mut svm,
            &maker,
            &mint_to_raise,
            &contributor_ata,
            60_000_000,
        )
        .send()
        .unwrap();
        // msg!("Minted 60_000_000 tokens to contributor_ata\n");

        let vault = spl_associated_token_account::get_associated_token_address(
            &fundraiser.0,  // owner will be the escrow PDA
            &mint_to_raise, // mint
        );
        // msg!("Vault PDA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;

        let state = SetupState {
            maker,
            user,
            maker_ata,
            mint_to_raise,
            fundraiser,
            contributor,
            contributor_ata,
            vault,
            associated_token_program,
            token_program,
            system_program,
        };

        (svm, state)
    }

    // #[test]
    // pub fn test_init_fundraiser() {
    //     let (mut svm, state) = setup();

    //     let program_id = program_id();
    //     init_fundraiser(&mut svm, &state).unwrap();

    //     let fundraiser_state = svm.get_account(&state.fundraiser.0).unwrap();
    //     let fundraiser =
    //         bytemuck::try_from_bytes::<crate::state::Fundraiser>(&fundraiser_state.data).unwrap();

    //     let amount: u64 = 100_000_000;
    //     let current_amount: u64 = 0;
    //     assert_eq!(fundraiser.amount_to_raise, amount.to_le_bytes());
    //     assert_eq!(fundraiser.current_amount, current_amount.to_le_bytes());
    // }

    // #[test]
    // pub fn test_user_contribute() {
    //     let (mut svm, state) = setup();

    //     let program_id = program_id();
    //     init_fundraiser(&mut svm, &state).unwrap();
    //     contribute(&mut svm, &state).unwrap(); // user 1 contributes

    //     let fundraiser_state = svm.get_account(&state.fundraiser.0).unwrap();
    //     let fundraiser =
    //         bytemuck::try_from_bytes::<crate::state::Fundraiser>(&fundraiser_state.data).unwrap();

    //     let amount_to_raise: u64 = 100_000_000;
    //     let expected_current: u64 = 10_000_000;
    //     assert_eq!(fundraiser.amount_to_raise, amount_to_raise.to_le_bytes());
    //     assert_eq!(fundraiser.current_amount, expected_current.to_le_bytes());

    //     let vault_state = svm.get_account(&state.vault).unwrap();
    //     let vault = Account::unpack(&vault_state.data).unwrap();

    //     // msg!("The Vault Account: {:?}", vault);
    //     // msg!("The fundraiser Account: {:?}", fundraiser);
    //     // msg!("new vault balance: {:?}", vault.amount);
    // }

    #[test]
    pub fn test_checker() {
        let (mut svm, state) = setup();

        let program_id = program_id();
        init_fundraiser(&mut svm, &state).unwrap();
        contribute(&mut svm, &state).unwrap();
        check_funds(&mut svm, &state).unwrap();
    }

    pub fn check_funds(svm: &mut LiteSVM, state: &SetupState) -> Result<(), ProgramError> {
        let maker = &state.maker;
        let mint_to_raise = state.mint_to_raise;
        let fundraiser = state.fundraiser;
        let vault = state.vault;
        let maker_ata = state.maker_ata;
        let token_program = state.token_program;
        let system_program = state.system_program;
        let associated_token_program = state.associated_token_program;

        let program_id = program_id();

        let checker_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new(mint_to_raise, false),
                AccountMeta::new(fundraiser.0, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(maker_ata, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(associated_token_program, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: vec![2u8],
        };

        let message = Message::new(&[checker_ix], Some(&maker.pubkey()));

        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[maker], message, recent_blockhash);

        let tx = svm.send_transaction(transaction).unwrap();

        msg!("CU's Consumed by Checker: {}", tx.compute_units_consumed);
        Ok(())
    }

    pub fn contribute(svm: &mut LiteSVM, state: &SetupState) -> Result<(), ProgramError> {
        let maker = &state.maker;
        let user = &state.user;
        let contributor = state.contributor;
        let contributor_ata = state.contributor_ata;
        let mint_to_raise = state.mint_to_raise;
        let fundraiser = state.fundraiser;
        let vault = state.vault;
        let token_program = state.token_program;
        let system_program = state.system_program;
        let associated_token_program = state.associated_token_program;

        let program_id = program_id();

        let amount: u64 = 10_000_000;
        let contribute_data_struct: ContributeInstructions = ContributeInstructions {
            amount: amount.to_le_bytes(),
        };

        // Serialize into bytes
        let contri_bytes = bytes_of(&contribute_data_struct).to_vec();
        let contribute_ix_data = [
            vec![1u8], // Discriminator for "Initialize" instruction
            contri_bytes,
        ]
        .concat();

        // msg!("Contribute");
        let init_tx = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(user.pubkey(), true),
                AccountMeta::new(mint_to_raise, false),
                AccountMeta::new(fundraiser.0, false),
                AccountMeta::new(contributor.0, false),
                AccountMeta::new(contributor_ata, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(associated_token_program, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: contribute_ix_data,
        };

        let message = Message::new(&[init_tx], Some(&user.pubkey()));

        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[user], message, recent_blockhash);

        let tx = svm.send_transaction(transaction).unwrap();

        // msg!("\n\n Contributor transaction sucessfull");
        msg!("CUs Consumed by contribute: {}", tx.compute_units_consumed);

        Ok(())
    }

    pub fn init_fundraiser(mut svm: &mut LiteSVM, state: &SetupState) -> Result<(), Error> {
        let maker = &state.maker;
        let maker_ata = state.maker_ata;
        let mint_to_raise = state.mint_to_raise;
        let fundraiser = state.fundraiser;
        let vault = state.vault;
        let token_program = state.token_program;
        let system_program = state.system_program;
        let associated_token_program = state.associated_token_program;

        CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_to_raise)
            .owner(&fundraiser.0)
            .token_program_id(&token_program)
            .send()
            .unwrap();

        let program_id = program_id();

        let amount: u64 = 100_000_000; // 1 token in 6 decimals
        let duration: u64 = 10; // 10 seconds, for example

        let init_data_struct: InitializeInstructionData = InitializeInstructionData {
            amount: amount.to_le_bytes(),
            duration: duration.to_le_bytes(),
        };

        // Serialize into bytes
        let ix_bytes = bytes_of(&init_data_struct).to_vec();
        let init_data = [
            vec![0u8], // Discriminator for "Initialize" instruction
            ix_bytes,
        ]
        .concat();

        msg!("init fundraiser");
        let init_tx = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new(mint_to_raise, false),
                AccountMeta::new(fundraiser.0, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(associated_token_program, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: init_data,
        };

        let message = Message::new(&[init_tx], Some(&maker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[maker], message, recent_blockhash);

        let tx = svm.send_transaction(transaction).unwrap();

        msg!("\n\nInit Fundraiser transaction sucessfull");
        msg!(
            "CUs Consumed by init fundraiser: {}",
            tx.compute_units_consumed
        );

        Ok(())
    }
}
