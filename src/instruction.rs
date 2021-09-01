use crate::error::DassiError::InvalidInstruction;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

pub enum DassiInstruction {
    /// Accounts Expected:
    ///
    /// 0. `[signer]` Lender Main Account
    /// 1. `[writable]` Lender DassiCoin ATA To Debit
    /// 2. `[writable]` DassiCoin Vault Account
    /// 3. `[]` Token Program
    /// 4. `[writable]` Loan Info Storage Account
    /// 5. `[writable]` Lenders Account Data Storage Account
    LendToBorrower {
        amount_to_lend_input: u64,
        lender_id_input: u32,
    },

    /// 0. `[signer]` Lender Main Account
    /// 1. `[writable]` Lender DassiCoin ATA To Credit
    /// 2. `[writable]` DassiCoin Vault Account
    /// 3. `[writable]` Lenders Account Data Storage Account
    /// 4. `[]` Token Program
    /// 5. `[]` The PDA Account ("DassiFinance")
    WithdrawLenderFreeWalletFunds { lender_id_input: u32 },

    /// 0. `[signer]` Borrower Main Account
    /// 1. `[writable]` Borrower DassiCoin ATA To Credit
    /// 2. `[writable]` DassiCoin Vault Account
    /// 3. `[]` Token Program
    /// 4. `[writable]` Loan Info Storage Account
    /// 5. `[]` The PDA Account ("DassiFinance")
    WithdrawCollectedLoanFunds {},

    /// 0. `[signer]` Initializer Account
    /// 1. `[writable]` DassiCoin Vault Account
    /// 2. `[]` Token Program
    TransferDassiVaultAccountOwnership {},

    /// 0. `[signer]` Fee Payer
    /// 1. `[writable]` Lenders Account Data Storage Account
    InitializeLendersStorageAccount {},

    /// 0. `[signer]` Guarantor Main Account
    /// 1. `[writable]` Guarantor Storage Account (seed: "DassiFinanceGuarantor")
    InitializeGuarantorAccount {},

    /// 0. `[signer]` Borrower Main Account
    /// 1. `[writable]` Borrower Storage Account (seed: "DassiFinanceBorrower")
    InitializeBorrowerAccount {},

    /// 0. `[signer]` Borrower Main Account
    /// 1. `[writable]` Borrower Dassi ATA To Debit
    /// 2. `[writable]` DassiCoin Vault Account
    /// 3. `[writable]` Borrower Storage Account (seed: "DassiFinanceBorrower")
    /// 4. `[]` Token Program
    /// 5. `[writable]` Loan Info Storage Account
    /// 6. `[writable]` Lenders Account Data Storage Account
    PayEMIforLoan { emi_amount_to_pay_input: u64 },

    /// 0. `[signer]` Guarantor Main Account
    /// 1. `[]` Borrower Main Account
    /// 2. `[writable]` Loan Info Storage Account
    /// 3. `[writable]` Borrower Storage Account
    InitializeLoanInfoAccount {
        num_days_left_for_first_repayment_input: u16,
        num_emis_needed_to_repay_the_loan_input: u16,
        num_days_for_fundraising_input: u16,
        total_loan_amount_input: u64,
    },

    /// 0. `[signer]` Fee Payer ( Airdrop User )
    /// 1. `[writable]` Dassi Coin Airdrop User Storage Account (seed: "DassiFinanceAirdrop")
    /// 2. `[writable]` User DassiCoin Token Associated Account To Credit 250 DassiCoin (max possible: 2500 DassiCoin)
    /// 3. `[writable]` Airdrop Vault DassiCoin Account
    /// 4. `[]` Token Program
    /// 5. `[]` The PDA Account ("DassiFinanceAirdrop")
    AirdropUsersWithDassiTestCoins {},

    /// 0. `[signer]` Initializer Account
    /// 1. `[writable]` DassiCoin Airdrop Vault Account (pda: "DassiFinanceAirdrop")
    /// 3. `[]` Token Program
    TransferAirdropVaultAccountOwnership {},


    ReturnFundsToLenders { num_accounts_input: u16 },

  
    CloseLoanInfoAccount {},
}


impl DassiInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        msg!("In {}", input.len());

        Ok(match input[0] {
            0 => Self::LendToBorrower {
                amount_to_lend_input: Self::unpack_to_u64(&input[1..9])?,
                lender_id_input: Self::unpack_to_u32(&input[9..13]),
            },
            1 => Self::WithdrawLenderFreeWalletFunds {
                lender_id_input: Self::unpack_to_u32(&input[1..5]),
            },
            2 => Self::WithdrawCollectedLoanFunds {},

            3 => Self::TransferDassiVaultAccountOwnership {},

            4 => Self::InitializeLendersStorageAccount {},

            5 => Self::InitializeBorrowerAccount {},

            6 => Self::InitializeGuarantorAccount {},

            7 => Self::PayEMIforLoan {
                emi_amount_to_pay_input: Self::unpack_to_u64(&input[1..9])?,
            },

            8 => Self::InitializeLoanInfoAccount {
                num_days_left_for_first_repayment_input: Self::unpack_to_u16(&input[1..3]),
                num_emis_needed_to_repay_the_loan_input: Self::unpack_to_u16(&input[3..5]),
                num_days_for_fundraising_input: Self::unpack_to_u16(&input[5..7]),
                total_loan_amount_input: Self::unpack_to_u64(&input[7..15])?,
            },
            9 => Self::AirdropUsersWithDassiTestCoins {},
            10 => Self::TransferAirdropVaultAccountOwnership {},

            11 => Self::ReturnFundsToLenders {
                num_accounts_input: Self::unpack_to_u16(&input[1..3]),
            },

            12 => Self::CloseLoanInfoAccount {},

            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_to_u64(input: &[u8]) -> Result<u64, ProgramError> {
        msg!("in unpack");
        let value = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        msg!("unpack value {}", value);
        Ok(value)
    }

    fn unpack_to_u16(input: &[u8]) -> u16 {
        (input[0] as u16) | (input[1] as u16) << 8
    }

    pub fn unpack_to_u32(input: &[u8]) -> u32 {
        let amount = (input[0] as u32)
            | (input[1] as u32) << 8
            | (input[2] as u32) << 16
            | (input[3] as u32) << 24;
        msg!("u32 unpack amount {}", amount);
        return amount;
    }
}
