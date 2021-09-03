use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum DassiError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Not Rent Exempt
    #[error("Not Rent Exempt")]
    NotRentExempt,
    /// Expected Amount Mismatch
    #[error("Expected Amount Mismatch")]
    ExpectedAmountMismatch,
    /// Amount Overflow
    #[error("Amount Overflow")]
    AmountOverflow,
    /// Account Not Owned By Dassi Program
    #[error("Account Not Owned By Dassi Program")]
    WrongAccountPassed,
    /// Borrower Account Already Initialized
    #[error("Borrower Account Already Initialized")]
    BorrowerAccountAlreadyInitialized,
    /// Guarantor Account Already Initialized
    #[error("Guarantor Account Already Initialized")]
    GuarantorAccountAlreadyInitialized,
    /// Borrower Account Mismatched
    #[error("Borrower Account Mismatched")]
    BorrowerAccountMismatched,
    /// Collected Loan Funds Already Withdrawn
    #[error("Collected Loan Funds Already Withdrawn")]
    CollectedLoanFundsAlreadyWithdrawn,
    /// Lenders Storage Data Already Initialized
    #[error("Lenders Storage Data Already Initialized")]
    LendersStorageDataAlreadyInitialized,
    /// Loan Info Data Already Initialized
    #[error("Loan Info Data Already Initialized")]
    LoanInfoDataAlreadyInitialized,
    /// Loan Already Paid
    #[error("Loan Already Paid")]
    LoanAlreadyPaid,
    /// Borrower Already Have Active Loan
    #[error("Borrower Already Have Active Loan")]
    BorrowerAlreadyHaveActiveLoan,
    /// Space in use
    #[error("Space Not Empty")]
    SpaceNotEmpty,
    /// Borrower Already Funded
    #[error("Borrower Already Funded")]
    BorrowerAlreadyFunded,
    /// Expected account is not same as passed account
    #[error("Account Mismatched")]
    AccountMismatched,
    /// User Account already airdroped
    #[error("User Account already airdroped")]
    UserAlreadyAirdroped,
    /// Expected Account Type Mismatched
    #[error("Expected Account Type Mismatched")]
    ExpectedAccountTypeMismatched,
    /// Invalid Token Program Id
    #[error("Invalid Token Program Id")]
    InvalidTokenProgram,
    /// Fundraising Period Expired
    #[error("Fundraising Period Expired")]
    FundraisingPeriodExpired,
    /// Invalid Lender Id Input
    #[error("Invalid Lender Id Input")]
    InvalidLenderIdInput,
    /// Expected Lenders Account Number Not Matched
    #[error("Expected Lenders Account Number Not Matched")]
    ExpectedLendersAccNumNotMatched,
    /// Admin Does Not Matched
    #[error("Admin Does Not Matched")]
    AdminDoesNotMatched,
    /// Dassi Vault Account Does Not Matched
    #[error("Dassi Vault Account Does Not Matched")]
    DassiVaultAccountDoesNotMatched,
    ///PDA Account Does Not Matched
    #[error("PDA Account Does Not Matched")]
    PdaAccountDoesNotMatched,
    ///Data Size Not Matched
    #[error("Data Size Does Not Matched")]
    DataSizeNotMatched,
}

impl From<DassiError> for ProgramError {
    fn from(e: DassiError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
