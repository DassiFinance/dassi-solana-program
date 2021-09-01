use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use crate::error::DassiError;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
pub enum AccTypes {
    BorrowerAcc = 2,
    LendersAcc = 3,
    GuarantorAcc = 4,
    LoanInfoAcc = 5,
}
// AccTypes::LenderAcc as u8

// total bytes needed to store LoanInfoAccData = 116 + 9000 + 800 = 9916
pub const LOAN_INFO_ACC_DATA_SIZE: usize = 9916;
// total bytes needed to store lender data = 65*50_000 + 2
pub const LENDERS_STORAGE_ACC_DATA_SIZE: usize = 3_250_002;
pub struct BorrowerAccount {
    pub is_initialized: bool,
    pub acc_type: u8,
    pub is_active_loan: u8,
    pub borrower_main_acc_pubkey: Pubkey,
    pub credit_score: u64,
    pub active_loan_address: Pubkey,
}

impl Sealed for BorrowerAccount {}

impl IsInitialized for BorrowerAccount {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for BorrowerAccount {
    const LEN: usize = 75;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, BorrowerAccount::LEN];
        let (
            is_initialized_src,
            type_src,
            is_active_loan_src,
            borrower_main_acc_pubkey_src,
            credit_score_src,
            active_loan_address_src,
        ) = array_refs![src, 1, 1, 1, 32, 8, 32];

        let is_initialized = match is_initialized_src {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(BorrowerAccount {
            is_initialized,
            acc_type: type_src[0],
            is_active_loan: is_active_loan_src[0],
            borrower_main_acc_pubkey: Pubkey::new_from_array(*borrower_main_acc_pubkey_src),
            credit_score: u64::from_le_bytes(*credit_score_src),
            active_loan_address: Pubkey::new_from_array(*active_loan_address_src),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, BorrowerAccount::LEN];
        let (
            is_initialized_dst,
            type_dst,
            is_active_loan_dst,
            borrower_main_acc_pubkey_dst,
            credit_score_dst,
            active_loan_address_dst,
        ) = mut_array_refs![dst, 1, 1, 1, 32, 8, 32];
        let BorrowerAccount {
            is_initialized,
            acc_type,
            is_active_loan,
            borrower_main_acc_pubkey,
            credit_score,
            active_loan_address,
        } = self;
        is_initialized_dst[0] = *is_initialized as u8;
        type_dst[0] = *acc_type as u8;
        is_active_loan_dst[0] = *is_active_loan as u8;
        borrower_main_acc_pubkey_dst.copy_from_slice(borrower_main_acc_pubkey.as_ref());
        *credit_score_dst = credit_score.to_le_bytes();
        active_loan_address_dst.copy_from_slice(active_loan_address.as_ref());
    }
}

pub struct GuarantorAccount {
    pub is_initialized: bool,
    pub acc_type: u8,
    pub guarantor_main_acc_pubkey: Pubkey,
    pub approval_score: u64,
}

impl Sealed for GuarantorAccount {}

impl IsInitialized for GuarantorAccount {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for GuarantorAccount {
    const LEN: usize = 42;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, GuarantorAccount::LEN];
        let (is_initialized_src, type_src, guarantor_main_acc_pubkey_src, approval_score_src) =
            array_refs![src, 1, 1, 32, 8];

