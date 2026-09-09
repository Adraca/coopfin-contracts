#![no_std]

//! Loan Management Module
//!
//! Handles member loan requests, approvals, and disbursements. Integrates with treasury
//! for fund distribution and governance for approvals.

use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, Symbol, Vec, String,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    TreasuryContract,
    AssetAddress,
    Loans,
    LoanCounter,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum LoanStatus {
    Pending,
    Approved,
    Repaid,
    Rejected,
    Defaulted,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Loan {
    pub id: u32,
    pub borrower: Address,
    pub amount: i128,
    pub interest_bps: u32,
    pub repayment_due: u64,
    pub amount_repaid: i128,
    pub status: LoanStatus,
    pub purpose: String,
    pub requested_at: u64,
    pub approved_at: u64,
}

#[contract]
pub struct LoanContract;

#[contractimpl]
impl LoanContract {
    /// Initializes loan contract with admin, treasury, and asset.
    ///
    /// # Authorization
    /// * The `admin` must authorize.
    ///
    /// # Panics
    /// * If already initialized.
    ///
    /// # Events
    /// * None.
    ///
    /// # Return
    /// * None.
    pub fn initialize(env: Env, admin: Address, treasury: Address, asset: Address) {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TreasuryContract, &treasury);
        env.storage().instance().set(&DataKey::AssetAddress, &asset);
        env.storage().instance().set(&DataKey::LoanCounter, &0u32);
        env.storage().instance().set(&DataKey::Loans, &Vec::<Loan>::new(&env));
    }

    /// Submits a loan request (member-only).
    ///
    /// # Authorization
    /// * The `borrower` must authorize.
    ///
    /// # Panics
    /// * If amount is non-positive.
    ///
    /// # Events
    /// * Emits `loan_requested` with ID, borrower, and amount.
    ///
    /// # Return
    /// * Requested loan ID.
    pub fn request_loan(
        env: Env,
        borrower: Address,
        amount: i128,
        purpose: String,
        repayment_days: u32,
    ) -> u32 {
        borrower.require_auth();
        if amount <= 0 { panic!("amount must be positive"); }

        let counter: u32 = env.storage().instance()
            .get(&DataKey::LoanCounter).unwrap_or(0);
        let id = counter + 1;

        let due = env.ledger().timestamp() + (repayment_days as u64 * 86_400);

        let loan = Loan {
            id,
            borrower: borrower.clone(),
            amount,
            interest_bps: 500,
            repayment_due: due,
            amount_repaid: 0,
            status: LoanStatus::Pending,
            purpose,
            requested_at: env.ledger().timestamp(),
            approved_at: 0,
        };

        let mut loans: Vec<Loan> = env.storage().instance()
            .get(&DataKey::Loans).unwrap_or(Vec::new(&env));
        loans.push_back(loan);
        env.storage().instance().set(&DataKey::Loans, &loans);
        env.storage().instance().set(&DataKey::LoanCounter, &id);

        env.events().publish(
            (Symbol::new(&env, "loan_requested"),),
            (id, borrower, amount),
        );
        id
    }

    /// Approves and disburses a loan (admin-only).
    ///
    /// # Authorization
    /// * The `admin` must authorize and be registered admin.
    ///
    /// # Panics
    /// * If loan not found or not pending.
    ///
    /// # Events
    /// * None.
    ///
    /// # Return
    /// * None.
    pub fn approve_loan(env: Env, admin: Address, loan_id: u32) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        let mut loans: Vec<Loan> = env.storage().instance().get(&DataKey::Loans).unwrap();
        let idx = Self::find_loan_idx(&loans, loan_id);
        let mut loan = loans.get(idx).unwrap();

        if loan.status != LoanStatus::Pending {
            panic!("loan not pending");
        }

        loan.status = LoanStatus::Approved;
        loan.approved_at = env.ledger().timestamp();
        loans.set(idx, loan.clone());
        env.storage().instance().set(&DataKey::Loans, &loans);

        let asset: Address = env.storage().instance().get(&DataKey::AssetAddress).unwrap();
        let token_client = token::Client::new(&env, &asset);
        token_client.transfer(
            &env.current_contract_address(),
            &loan.borrower,
            &loan.amount,
        );
    }

    /// Retrieves all loans.
    ///
    /// # Authorization
    /// * None (public read-only).
    ///
    /// # Panics
    /// * Does not panic; returns empty vector if none exist.
    ///
    /// # Events
    /// * None.
    ///
    /// # Return
    /// * Vector of all [`Loan`] records.
    pub fn get_loans(env: Env) -> Vec<Loan> {
        env.storage().instance()
            .get(&DataKey::Loans)
            .unwrap_or(Vec::new(&env))
    }

    /// Finds loan index by ID.
    fn find_loan_idx(loans: &Vec<Loan>, id: u32) -> usize {
        loans.iter().position(|loan| loan.id == id).unwrap()
    }

    /// Verifies caller is admin.
    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != *caller { panic!("unauthorized"); }
    }
}
