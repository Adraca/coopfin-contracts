#![no_std]

//! Governance Module
//!
//! Manages cooperative rules, voting parameters, and administrative privileges.
//! Enables rule updates and retrieval via admin-controlled functions.

use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, Symbol,
};

/// ─── TTL Constants ──────────────────────────────────────────────────────────
const LEDGERS_PER_DAY: u32 = 17_280;
const INSTANCE_TTL_THRESHOLD: u32 = 30 * LEDGERS_PER_DAY;
const INSTANCE_TTL_EXTEND_TO: u32 = 180 * LEDGERS_PER_DAY;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    VotingContract,
    LoanContract,
    TreasuryContract,
    Rules,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct CoopRules {
    pub min_contribution: i128,
    pub contribution_period_days: u32,
    pub max_loan_multiplier: u32,
    pub loan_interest_bps: u32,
    pub voting_quorum: u32,
    pub voting_period_days: u32,
    pub late_penalty_bps: u32,
}

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    /// Initializes governance contracts and sets default rules.
    ///
    /// # Authorization
    /// * The `admin` must authorize the transaction.
    ///
    /// # Panics
    /// * If `admin` does not authorize.
    ///
    /// # Events
    /// * None.
    ///
    /// # Return
    /// * None.
    pub fn initialize(
        env: Env,
        admin: Address,
        voting: Address,
        loan: Address,
        treasury: Address,
    ) {
        admin.require_auth();
        Self::bump_instance(&env);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::VotingContract, &voting);
        env.storage().instance().set(&DataKey::LoanContract, &loan);
        env.storage().instance().set(&DataKey::TreasuryContract, &treasury);

        let rules = CoopRules {
            min_contribution: 10_000_000,
            contribution_period_days: 30,
            max_loan_multiplier: 3,
            loan_interest_bps: 500,
            voting_quorum: 3,
            voting_period_days: 7,
            late_penalty_bps: 200,
        };
        env.storage().instance().set(&DataKey::Rules, &rules);
    }

    /// Updates cooperative rules (admin-only).
    ///
    /// # Authorization
    /// * The `admin` must authorize and be registered admin.
    ///
    /// # Panics
    /// * If caller is not admin.
    ///
    /// # Events
    /// * Emits `rules_updated`.
    ///
    /// # Return
    /// * None.
    pub fn update_rules(env: Env, admin: Address, rules: CoopRules) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        Self::bump_instance(&env);
        env.storage().instance().set(&DataKey::Rules, &rules);
        env.events().publish((Symbol::new(&env, "rules_updated"),), ());
    }

    /// Retrieves current cooperative rules.
    ///
    /// # Authorization
    /// * None (public read-only).
    ///
    /// # Panics
    /// * If rules not initialized.
    ///
    /// # Events
    /// * None.
    ///
    /// # Return
    /// * Current [`CoopRules`].
    pub fn get_rules(env: Env) -> CoopRules {
        env.storage().instance().get(&DataKey::Rules).unwrap()
    }

    /// Verifies caller is admin.
    ///
    /// # Panics
    /// * If caller is not admin.
    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != *caller { panic!("unauthorized"); }
    }
}