        let is_initialized = match is_initialized_src {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(GuarantorAccount {
            is_initialized,
            acc_type: type_src[0],
            guarantor_main_acc_pubkey: Pubkey::new_from_array(*guarantor_main_acc_pubkey_src),
            approval_score: u64::from_le_bytes(*approval_score_src),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, GuarantorAccount::LEN];
        let (is_initialized_dst, type_dst, guarantor_main_acc_pubkey_dst, approval_score_dst) =
            mut_array_refs![dst, 1, 1, 32, 8];
        let GuarantorAccount {
            is_initialized,
            acc_type,
            guarantor_main_acc_pubkey,
            approval_score,
        } = self;
        is_initialized_dst[0] = *is_initialized as u8;
        type_dst[0] = *acc_type as u8;
        guarantor_main_acc_pubkey_dst.copy_from_slice(guarantor_main_acc_pubkey.as_ref());
        *approval_score_dst = approval_score.to_le_bytes();
    }
}

// LoanInfoAccDataHeader has 116 bytes of data
pub struct LoanInfoAccDataHeader {
    pub acc_type: u8,
    pub borrower_main_acc_pubkey: Pubkey,
    pub guarantor_main_acc_pubkey: Pubkey,
    pub loan_approval_timestamp: u64,
    pub fundraising_period_ending_timestamp: u64,
    pub first_repayment_last_date_timestamp: u64,
    pub total_loan_amount: u64,
    pub total_amount_lended: u64,
    pub repaid_amount_by_borrower: u64,
    pub next_index_to_store_lender_data: u8,
    pub next_index_to_store_repayment_info: u8,
    pub number_of_emis_needed_to_repay_the_loan: u8,
}

pub fn unpack_to_loan_info_header(input: &[u8]) -> Result<LoanInfoAccDataHeader, ProgramError> {
    const LOAN_INFO_HEADER_DATA_BYTES: usize = 116;
    if input.len() != LOAN_INFO_HEADER_DATA_BYTES {
        return Err(ProgramError::InvalidAccountData);
    }
    let input = array_ref![input, 0, LOAN_INFO_HEADER_DATA_BYTES];
    let (
        acc_type_src,
        borrower_main_acc_pubkey_src,
        guarantor_main_acc_pubkey_src,
        loan_approval_timestamp_src,
        fundraising_period_ending_timestamp_src,
        first_repayment_last_date_timestamp_src,
        total_loan_amount_src,
        total_amount_lended_src,
        repaid_amount_by_borrower_src,
        next_index_to_store_lender_data_src,
        next_index_to_store_repayment_info_src,
        number_of_emis_needed_to_repay_the_loan_src,
    ) = array_refs![input, 1, 32, 32, 8, 8, 8, 8, 8, 8, 1, 1, 1];

    if acc_type_src[0] != AccTypes::LoanInfoAcc as u8 {
        return Err(DassiError::ExpectedAccountTypeMismatched.into());
    }

    Ok(LoanInfoAccDataHeader {
        acc_type: acc_type_src[0],
        borrower_main_acc_pubkey: Pubkey::new_from_array(*borrower_main_acc_pubkey_src),
        guarantor_main_acc_pubkey: Pubkey::new_from_array(*guarantor_main_acc_pubkey_src),
        loan_approval_timestamp: u64::from_le_bytes(*loan_approval_timestamp_src),
        fundraising_period_ending_timestamp: u64::from_le_bytes(
            *fundraising_period_ending_timestamp_src,
        ),
        first_repayment_last_date_timestamp: u64::from_le_bytes(
            *first_repayment_last_date_timestamp_src,
        ),
        total_loan_amount: u64::from_le_bytes(*total_loan_amount_src),
        total_amount_lended: u64::from_le_bytes(*total_amount_lended_src),
        repaid_amount_by_borrower: u64::from_le_bytes(*repaid_amount_by_borrower_src),
        next_index_to_store_lender_data: next_index_to_store_lender_data_src[0],
        next_index_to_store_repayment_info: next_index_to_store_repayment_info_src[0],
        number_of_emis_needed_to_repay_the_loan: number_of_emis_needed_to_repay_the_loan_src[0],
    })
}

     
pub fn unpack_unchecked_to_loan_info_header(input: &[u8]) -> Result<LoanInfoAccDataHeader, ProgramError> {
    const LOAN_INFO_HEADER_DATA_BYTES: usize = 116;
    if input.len() != LOAN_INFO_HEADER_DATA_BYTES {
        return Err(ProgramError::InvalidAccountData);
    }
    let input = array_ref![input, 0, LOAN_INFO_HEADER_DATA_BYTES];
    let (
        acc_type_src,
        borrower_main_acc_pubkey_src,
        guarantor_main_acc_pubkey_src,
        loan_approval_timestamp_src,
        fundraising_period_ending_timestamp_src,
        first_repayment_last_date_timestamp_src,
        total_loan_amount_src,
        total_amount_lended_src,
        repaid_amount_by_borrower_src,
        next_index_to_store_lender_data_src,
        next_index_to_store_repayment_info_src,
        number_of_emis_needed_to_repay_the_loan_src,
    ) = array_refs![input, 1, 32, 32, 8, 8, 8, 8, 8, 8, 1, 1, 1];
/*
    if acc_type_src[0] != AccTypes::LoanInfoAcc as u8 {
        return Err(DassiError::ExpectedAccountTypeMismatched.into());
    }
*/
    Ok(LoanInfoAccDataHeader {
        acc_type: acc_type_src[0],
        borrower_main_acc_pubkey: Pubkey::new_from_array(*borrower_main_acc_pubkey_src),
        guarantor_main_acc_pubkey: Pubkey::new_from_array(*guarantor_main_acc_pubkey_src),
        loan_approval_timestamp: u64::from_le_bytes(*loan_approval_timestamp_src),
        fundraising_period_ending_timestamp: u64::from_le_bytes(
            *fundraising_period_ending_timestamp_src,
        ),
        first_repayment_last_date_timestamp: u64::from_le_bytes(
            *first_repayment_last_date_timestamp_src,
        ),
        total_loan_amount: u64::from_le_bytes(*total_loan_amount_src),
        total_amount_lended: u64::from_le_bytes(*total_amount_lended_src),
        repaid_amount_by_borrower: u64::from_le_bytes(*repaid_amount_by_borrower_src),
        next_index_to_store_lender_data: next_index_to_store_lender_data_src[0],
        next_index_to_store_repayment_info: next_index_to_store_repayment_info_src[0],
        number_of_emis_needed_to_repay_the_loan: number_of_emis_needed_to_repay_the_loan_src[0],
    })
}

pub fn pack_to_loan_info_header(
    src: LoanInfoAccDataHeader,
    dst: &mut [u8],
) -> Result<(), ProgramError> {
    const LOAN_INFO_HEADER_DATA_BYTES: usize = 116;
    if dst.len() != LOAN_INFO_HEADER_DATA_BYTES {
        return Err(ProgramError::InvalidAccountData);
    }

    let dst = array_mut_ref![dst, 0, LOAN_INFO_HEADER_DATA_BYTES];
    let (
        acc_type_dst,
        borrower_main_acc_pubkey_dst,
        guarantor_main_acc_pubkey_dst,
        loan_approval_timestamp_dst,
        fundraising_period_ending_timestamp_dst,
        first_repayment_last_date_timestamp_dst,
        total_loan_amount_dst,
        total_amount_lended_dst,
        repaid_amount_by_borrower_dst,
        next_index_to_store_lender_data_dst,
        next_index_to_store_repayment_info_dst,
        number_of_emis_needed_to_repay_the_loan_dst,
    ) = mut_array_refs![dst, 1, 32, 32, 8, 8, 8, 8, 8, 8, 1, 1, 1];
    let LoanInfoAccDataHeader {
        acc_type,
        borrower_main_acc_pubkey,
        guarantor_main_acc_pubkey,
        loan_approval_timestamp,
        fundraising_period_ending_timestamp,
        first_repayment_last_date_timestamp,
        total_loan_amount,
        total_amount_lended,
        repaid_amount_by_borrower,
        next_index_to_store_lender_data,
        next_index_to_store_repayment_info,
        number_of_emis_needed_to_repay_the_loan,
    } = src;
    acc_type_dst[0] = acc_type;
    borrower_main_acc_pubkey_dst.copy_from_slice(borrower_main_acc_pubkey.as_ref());
    guarantor_main_acc_pubkey_dst.copy_from_slice(guarantor_main_acc_pubkey.as_ref());
    *loan_approval_timestamp_dst = loan_approval_timestamp.to_le_bytes();
    *fundraising_period_ending_timestamp_dst = fundraising_period_ending_timestamp.to_le_bytes();
    *first_repayment_last_date_timestamp_dst = first_repayment_last_date_timestamp.to_le_bytes();
    *total_loan_amount_dst = total_loan_amount.to_le_bytes();
    *total_amount_lended_dst = total_amount_lended.to_le_bytes();
    *repaid_amount_by_borrower_dst = repaid_amount_by_borrower.to_le_bytes();
    next_index_to_store_lender_data_dst[0] = next_index_to_store_lender_data;
    next_index_to_store_repayment_info_dst[0] = next_index_to_store_repayment_info;
    number_of_emis_needed_to_repay_the_loan_dst[0] = number_of_emis_needed_to_repay_the_loan;
    Ok(())
}

// each LoanInfoAccLendersData takes 45 bytes
// lenders can lend in multiples of 10 DassiCoin for prototype
// Borrowers can borrow in multiples of 100 DassiCoin for prototype
// For prototype borrowers can borrow a loan amount between 200 DassiCoin to 2000 DassiCoin
// So, total bytes needed to store LoanInfoAccLendersData 45*(2000/10) = 9000 bytes
pub struct LoanInfoAccLendersData {
    pub lender_main_acc_pubkey: Pubkey,
    pub lenders_data_storage_acc_number: u8,
    pub lender_id: u32,
    pub lent_amount: u64,
}

pub fn unpack_to_loan_info_acc_lender_data(
    input: &[u8],
) -> Result<LoanInfoAccLendersData, ProgramError> {
    const LOAN_INFO_ACC_LENDER_DATA_BYTES: usize = 45;
    if input.len() != LOAN_INFO_ACC_LENDER_DATA_BYTES {
        return Err(ProgramError::InvalidAccountData);
    }
    let input = array_ref![input, 0, LOAN_INFO_ACC_LENDER_DATA_BYTES];
    let (
        lender_main_acc_pubkey_src,
        lenders_data_storage_acc_number_src,
        lender_id_src,
        lent_amount_src,
    ) = array_refs![input, 32, 1, 4, 8];

    Ok(LoanInfoAccLendersData {
        lender_main_acc_pubkey: Pubkey::new_from_array(*lender_main_acc_pubkey_src),
        lenders_data_storage_acc_number: lenders_data_storage_acc_number_src[0],
        lender_id: u32::from_le_bytes(*lender_id_src),
        lent_amount: u64::from_le_bytes(*lent_amount_src),
    })
}

pub fn pack_to_loan_info_acc_lender_data(
    src: LoanInfoAccLendersData,
    dst: &mut [u8],
) -> Result<(), ProgramError> {
    const LOAN_INFO_ACC_LENDER_DATA_BYTES: usize = 45;
    if dst.len() != LOAN_INFO_ACC_LENDER_DATA_BYTES {
        return Err(ProgramError::InvalidAccountData);
    }

    let dst = array_mut_ref![dst, 0, LOAN_INFO_ACC_LENDER_DATA_BYTES];
    let (
        lender_main_acc_pubkey_dst,
        lenders_data_storage_acc_number_dst,
        lender_id_dst,
        lent_amount_dst,
    ) = mut_array_refs![dst, 32, 1, 4, 8];
    let LoanInfoAccLendersData {
        lender_main_acc_pubkey,
        lenders_data_storage_acc_number,
        lender_id,
        lent_amount,
    } = src;
    lender_main_acc_pubkey_dst.copy_from_slice(lender_main_acc_pubkey.as_ref());
    lenders_data_storage_acc_number_dst[0] = lenders_data_storage_acc_number;
    *lender_id_dst = lender_id.to_le_bytes();
    *lent_amount_dst = lent_amount.to_le_bytes();
    Ok(())
}


// each LoanInfoAccRepaymentData take 16 bytes
// a borrower can choose to pay a loan in maximum 50 emis, so LoanInfoAccRepaymentsData will take 16*50 = 800 bytes
pub struct LoanInfoAccRepaymentData {
    pub emi_repayment_timestamp: u64,
    pub emi_repayment_amount: u64,
}

/*

pub struct LoanInfoAccData {

    pub loan_info_acc_data_header: LoanInfoAccDataHeader,
    pub loan_info_acc_lenders_data: [LoanInfoAccLendersData],
    pub loan_info_acc_repayment_data: [LoanInfoAccRepaymentData],
}

*/

pub const LOAN_INFO_HEADER_DATA_BYTES: usize = 116;
pub const LOAN_INFO_ACC_LENDER_DATA_BYTES: usize = 45;
pub const LOAN_INFO_LENDERS_DATA_BYTES: usize = 9000;
pub const LOAN_INFO_REPAYMENT_DATA_BYTES: usize = 800;
// total bytes needed to store LoanInfoAccData = 116 + 9000 + 800 = 9916
pub const LOAN_INFO_DATA_TOTAL_BYTES: usize =
    LOAN_INFO_HEADER_DATA_BYTES + LOAN_INFO_LENDERS_DATA_BYTES + LOAN_INFO_REPAYMENT_DATA_BYTES;
pub const LOAN_INFO_ACC_DATA_TYPE_INDEX: usize = 0;
pub const LOAN_INFO_HEADER_START_INDEX: usize = 0;
pub const LOAN_INFO_HEADER_END_INDEX: usize =
    LOAN_INFO_HEADER_START_INDEX + LOAN_INFO_HEADER_DATA_BYTES;
pub const PUBKEY_DATA_BYTES: usize = 32;
pub const TIMESTAMP_DATA_BYTES: usize = 8;
pub const AMOUNT_DATA_BYTES: usize = 8;
pub const U8_DATA_BYTES: usize = 1;
pub const LOAN_INFO_BORROWER_MAIN_ACCOUNT_START_INDEX: usize = LOAN_INFO_HEADER_START_INDEX;
pub const LOAN_INFO_BORROWER_MAIN_ACCOUNT_END_INDEX: usize =
    LOAN_INFO_BORROWER_MAIN_ACCOUNT_START_INDEX + PUBKEY_DATA_BYTES;
pub const LOAN_INFO_GUARANTOR_MAIN_ACCOUNT_END_INDEX: usize =
    LOAN_INFO_BORROWER_MAIN_ACCOUNT_END_INDEX + PUBKEY_DATA_BYTES;
pub const LOAN_INFO_LOAN_APPROVAL_TIMESTAMP_EI: usize =
    LOAN_INFO_GUARANTOR_MAIN_ACCOUNT_END_INDEX + TIMESTAMP_DATA_BYTES;
pub const lOAN_INFO_FUNDRAISING_PERIOD_ENDING_TIMESTAMP_EI: usize =
    LOAN_INFO_LOAN_APPROVAL_TIMESTAMP_EI + TIMESTAMP_DATA_BYTES;
pub const LOAN_INFO_FIRST_REPAYMENT_LAST_DATE_TIMESTAMP_EI: usize =
    lOAN_INFO_FUNDRAISING_PERIOD_ENDING_TIMESTAMP_EI + TIMESTAMP_DATA_BYTES;
pub const LOAN_INFO_TOTAL_LOAN_AMOUNT_EI: usize =
    LOAN_INFO_FIRST_REPAYMENT_LAST_DATE_TIMESTAMP_EI + AMOUNT_DATA_BYTES;
pub const LOAN_INFO_TOTAL_TOTAL_AMOUNT_LENDED_EI: usize =
    LOAN_INFO_TOTAL_LOAN_AMOUNT_EI + AMOUNT_DATA_BYTES;
pub const LOAN_INFO_REPAID_AMOUNT_BY_BORROWER_EI: usize =
    LOAN_INFO_TOTAL_TOTAL_AMOUNT_LENDED_EI + AMOUNT_DATA_BYTES;
pub const LOAN_INFO_NEXT_INDEX_TO_STORE_LENDER_DATA: usize =
    LOAN_INFO_REPAID_AMOUNT_BY_BORROWER_EI + U8_DATA_BYTES;
pub const LOAN_INFO_NEXT_INDEX_TO_STORE_REPAYMENT_DATA: usize =
    LOAN_INFO_NEXT_INDEX_TO_STORE_LENDER_DATA + U8_DATA_BYTES;
pub const LOAN_INFO_NUMBER_EMIS_NEEDED_TO_REPAY_LOAN_EI: usize =
    LOAN_INFO_NEXT_INDEX_TO_STORE_REPAYMENT_DATA + U8_DATA_BYTES;

pub const LOAN_INFO_ACC_LENDERS_DATA_START_INDEX: usize = LOAN_INFO_HEADER_DATA_BYTES;
pub const LOAN_INFO_ACC_LENDERS_DATA_END_INDEX: usize =
    LOAN_INFO_ACC_LENDERS_DATA_START_INDEX + LOAN_INFO_LENDERS_DATA_BYTES;
pub const LOAN_INFO_ACC_REPAYMENT_DATA_SI: usize = LOAN_INFO_ACC_LENDERS_DATA_END_INDEX;
pub const LOAN_INFO_ACC_REPAYMENT_DATA_EI: usize =
    LOAN_INFO_ACC_LENDERS_DATA_END_INDEX + LOAN_INFO_REPAYMENT_DATA_BYTES;

// total bytes needed to store LoanInfoAccData = 116 + 9000 + 800 = 9916

// If we take size of Lenders_data_storage_acc to be 10Mb, then it can accomodate 153_846 lenders data as each lender data consumes 65 bytes
// So if in future we have a lot of users (for ex 1.6 Million lenders) then we will generate 10 Lenders_data_storage_acc address each of 10Mb,
// and we will pass all 10 accounts for every program call, and then in our program lenders_data_storage_acc_number will tell us in which account to look for a lender with lender_id
// lenders_data_storage_acc_number stores numbers from 1,2,3...

// for prototype I will use 3.25 Mb for Lenders_data_storage_acc, so it will accomodate 50_000 lenders data
// Total account size for LendersAccountData = 3_250_002 bytes
// total_lending_amount can be act as lending_score
pub struct LenderAccountData {
    pub is_account_active: u8,
    pub lender_main_acc_pubkey: Pubkey,
    pub total_lending_amount: u128,
    pub total_unique_lending_amount: u64,
    pub amount_to_withdraw_or_lend: u64,
}

pub fn unpack_to_lender_account_data(
    input: &[u8],
) -> Result<LenderAccountData, ProgramError> {
    const LENDER_ACCOUNT_DATA_BYTES: usize = 65;
    if input.len() != LENDER_ACCOUNT_DATA_BYTES {
        return Err(ProgramError::InvalidAccountData);
    }
    let input = array_ref![input, 0, LENDER_ACCOUNT_DATA_BYTES];
    let (
        is_account_active_src,
        lender_main_acc_pubkey_src,
        total_lending_amount_src,
        total_unique_lending_amount_src,
        amount_to_withdraw_or_lend_src,
    ) = array_refs![input, 1, 32, 16, 8, 8];

    Ok(LenderAccountData {
        is_account_active: is_account_active_src[0],
        lender_main_acc_pubkey: Pubkey::new_from_array(*lender_main_acc_pubkey_src),
        total_lending_amount: u128::from_le_bytes(*total_lending_amount_src),
        total_unique_lending_amount: u64::from_le_bytes(*total_unique_lending_amount_src),
        amount_to_withdraw_or_lend: u64::from_le_bytes(*amount_to_withdraw_or_lend_src),
    })
}

pub fn pack_to_lender_account_data(
    src: LenderAccountData,
    dst: &mut [u8],
) -> Result<(), ProgramError> {
    const LENDER_ACCOUNT_DATA_BYTES: usize = 65;
    if dst.len() != LENDER_ACCOUNT_DATA_BYTES {
        return Err(ProgramError::InvalidAccountData);
    }

    let dst = array_mut_ref![dst, 0, LENDER_ACCOUNT_DATA_BYTES];
    let (
        is_account_active_dst,
        lender_main_acc_pubkey_dst,
        total_lending_amount_dst,
        total_unique_lending_amount_dst,
        amount_to_withdraw_or_lend_dst,
    ) = mut_array_refs![dst, 1, 32, 16, 8, 8];
    let LenderAccountData {
        is_account_active,
        lender_main_acc_pubkey,
        total_lending_amount,
        total_unique_lending_amount,
        amount_to_withdraw_or_lend,
    } = src;
    is_account_active_dst[0] = is_account_active;
    lender_main_acc_pubkey_dst.copy_from_slice(lender_main_acc_pubkey.as_ref());
    *total_lending_amount_dst = total_lending_amount.to_le_bytes();
    *total_unique_lending_amount_dst = total_unique_lending_amount.to_le_bytes();
    *amount_to_withdraw_or_lend_dst = amount_to_withdraw_or_lend.to_le_bytes();
    Ok(())
}




pub const LENDERS_ACC_DATA_TYPE_INDEX: usize = 0;
pub const LENDER_ACC_DATA_SIZE: usize = 65;
pub const LENDERS_ACC_DATA_STARTING_INDEX: usize = 1;

pub struct LendersAccountDataArray {
    pub acc_type: u8,
    pub lenders_data_storage_acc_number: u8,
    pub lenders_acc_array_data: [LenderAccountData],
}
