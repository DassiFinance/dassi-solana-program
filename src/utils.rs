use solana_program::{msg, pubkey::Pubkey};
use std::convert::TryInto;


pub enum AccTypes {
    BorrowerAcc = 2,
    LendersAcc = 3,
    GuarantorAcc = 4,
    LoanInfoAcc = 5,
}
// AccTypes::LenderAcc as u8


pub fn get_admin_pubkey() -> Pubkey {
    let admin_pubkey_str = "857Tm9dNi6Ypur9zCcJ9oAhqYd3bE6J6s2ww77PKCSa";
    let pubkey_vec = bs58::decode(admin_pubkey_str).into_vec().unwrap();
    let admin_pubkey = Pubkey::new(&pubkey_vec);
    return admin_pubkey;
}



/*

let borrower_storage_account = next_account_info(account_info_iter)?;
        if borrower_storage_account.owner != program_id {
            return Err(DassiError::WrongAccountPassed.into());
        }

        let expected_borrower_storage_account_pubkey = Pubkey::create_with_seed(
            borrower_main_account.key,
            "DassiFinanceBorrower",
            program_id,
        )?;

        if expected_borrower_storage_account_pubkey != *borrower_storage_account.key {
            return Err(DassiError::AccountMismatched.into());
        }

        let mut borrower_data = BorrowerAccount::unpack(&borrower_storage_account.data.try_borrow().unwrap())?;
        if borrower_data.is_active_loan != 0 {
            return Err(DassiError::BorrowerAlreadyHaveActiveLoan.into());
        }
        if borrower_data.acc_type != AccTypes::BorrowerAcc as u8 {
            return Err(DassiError::ExpectedAccountTypeMismatched.into());
        }
        borrower_data.active_loan_address = *loan_info_storage_account.key;
        BorrowerAccount::pack(borrower_data, &mut borrower_storage_account.data.try_borrow_mut().unwrap())?;

        */