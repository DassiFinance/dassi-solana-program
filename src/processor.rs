//use crate::utils::{self, AccTypes};
use crate::{
    error::DassiError, instruction::DassiInstruction, state, state::AccTypes,
    state::BorrowerAccount, state::GuarantorAccount, state::LenderAccountData,
    state::LoanInfoAccDataHeader, state::LoanInfoAccLendersData,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{clock::Clock, rent::Rent, Sysvar},
};
//use solana_program_test::tokio::fs::remove_dir;
use spl_token::state::Account as TokenAccount;
// deployed Program Id: 61Yze1wz1D1adaAmuqKnrCBTHhdmW9BmC64Ejv5XK9Hc
//use std::cmp;
use std::convert::TryInto;
//use solana_sdk::{declare_id};

//solana_sdk::declare_id!("EnvhHCLvg55P7PDtbvR1NwuTuAeodqpusV3MR5QEK8gs");
//const ADMIN_PUBKEY: &Pubkey = Pubkey::from_str("EnvhHCLvg55P7PDtbvR1NwuTuAeodqpusV3MR5QEK8gs");

const DASSI_COIN_DECIMALS: u64 = 1000_000_000;
const MIN_LENDING_AMOUNT: u64 = 10_000_000_000u64;
pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("in processor");
        let instruction = DassiInstruction::unpack(instruction_data)?;
        msg!("out instruction");
        match instruction {
            DassiInstruction::LendToBorrower {
                amount_to_lend_input,
                lender_id_input,
            } => {
                msg!("DassiInstruction::LendToBorrower");
                Self::process_lend_to_borrower(
                    accounts,
                    amount_to_lend_input,
                    lender_id_input,
                    program_id,
                )
            }
            DassiInstruction::WithdrawLenderFreeWalletFunds { lender_id_input } => {
                msg!("DassiInstruction::WithdrawLenderFreeWalletFunds");
                Self::process_withdraw_lender_free_wallet_funds(
                    accounts,
                    lender_id_input,
                    program_id,
                )
            }
            DassiInstruction::WithdrawCollectedLoanFunds {} => {
                msg!("DassiInstruction::WithdrawCollectedLoanFunds");
                Self::process_withdraw_collected_loan_funds(accounts, program_id)
            }

            DassiInstruction::TransferDassiVaultAccountOwnership {} => {
                msg!("DassiInstruction::TransferDassiVaultAccountOwnership");
                Self::process_transfer_dassi_vault_account_ownership(accounts, program_id)
            }

            DassiInstruction::InitializeLendersStorageAccount {} => {
                msg!("DassiInstruction::InitializeLendersStorageAccount");
                Self::process_initialize_lenders_storage_account(accounts)
            }
            DassiInstruction::InitializeBorrowerAccount {} => {
                msg!("DassiInstruction::InitializeBorrowerAccount");
                Self::process_initialize_borrower_storage_account(accounts, program_id)
            }
            DassiInstruction::InitializeGuarantorAccount {} => {
                msg!("DassiInstruction::InitializeGuarantorAccount");
                Self::process_initialize_guarantor_storage_account(accounts, program_id)
            }

            DassiInstruction::PayEMIforLoan {
                emi_amount_to_pay_input,
            } => {
                msg!("DassiInstruction::PayEMIforLoan");
                Self::process_pay_emi(accounts, emi_amount_to_pay_input, program_id)
            }

            DassiInstruction::InitializeLoanInfoAccount {
                num_days_left_for_first_repayment_input,
                num_emis_needed_to_repay_the_loan_input,
                num_days_for_fundraising_input,
                total_loan_amount_input,
            } => {
                msg!("DassiInstruction::InitializeLoanInfoAccount");
                Self::initialize_loan_info_account(
                    accounts,
                    num_days_left_for_first_repayment_input,
                    num_emis_needed_to_repay_the_loan_input,
                    num_days_for_fundraising_input,
                    total_loan_amount_input,
                    program_id,
                )
            }

            DassiInstruction::AirdropUsersWithDassiTestCoins {} => {
                msg!("DassiInstruction::AirdropUsersWithDassiTestCoins");
                Self::process_airdrop_users_with_dassi_test_coins(accounts, program_id)
            }

            DassiInstruction::TransferAirdropVaultAccountOwnership {} => {
                msg!("DassiInstruction::TransferAirdropVaultAccountOwnership");
                Self::process_transfer_airdrop_vault_account_ownership(accounts, program_id)
            }

            DassiInstruction::ReturnFundsToLenders { num_accounts_input } => {
                msg!("DassiInstruction::ReturnFundsToLenders");
                Self::process_return_funds_to_lenders(accounts, num_accounts_input, program_id)
            }

            DassiInstruction::CloseLoanInfoAccount {} => {
                msg!("DassiInstruction::CloseLoanInfoAccount");
                Self::process_close_loan_info_account(accounts)
            }
        }
    }

    fn process_lend_to_borrower(
        accounts: &[AccountInfo],
        amount_to_lend_input: u64,
        lender_id_input: u32,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lender_main_account = next_account_info(account_info_iter)?;

        if !lender_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let lender_dassi_coin_account_to_debit = next_account_info(account_info_iter)?;

        // dassi_coin_vault_account is program controlled vault account and can be only controlled by our deployed program for debit funds or any kind of operation
        let dassi_coin_vault_account = next_account_info(account_info_iter)?;

        

        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(DassiError::InvalidTokenProgram.into());
        }

        let dassi_coin_vault_account_data_before =
            TokenAccount::unpack(&dassi_coin_vault_account.data.borrow())?;
        let dassi_coin_vault_balance_before = dassi_coin_vault_account_data_before.amount;

        let (pda_dassi_vault, _bump_seed) =
            Pubkey::find_program_address(&[b"DassiFinance"], program_id);

        if dassi_coin_vault_account_data_before.owner != pda_dassi_vault {
            return Err(DassiError::DassiVaultAccountDoesNotMatched.into());
        }

        let transfer_lending_amount_to_vault_ix = spl_token::instruction::transfer(
            token_program.key,
            lender_dassi_coin_account_to_debit.key,
            dassi_coin_vault_account.key,
            lender_main_account.key,
            &[],
            amount_to_lend_input,
        )?;

        msg!("Calling the token program to transfer lending amount to vault...");
        msg!(
            "amount of dassi coin tokens to transfer {}, lender debit key {}",
            (amount_to_lend_input as f64 / DASSI_COIN_DECIMALS as f64),
            lender_dassi_coin_account_to_debit.key.to_string()
        );

        invoke(
            &transfer_lending_amount_to_vault_ix,
            &[
                lender_dassi_coin_account_to_debit.clone(),
                dassi_coin_vault_account.clone(),
                lender_main_account.clone(),
                token_program.clone(),
            ],
        )?;

        let dassi_coin_vault_account_data_after =
            TokenAccount::unpack(&dassi_coin_vault_account.data.borrow())?;
        let dassi_coin_vault_balance_after = dassi_coin_vault_account_data_after.amount;
        msg!(
            "DassiCoin vault balance after: {}",
            dassi_coin_vault_balance_after
        );
        let vault_balance_increased = dassi_coin_vault_balance_after
            .checked_sub(dassi_coin_vault_balance_before)
            .unwrap();
    
        // Minimum DassiCoin to lend = 10
        if vault_balance_increased < MIN_LENDING_AMOUNT {
            return Err(DassiError::ExpectedAmountMismatch.into());
        }
       
        let loan_info_storage_account = next_account_info(account_info_iter)?;
       
        if loan_info_storage_account.owner != program_id {
            return Err(DassiError::WrongAccountPassed.into());
        }
       
        let lenders_data_storage_account = next_account_info(account_info_iter)?;
    
        if lenders_data_storage_account.owner != program_id {
            return Err(DassiError::WrongAccountPassed.into());
        }
       
        let mut lenders_storage_data_byte_array =
            lenders_data_storage_account.try_borrow_mut_data()?;
        
        if lenders_storage_data_byte_array[0] != AccTypes::LendersAcc as u8 {
            return Err(DassiError::ExpectedAccountTypeMismatched.into());
        }
      
        if lenders_storage_data_byte_array[1] != 1u8 {
            return Err(DassiError::ExpectedLendersAccNumNotMatched.into());
        }   
     
        if lender_id_input > 49_999u32 {
            return Err(DassiError::InvalidLenderIdInput.into());
        }
      
        // lender_id_input can vary from 0 to 49_999 included
        let lender_si_in_lenders_data_byte_array: usize = 2usize
            + (lender_id_input as usize)
                .checked_mul(state::LENDER_ACC_DATA_SIZE)
                .unwrap();
        let lender_ei_in_lenders_data_byte_array: usize =
            lender_si_in_lenders_data_byte_array + state::LENDER_ACC_DATA_SIZE;
        let mut lender_acc_data: LenderAccountData = state::unpack_to_lender_account_data(
            &lenders_storage_data_byte_array
                [lender_si_in_lenders_data_byte_array..lender_ei_in_lenders_data_byte_array],
        )
        .unwrap();

        //let lender_main_account_si: usize = lender_si_in_lenders_data_byte_array + 1usize;
        //let lender_main_account_ei: usize = lender_main_account_si + 32usize;
        // means lender is lending for first time, initialize his account
      
        if lender_acc_data.is_account_active != 1u8 {
            lender_acc_data.is_account_active = 1u8;
            lender_acc_data.lender_main_acc_pubkey = *lender_main_account.key;
        } else {
         
            if lender_acc_data.lender_main_acc_pubkey != *lender_main_account.key {
                return Err(DassiError::InvalidLenderIdInput.into());
            }
        }
       
        // update lender data in LendersAccountDataArray
        lender_acc_data.total_lending_amount = lender_acc_data
            .total_lending_amount
            .checked_add(vault_balance_increased as u128)
            .unwrap();
        lender_acc_data.total_unique_lending_amount = lender_acc_data
            .total_unique_lending_amount
            .checked_add(vault_balance_increased)
            .unwrap();
         
        state::pack_to_lender_account_data(
            lender_acc_data,
            &mut lenders_storage_data_byte_array
                [lender_si_in_lenders_data_byte_array..lender_ei_in_lenders_data_byte_array],
        )
        .unwrap();

        // update lender payment in LoanInfoAccData
        let mut loan_info_data_byte_array = loan_info_storage_account.try_borrow_mut_data()?;
        let mut loan_info_header_data: LoanInfoAccDataHeader = state::unpack_to_loan_info_header(
            &loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();

        if loan_info_header_data.acc_type != AccTypes::LoanInfoAcc as u8 {
            return Err(DassiError::ExpectedAccountTypeMismatched.into());
        }

        let now = Clock::get()?.unix_timestamp as u64;

        if loan_info_header_data.fundraising_period_ending_timestamp < now
            && loan_info_header_data.total_amount_lended < loan_info_header_data.total_loan_amount
        {
            // fundraising period expired, and loan is not fulfilled so, send funds back to lenders
            return Err(DassiError::FundraisingPeriodExpired.into());
        }
     
        if loan_info_header_data.total_amount_lended >= loan_info_header_data.total_loan_amount {
            return Err(DassiError::BorrowerAlreadyFunded.into());
        }

        let loan_info_lender_data_si: usize = state::LOAN_INFO_HEADER_DATA_BYTES
            + (loan_info_header_data.next_index_to_store_lender_data as usize)
                * state::LOAN_INFO_ACC_LENDER_DATA_BYTES;
        let loan_info_lender_data_ei: usize =
            loan_info_lender_data_si + state::LOAN_INFO_ACC_LENDER_DATA_BYTES;
        let mut loan_info_lender_data: LoanInfoAccLendersData =
            state::unpack_to_loan_info_acc_lender_data(
                &loan_info_data_byte_array[loan_info_lender_data_si..loan_info_lender_data_ei],
            )
            .unwrap();

        loan_info_lender_data.lender_main_acc_pubkey = *lender_main_account.key;
        loan_info_lender_data.lenders_data_storage_acc_number = 1u8;
        loan_info_lender_data.lender_id = lender_id_input;
        loan_info_lender_data.lent_amount = vault_balance_increased;

        state::pack_to_loan_info_acc_lender_data(
            loan_info_lender_data,
            &mut loan_info_data_byte_array[loan_info_lender_data_si..loan_info_lender_data_ei],
        )
        .unwrap();

        loan_info_header_data.next_index_to_store_lender_data =
            loan_info_header_data.next_index_to_store_lender_data + 1;
        loan_info_header_data.total_amount_lended = loan_info_header_data
            .total_amount_lended
            .checked_add(vault_balance_increased)
            .unwrap();
        state::pack_to_loan_info_header(
            loan_info_header_data,
            &mut loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();

        Ok(())
    }

    fn process_pay_emi(
        accounts: &[AccountInfo],
        emi_amount_to_pay_input: u64,

        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let borrower_main_account = next_account_info(account_info_iter)?;

        if !borrower_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let borrower_dassi_coin_account_to_debit = next_account_info(account_info_iter)?;

        // dassi_coin_vault_account is program controlled vault account and can be only controlled by our deployed program for debit funds or any kind of operation
        let dassi_coin_vault_account = next_account_info(account_info_iter)?;
        


        let borrower_storage_account = next_account_info(account_info_iter)?;
        if borrower_storage_account.owner != program_id {
            return Err(DassiError::WrongAccountPassed.into());
        }

        /*
        let expected_borrower_storage_account_pubkey = Pubkey::create_with_seed(
            borrower_main_account.key,
            "DassiFinanceBorrower",
            program_id,
        )?;

        if expected_borrower_storage_account_pubkey != *borrower_storage_account.key {
            return Err(DassiError::AccountMismatched.into());
        }
        */

        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(DassiError::InvalidTokenProgram.into());
        }

        let dassi_coin_vault_account_data_before =
            TokenAccount::unpack(&dassi_coin_vault_account.data.borrow())?;
            let (pda_dassi_vault, _bump_seed) =
            Pubkey::find_program_address(&[b"DassiFinance"], program_id);

        if dassi_coin_vault_account_data_before.owner != pda_dassi_vault {
            return Err(DassiError::DassiVaultAccountDoesNotMatched.into());
        }
        let dassi_coin_vault_balance_before = dassi_coin_vault_account_data_before.amount;
        let transfer_emi_amount_to_vault_ix = spl_token::instruction::transfer(
            token_program.key,
            borrower_dassi_coin_account_to_debit.key,
            dassi_coin_vault_account.key,
            borrower_main_account.key,
            &[],
            emi_amount_to_pay_input,
        )?;

        msg!("Calling the token program to transfer emi amount to vault...");
        msg!(
            "amount of dassi coin tokens to transfer {}, borrower debit key {}",
            (emi_amount_to_pay_input as f64 / DASSI_COIN_DECIMALS as f64),
            borrower_dassi_coin_account_to_debit.key.to_string()
        );

        invoke(
            &transfer_emi_amount_to_vault_ix,
            &[
                borrower_dassi_coin_account_to_debit.clone(),
                dassi_coin_vault_account.clone(),
                borrower_main_account.clone(),
                token_program.clone(),
            ],
        )?;

        let dassi_coin_vault_account_data_after =
            TokenAccount::unpack(&dassi_coin_vault_account.data.borrow())?;
        let dassi_coin_vault_balance_after = dassi_coin_vault_account_data_after.amount;
        msg!(
            "DassiCoin vault balance after: {}",
            dassi_coin_vault_balance_after
        );
        let vault_balance_increased = dassi_coin_vault_balance_after
            .checked_sub(dassi_coin_vault_balance_before)
            .unwrap();

        if vault_balance_increased != emi_amount_to_pay_input {
            return Err(DassiError::ExpectedAmountMismatch.into());
        }

        let loan_info_storage_account = next_account_info(account_info_iter)?;

        if loan_info_storage_account.owner != program_id {
            return Err(DassiError::WrongAccountPassed.into());
        }

        // update lender payment in LoanInfoAccData
        let mut loan_info_data_byte_array = loan_info_storage_account.try_borrow_mut_data()?;
        let mut loan_info_header_data: LoanInfoAccDataHeader = state::unpack_to_loan_info_header(
            &loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();

        if loan_info_header_data.acc_type != AccTypes::LoanInfoAcc as u8 {
            return Err(DassiError::ExpectedAccountTypeMismatched.into());
        }

        if loan_info_header_data
            .total_loan_amount
            .checked_div(loan_info_header_data.number_of_emis_needed_to_repay_the_loan as u64)
            .unwrap()
            > emi_amount_to_pay_input
        {
            return Err(DassiError::ExpectedAmountMismatch.into());
        }

        if loan_info_header_data.repaid_amount_by_borrower
            >= loan_info_header_data.total_loan_amount
        {
            return Err(DassiError::LoanAlreadyPaid.into());
        }
        loan_info_header_data.repaid_amount_by_borrower = loan_info_header_data
            .repaid_amount_by_borrower
            .checked_add(vault_balance_increased)
            .unwrap();

        let loan_info_repayment_timestamp_si = 9116usize
            + (loan_info_header_data.next_index_to_store_repayment_info as usize) * (16usize);
        let loan_info_repayment_timestamp_ei = loan_info_repayment_timestamp_si + 8usize;
        let loan_info_repayment_amount_ei = loan_info_repayment_timestamp_ei + 8usize;
        let now = Clock::get()?.unix_timestamp as u64;
        loan_info_data_byte_array
            [loan_info_repayment_timestamp_si..loan_info_repayment_timestamp_ei]
            .copy_from_slice(&now.to_le_bytes());
        loan_info_data_byte_array[loan_info_repayment_timestamp_ei..loan_info_repayment_amount_ei]
            .copy_from_slice(&vault_balance_increased.to_le_bytes());
        loan_info_header_data.next_index_to_store_repayment_info = loan_info_header_data
            .next_index_to_store_repayment_info
            .checked_add(1u8)
            .unwrap();

            let lenders_data_storage_account = next_account_info(account_info_iter)?;

            if lenders_data_storage_account.owner != program_id {
                return Err(DassiError::WrongAccountPassed.into());
            }
    
            let mut lenders_storage_data_byte_array =
                lenders_data_storage_account.try_borrow_mut_data()?;
    
            if lenders_storage_data_byte_array[0] != AccTypes::LendersAcc as u8 {
                return Err(DassiError::ExpectedAccountTypeMismatched.into());
            }
    
            if lenders_storage_data_byte_array[1] != 1u8 {
                return Err(DassiError::ExpectedLendersAccNumNotMatched.into());
            }
    
        let emi_amount_distributed_per_lender: u64 = vault_balance_increased.checked_div(loan_info_header_data.next_index_to_store_lender_data as u64).unwrap();

        for i in 0..loan_info_header_data.next_index_to_store_lender_data {
            let loan_info_lender_data_si: usize = state::LOAN_INFO_HEADER_DATA_BYTES
            + (i as usize)
                * state::LOAN_INFO_ACC_LENDER_DATA_BYTES;
        let loan_info_lender_data_ei: usize =
            loan_info_lender_data_si + state::LOAN_INFO_ACC_LENDER_DATA_BYTES;
        let loan_info_lender_data: LoanInfoAccLendersData =
            state::unpack_to_loan_info_acc_lender_data(
                &loan_info_data_byte_array[loan_info_lender_data_si..loan_info_lender_data_ei],
            )
            .unwrap();
        

        // lender_id_input can vary from 0 to 49_999 included
        let lender_si_in_lenders_data_byte_array: usize = 2usize
            + (loan_info_lender_data.lender_id as usize)
                .checked_mul(state::LENDER_ACC_DATA_SIZE)
                .unwrap();
        let lender_ei_in_lenders_data_byte_array: usize =
            lender_si_in_lenders_data_byte_array + state::LENDER_ACC_DATA_SIZE;
        let mut lender_acc_data: LenderAccountData = state::unpack_to_lender_account_data(
            &lenders_storage_data_byte_array
                [lender_si_in_lenders_data_byte_array..lender_ei_in_lenders_data_byte_array],
        )
        .unwrap();

        // update lender data in LendersAccountDataArray
        lender_acc_data.total_lending_amount = lender_acc_data
            .total_lending_amount
            .checked_add(vault_balance_increased as u128)
            .unwrap();
        lender_acc_data.total_unique_lending_amount = lender_acc_data
            .total_unique_lending_amount
            .checked_add(vault_balance_increased)
            .unwrap();

        lender_acc_data.amount_to_withdraw_or_lend = lender_acc_data.amount_to_withdraw_or_lend.checked_add(emi_amount_distributed_per_lender).unwrap();

        state::pack_to_lender_account_data(
            lender_acc_data,
            &mut lenders_storage_data_byte_array
                [lender_si_in_lenders_data_byte_array..lender_ei_in_lenders_data_byte_array],
        )
        .unwrap();
        }

        state::pack_to_loan_info_header(
            loan_info_header_data,
            &mut loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();

        Ok(())
    }

    fn initialize_loan_info_account(
        accounts: &[AccountInfo],
        num_days_left_for_first_repayment_input: u16,
        num_emis_needed_to_repay_the_loan_input: u16,
        num_days_for_fundraising_input: u16,
        total_loan_amount_input: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        // also update borrower storage account
        let account_info_iter = &mut accounts.iter();
        let guarantor_main_account = next_account_info(account_info_iter)?;

        if !guarantor_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
       let borrower_main_account = next_account_info(account_info_iter)?;

        let loan_info_storage_account = next_account_info(account_info_iter)?;

        // just for extra safety, even this check is not required
        if loan_info_storage_account.owner != program_id {
            return Err(DassiError::WrongAccountPassed.into());
        }

        let rent = Rent::get()?;

        if !rent.is_exempt(
            loan_info_storage_account.lamports(),
            loan_info_storage_account.data_len(),
        ) {
            return Err(DassiError::NotRentExempt.into());
        }

        if loan_info_storage_account.data_len() != state::LOAN_INFO_ACC_DATA_SIZE {
            return Err(DassiError::DataSizeNotMatched.into());
        }

        let mut loan_info_data_byte_array = loan_info_storage_account.data.borrow_mut();

        if loan_info_data_byte_array[0] != 0 {
            return Err(DassiError::LoanInfoDataAlreadyInitialized.into());
        }

        let borrower_storage_account = next_account_info(account_info_iter)?;
        if borrower_storage_account.owner != program_id {
            return Err(DassiError::WrongAccountPassed.into());
        }

        /*
        let expected_borrower_storage_account_pubkey = Pubkey::create_with_seed(
            borrower_main_account.key,
            "DassiFinanceBorrower",
            program_id,
        )?;

        if expected_borrower_storage_account_pubkey != *borrower_storage_account.key {
            return Err(DassiError::AccountMismatched.into());
        }
        */

        let mut borrower_data =
            BorrowerAccount::unpack(&borrower_storage_account.data.try_borrow().unwrap())?;
        if borrower_data.is_active_loan != 0 {
            return Err(DassiError::BorrowerAlreadyHaveActiveLoan.into());
        }
        if borrower_data.acc_type != AccTypes::BorrowerAcc as u8 {
            return Err(DassiError::ExpectedAccountTypeMismatched.into());
        }
        borrower_data.active_loan_address = *loan_info_storage_account.key;
        BorrowerAccount::pack(
            borrower_data,
            &mut borrower_storage_account.data.try_borrow_mut().unwrap(),
        )?;

        let num_seconds_in_one_day: u64 = 86400u64;
        let now = Clock::get()?.unix_timestamp as u64;
        let mut loan_info_header_data: LoanInfoAccDataHeader = state::unpack_unchecked_to_loan_info_header(
            &loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();
        loan_info_header_data.acc_type = AccTypes::LoanInfoAcc as u8;
        loan_info_header_data.borrower_main_acc_pubkey = *borrower_main_account.key;
        loan_info_header_data.guarantor_main_acc_pubkey = *guarantor_main_account.key;
        loan_info_header_data.loan_approval_timestamp = now.clone();
        let calculate_fundraising_period_ending_timestamp: u64 = now
            .checked_add(
                (num_days_for_fundraising_input as u64)
                    .checked_mul(num_seconds_in_one_day)
                    .unwrap(),
            )
            .unwrap();
        loan_info_header_data.fundraising_period_ending_timestamp =
            calculate_fundraising_period_ending_timestamp;
        // a user can pay upto 5 days late, after that his credit score will decrease
        loan_info_header_data.first_repayment_last_date_timestamp =
            (num_days_left_for_first_repayment_input
                .checked_add(5u16)
                .unwrap() as u64)
                .checked_mul(num_seconds_in_one_day)
                .unwrap();
        loan_info_header_data.total_loan_amount = total_loan_amount_input;
        loan_info_header_data.number_of_emis_needed_to_repay_the_loan =
            num_emis_needed_to_repay_the_loan_input as u8;

        state::pack_to_loan_info_header(
            loan_info_header_data,
            &mut loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();

        Ok(())
    }

    // This will credit all free funds to lender wallet
    fn process_withdraw_lender_free_wallet_funds(
        accounts: &[AccountInfo],
        lender_id_input: u32,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lender_main_account = next_account_info(account_info_iter)?;

        if !lender_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let lender_dassi_coin_account_to_credit = next_account_info(account_info_iter)?;

        // dassi_coin_vault_account is program controlled vault account and can be only controlled by our deployed program for debit funds or any kind of operation
        let dassi_coin_vault_account = next_account_info(account_info_iter)?;

        /*
        check if right vault is passed 
        */

        let lenders_data_storage_account = next_account_info(account_info_iter)?;

        if lenders_data_storage_account.owner != program_id {
            return Err(DassiError::WrongAccountPassed.into());
        }

        let mut lenders_storage_data_byte_array =
            lenders_data_storage_account.try_borrow_mut_data()?;

        if lenders_storage_data_byte_array[0] != AccTypes::LendersAcc as u8 {
            return Err(DassiError::ExpectedAccountTypeMismatched.into());
        }

        if lenders_storage_data_byte_array[1] != 1u8 {
            return Err(DassiError::ExpectedLendersAccNumNotMatched.into());
        }

        if lender_id_input > 49_999u32 {
            return Err(DassiError::InvalidLenderIdInput.into());
        }

        // lender_id_input can vary from 0 to 49_999 included
        let lender_si_in_lenders_data_byte_array: usize = 2usize
            + (lender_id_input as usize)
                .checked_mul(state::LENDER_ACC_DATA_SIZE)
                .unwrap();
        let lender_ei_in_lenders_data_byte_array: usize =
            lender_si_in_lenders_data_byte_array + state::LENDER_ACC_DATA_SIZE;
        let mut lender_acc_data: LenderAccountData = state::unpack_to_lender_account_data(
            &lenders_storage_data_byte_array
                [lender_si_in_lenders_data_byte_array..lender_ei_in_lenders_data_byte_array],
        )
        .unwrap();

        if lender_acc_data.is_account_active != 1u8
            && lender_acc_data.lender_main_acc_pubkey != *lender_main_account.key
        {
            return Err(DassiError::InvalidLenderIdInput.into());
        }

        let withdraw_amount: u64 = lender_acc_data.amount_to_withdraw_or_lend;
        lender_acc_data.total_unique_lending_amount = lender_acc_data
            .total_unique_lending_amount
            .checked_sub(withdraw_amount.clone())
            .unwrap();
        lender_acc_data.amount_to_withdraw_or_lend = 0u64;

        state::pack_to_lender_account_data(
            lender_acc_data,
            &mut lenders_storage_data_byte_array
                [lender_si_in_lenders_data_byte_array..lender_ei_in_lenders_data_byte_array],
        )
        .unwrap();

        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(DassiError::InvalidTokenProgram.into());
        }

        let dassi_coin_vault_account_data_before =
            TokenAccount::unpack(&dassi_coin_vault_account.data.borrow())?;
        let dassi_coin_vault_balance_before = dassi_coin_vault_account_data_before.amount;

        let pda_account = next_account_info(account_info_iter)?;

        // we can also store bump_seed to save computations
        let (pda, bump_seed) = Pubkey::find_program_address(&[b"DassiFinance"], program_id);
        if pda != *pda_account.key {
            return Err(DassiError::PdaAccountDoesNotMatched.into());
        }

        let transfer_withdraw_amount_to_lender_ix = spl_token::instruction::transfer(
            token_program.key,
            dassi_coin_vault_account.key,
            lender_dassi_coin_account_to_credit.key,
            &pda,
            &[&pda],
            withdraw_amount,
        )?;
        msg!("Calling the token program to transfer withdraw amount to lender...");
        msg!(
            "amount of dassi coin tokens to transfer {}, lender credit key {}",
            (withdraw_amount as f64 / DASSI_COIN_DECIMALS as f64),
            lender_dassi_coin_account_to_credit.key.to_string()
        );
        invoke_signed(
            &transfer_withdraw_amount_to_lender_ix,
            &[
                dassi_coin_vault_account.clone(),
                lender_dassi_coin_account_to_credit.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"DassiFinance"[..], &[bump_seed]]],
        )?;

        let dassi_coin_vault_account_data_after =
            TokenAccount::unpack(&dassi_coin_vault_account.data.borrow())?;
        let dassi_coin_vault_balance_after = dassi_coin_vault_account_data_after.amount;
        msg!(
            "DassiCoin vault balance after: {}",
            dassi_coin_vault_balance_after
        );

        let vault_balance_decreased = dassi_coin_vault_balance_before
            .checked_sub(dassi_coin_vault_balance_after)
            .unwrap();

        if vault_balance_decreased != withdraw_amount {
            return Err(DassiError::ExpectedAmountMismatch.into());
        }

        Ok(())
    }

    fn process_withdraw_collected_loan_funds(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let borrower_main_account = next_account_info(account_info_iter)?;

        if !borrower_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let borrower_dassi_ata_to_credit = next_account_info(account_info_iter)?;

        let dassi_coin_vault_account = next_account_info(account_info_iter)?;

        /*
        let dassi_vault_pubkey = utils::get_dassi_vault_pubkey();

        if dassi_vault_pubkey != *dassi_coin_vault_account.key {
            return Err(DassiError::DassiVaultAccountDoesNotMatched.into());
        }

        */

        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(DassiError::InvalidTokenProgram.into());
        }

        let loan_info_storage_account = next_account_info(account_info_iter)?;

        if loan_info_storage_account.owner != program_id {
            return Err(DassiError::WrongAccountPassed.into());
        }

        // update lender payment in LoanInfoAccData
        let mut loan_info_data_byte_array = loan_info_storage_account.try_borrow_mut_data()?;
        let mut loan_info_header_data: LoanInfoAccDataHeader = state::unpack_to_loan_info_header(
            &loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();

        if loan_info_header_data.acc_type != AccTypes::LoanInfoAcc as u8 {
            return Err(DassiError::ExpectedAccountTypeMismatched.into());
        }

        if loan_info_header_data.total_amount_lended < loan_info_header_data.total_loan_amount {
            return Err(DassiError::CollectedLoanFundsAlreadyWithdrawn.into());
        }

        if loan_info_header_data.borrower_main_acc_pubkey != *borrower_main_account.key {
            return Err(DassiError::BorrowerAccountMismatched.into());
        }

        // borrower can withdraw total_amount_lended, after withdrawing we will set it to 0, so that he can't withdraw second time. We can also use a extra variable to store if funds
        // withdrawn or not

        let dassi_coin_vault_account_data_before =
            TokenAccount::unpack(&dassi_coin_vault_account.data.borrow())?;
        let dassi_coin_vault_balance_before = dassi_coin_vault_account_data_before.amount;

        let pda_account = next_account_info(account_info_iter)?;

        // we can also store bump_seed to save computations
        let (pda, bump_seed) = Pubkey::find_program_address(&[b"DassiFinance"], program_id);
        if pda != *pda_account.key {
            return Err(DassiError::PdaAccountDoesNotMatched.into());
        }

        let transfer_collected_loan_funds_to_borrower_ix = spl_token::instruction::transfer(
            token_program.key,
            dassi_coin_vault_account.key,
            borrower_dassi_ata_to_credit.key,
            &pda,
            &[&pda],
            loan_info_header_data.total_amount_lended,
        )?;
        msg!("Calling the token program to transfer collected loan amount to borrower...");
        msg!(
            "amount of dassi coin tokens to transfer {}, lender credit key {}",
            (loan_info_header_data.total_amount_lended as f64 / DASSI_COIN_DECIMALS as f64),
            borrower_dassi_ata_to_credit.key.to_string()
        );
        invoke_signed(
            &transfer_collected_loan_funds_to_borrower_ix,
            &[
                dassi_coin_vault_account.clone(),
                borrower_dassi_ata_to_credit.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"DassiFinance"[..], &[bump_seed]]],
        )?;

        let dassi_coin_vault_account_data_after =
            TokenAccount::unpack(&dassi_coin_vault_account.data.borrow())?;
        let dassi_coin_vault_balance_after = dassi_coin_vault_account_data_after.amount;
        msg!(
            "DassiCoin vault balance after: {}",
            dassi_coin_vault_balance_after
        );

        let vault_balance_decreased = dassi_coin_vault_balance_before
            .checked_sub(dassi_coin_vault_balance_after)
            .unwrap();

        if vault_balance_decreased != loan_info_header_data.total_amount_lended {
            return Err(DassiError::ExpectedAmountMismatch.into());
        }

        loan_info_header_data.total_amount_lended = 0u64;

        state::pack_to_loan_info_header(
            loan_info_header_data,
            &mut loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();

        Ok(())
    }

    fn process_transfer_dassi_vault_account_ownership(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer_account = next_account_info(account_info_iter)?;

        if !initializer_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let dassi_vault_account = next_account_info(account_info_iter)?;

        let (pda, _nonce) = Pubkey::find_program_address(&[b"DassiFinance"], program_id);

        let rent = Rent::get()?;

        if !rent.is_exempt(
            dassi_vault_account.lamports(),
            dassi_vault_account.data_len(),
        ) {
            return Err(DassiError::NotRentExempt.into());
        }
        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(DassiError::InvalidTokenProgram.into());
        }

        let dassi_vault_owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            dassi_vault_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer_account.key,
            &[&initializer_account.key],
        )?;

        msg!("Calling the token program to transfer dassi vault account ownership to program...");
        invoke(
            &dassi_vault_owner_change_ix,
            &[
                dassi_vault_account.clone(),
                initializer_account.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_initialize_lenders_storage_account(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer_account = next_account_info(account_info_iter)?;

        if !initializer_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let lenders_storage_account = next_account_info(account_info_iter)?;

        let rent = Rent::get()?;

        if !rent.is_exempt(
            lenders_storage_account.lamports(),
            lenders_storage_account.data_len(),
        ) {
            return Err(DassiError::NotRentExempt.into());
        }

        if lenders_storage_account.data_len() != state::LENDERS_STORAGE_ACC_DATA_SIZE {
            return Err(DassiError::DataSizeNotMatched.into());
        }

        let mut lenders_storage_data_byte_array =
            lenders_storage_account.data.try_borrow_mut().unwrap();

        if lenders_storage_data_byte_array[0] != 0 {
            return Err(DassiError::LendersStorageDataAlreadyInitialized.into());
        }

        lenders_storage_data_byte_array[0] = AccTypes::LendersAcc as u8;
        // currently for prototype I'm making every lenders_data_storage_acc_number to 1, but in future when we need more accounts, we have to increment it for every new account generation
        lenders_storage_data_byte_array[1] = 1u8;

        Ok(())
    }

    fn process_initialize_borrower_storage_account(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let borrower_main_account = next_account_info(account_info_iter)?;
        if !borrower_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let borrower_storage_account = next_account_info(account_info_iter)?;
        if borrower_storage_account.owner != program_id {
            return Err(DassiError::WrongAccountPassed.into());
        }

        /*
        let expected_borrower_storage_account_pubkey = Pubkey::create_with_seed(
            borrower_main_account.key,
            "DassiFinanceBorrower",
            program_id,
        )?;

        if expected_borrower_storage_account_pubkey != *borrower_storage_account.key {
            return Err(DassiError::AccountMismatched.into());
        }
        */

        let rent = Rent::get()?;
        if !rent.is_exempt(
            borrower_storage_account.lamports(),
            borrower_storage_account.data_len(),
        ) {
            return Err(DassiError::NotRentExempt.into());
        }

        // put a condition if borrower_storage_account.data_len() != 75, then error

        let mut borrower_data =
            BorrowerAccount::unpack_unchecked(&borrower_storage_account.data.borrow())?;

        if borrower_data.is_initialized() {
            return Err(DassiError::BorrowerAccountAlreadyInitialized.into());
        }

        borrower_data.is_initialized = true;
        borrower_data.acc_type = AccTypes::BorrowerAcc as u8;
        borrower_data.borrower_main_acc_pubkey = *borrower_main_account.key;
        borrower_data.credit_score = 500_000_000_000u64;

        BorrowerAccount::pack(
            borrower_data,
            &mut borrower_storage_account.data.borrow_mut(),
        )?;

        Ok(())
    }

    fn process_initialize_guarantor_storage_account(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let guarantor_main_account = next_account_info(account_info_iter)?;
        if !guarantor_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let guarantor_storage_account = next_account_info(account_info_iter)?;
        if guarantor_storage_account.owner != program_id {
            return Err(DassiError::WrongAccountPassed.into());
        }

        /*
        let expected_guarantor_storage_account_pubkey = Pubkey::create_with_seed(
            guarantor_main_account.key,
            "DassiFinanceGuarantor",
            program_id,
        )?;

        if expected_guarantor_storage_account_pubkey != *guarantor_storage_account.key {
            return Err(DassiError::AccountMismatched.into());
        }
        */
        let rent = Rent::get()?;
        if !rent.is_exempt(
            guarantor_storage_account.lamports(),
            guarantor_storage_account.data_len(),
        ) {
            return Err(DassiError::NotRentExempt.into());
        }

        // put a condition if guarantor_storage_account.data_len() != 75, then error

        let mut guarantor_data =
            GuarantorAccount::unpack_unchecked(&guarantor_storage_account.data.borrow())?;

        if guarantor_data.is_initialized() {
            return Err(DassiError::GuarantorAccountAlreadyInitialized.into());
        }

        guarantor_data.is_initialized = true;
        guarantor_data.acc_type = AccTypes::GuarantorAcc as u8;
        guarantor_data.guarantor_main_acc_pubkey = *guarantor_main_account.key;
        guarantor_data.approval_score = 500_000_000_000u64;

        GuarantorAccount::pack(
            guarantor_data,
            &mut guarantor_storage_account.data.borrow_mut(),
        )?;

        Ok(())
    }

    //On each airdrop, users will get 250 DassiCoin test tokens. A user can airdrop a maximum of 10 times.
    fn process_airdrop_users_with_dassi_test_coins(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let amount_to_airdrop = 500_000_000_000u64;
        let max_amount_to_airdrop = 2_500_000_000_000u64;

        let account_info_iter = &mut accounts.iter();
        let airdrop_user_main_account = next_account_info(account_info_iter)?;

        if !airdrop_user_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let dassi_coin_airdrop_user_storage_account = next_account_info(account_info_iter)?;

        let rent = Rent::get()?;

        if !rent.is_exempt(
            dassi_coin_airdrop_user_storage_account.lamports(),
            dassi_coin_airdrop_user_storage_account.data_len(),
        ) {
            return Err(DassiError::NotRentExempt.into());
        }

        let expected_dassi_coin_airdrop_user_storage_pubkey = Pubkey::create_with_seed(
            airdrop_user_main_account.key,
            "DassiFinanceAirdrop",
            program_id,
        )?;

        if expected_dassi_coin_airdrop_user_storage_pubkey
            != *dassi_coin_airdrop_user_storage_account.key
        {
            return Err(DassiError::AccountMismatched.into());
        }

        let mut dassi_coin_airdrop_user_storage_byte_array =
            dassi_coin_airdrop_user_storage_account.try_borrow_mut_data()?;
        let stored_amount = u64::from_le_bytes(
            dassi_coin_airdrop_user_storage_byte_array[0..8]
                .try_into()
                .unwrap(),
        );
        if stored_amount >= max_amount_to_airdrop {
            return Err(DassiError::UserAlreadyAirdroped.into());
        }
        let new_total_airdrop_amount_for_user: u64 =
            stored_amount.checked_add(amount_to_airdrop).unwrap();
        dassi_coin_airdrop_user_storage_byte_array
            .copy_from_slice(&new_total_airdrop_amount_for_user.to_le_bytes());

        let user_dassi_coin_associated_token_to_credit_account =
            next_account_info(account_info_iter)?;

        let airdrop_vault_dassi_coin_account = next_account_info(account_info_iter)?;

        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(DassiError::InvalidTokenProgram.into());
        }

        let pda_account = next_account_info(account_info_iter)?;

        // we can also store bump_seed to save computations
        let (pda, bump_seed) = Pubkey::find_program_address(&[b"DassiFinanceAirdrop"], program_id);
        if pda != *pda_account.key {
            return Err(DassiError::PdaAccountDoesNotMatched.into());
        }

        let airdrop_vault_account_data =
            TokenAccount::unpack(&airdrop_vault_dassi_coin_account.data.borrow())?;

        let airdrop_vault_balance_before = airdrop_vault_account_data.amount;

        let transfer_dassi_coin_to_airdroper_ix = spl_token::instruction::transfer(
            token_program.key,
            airdrop_vault_dassi_coin_account.key,
            user_dassi_coin_associated_token_to_credit_account.key,
            &pda,
            &[&pda],
            amount_to_airdrop,
        )?;
        msg!("Calling the token program to transfer airdrop tokens to the user...");
        invoke_signed(
            &transfer_dassi_coin_to_airdroper_ix,
            &[
                airdrop_vault_dassi_coin_account.clone(),
                user_dassi_coin_associated_token_to_credit_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"DassiFinanceAirdrop"[..], &[bump_seed]]],
        )?;

        let airdrop_vault_account_data_after =
            TokenAccount::unpack(&airdrop_vault_dassi_coin_account.data.borrow())?;
        let airdrop_vault_balance_after = airdrop_vault_account_data_after.amount;
        let vault_balance_decreased = airdrop_vault_balance_before
            .checked_sub(airdrop_vault_balance_after)
            .unwrap();

        if vault_balance_decreased != amount_to_airdrop {
            return Err(DassiError::ExpectedAmountMismatch.into());
        }

        Ok(())
    }

    fn process_transfer_airdrop_vault_account_ownership(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer_account = next_account_info(account_info_iter)?;

        if !initializer_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let airdrop_vault_dassi_coin_account = next_account_info(account_info_iter)?;

        let (pda, _nonce) = Pubkey::find_program_address(&[b"DassiFinanceAirdrop"], program_id);

        let rent = Rent::get()?;

        if !rent.is_exempt(
            airdrop_vault_dassi_coin_account.lamports(),
            airdrop_vault_dassi_coin_account.data_len(),
        ) {
            return Err(DassiError::NotRentExempt.into());
        }
        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(DassiError::InvalidTokenProgram.into());
        }

        let airdrop_vault_owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            airdrop_vault_dassi_coin_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer_account.key,
            &[&initializer_account.key],
        )?;

        msg!("Calling the token program to transfer Dassi airdrop vault account ownership to program...");
        invoke(
            &airdrop_vault_owner_change_ix,
            &[
                airdrop_vault_dassi_coin_account.clone(),
                initializer_account.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_close_loan_info_account(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        Ok(())
    }

    fn process_return_funds_to_lenders(
        accounts: &[AccountInfo],
        num_accounts_input: u16,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        Ok(())
    }
}
