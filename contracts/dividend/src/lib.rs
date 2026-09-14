#![no_std]

//! Dividend Distribution Module
//!
//! Manages profit distributions to cooperative members. Allows admins to distribute profits
//! proportionally based on member share weights, records distributions, and emits events.

use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, String, Symbol, Vec,
};

/// ─── TTL Constants ──────────────────────────────────────────────────────────
const LEDGERS_PER_DAY: u32 = 17_280;
const INSTANCE_TTL_THRESHOLD: u32 = 30 * LEDGERS_PER_DAY;
const INSTANCE_TTL_EXTEND_TO: u32 = 180 * LEDGERS_PER_DAY;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    AssetAddress,
    TreasuryContract,
    Distributions,
    DistributionCounter,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Distribution {
    pub id: u32,
    pub total_profit: i128,
    pub total_shares: i128,
    pub recipients: Vec<Address>,
    pub amounts: Vec<i128>,
    pub executed_at: u64,
    pub period: String,
}

#[contract]
pub struct DividendContract;

#[contractimpl]
impl DividendContract {
    /// Initializes the dividend contract with admin, asset, and treasury addresses.
    ///
    /// # Authorization
    /// * The `admin` address must authorize the transaction.
    ///
    /// # Panics
    /// * If `admin` does not authorize the transaction.
    ///
    /// # Events
    /// * None.
    ///
    /// # Return
    /// * None.
    pub fn initialize(env: Env, admin: Address, asset: Address, treasury: Address) {
        admin.require_auth();
        Self::bump_instance(&env);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::AssetAddress, &asset);
        env.storage().instance().set(&DataKey::TreasuryContract, &treasury);
        env.storage().instance().set(&DataKey::DistributionCounter, &0u32);
        env.storage().instance()
            .set(&DataKey::Distributions, &Vec::<Distribution>::new(&env));
        Self::bump_instance(&env);
    }

    /// Distributes profits to members based on their share weights.
    ///
    /// Requires equal-length `recipients` and `shares` vectors. Each member receives:
    /// `profit * (member_shares / total_shares)`.
    ///
    /// # Authorization
    /// * The `admin` must authorize and be the registered admin.
    ///
    /// # Panics
    /// * If `recipients` and `shares` lengths differ.
    /// * If `total_profit` is not positive.
    /// * If total shares sum to zero.
    /// * If caller is not admin.
    ///
    /// # Events
    /// * Emits `dividend_distributed` with distribution ID, total profit, and recipient count.
    ///
    /// # Return
    /// * ID of the created distribution.
    pub fn distribute(
        env: Env,
        admin: Address,
        recipients: Vec<Address>,
        shares: Vec<i128>,
        total_profit: i128,
        period: soroban_sdk::String,
    ) -> u32 {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        Self::bump_instance(&env);

        if recipients.len() != shares.len() {
            panic!("recipients and shares length mismatch");
        }
        if total_profit <= 0 { panic!("profit must be positive"); }

        let total_shares: i128 = shares.iter().sum();
        if total_shares == 0 { panic!("total shares cannot be zero"); }

        let asset: Address = env.storage().instance().get(&DataKey::AssetAddress).unwrap();
        let token_client = token::Client::new(&env, &asset);

        let mut amounts: Vec<i128> = Vec::new(&env);
        for i in 0..recipients.len() {
            let member_shares = shares.get(i).unwrap();
            let payout = (total_profit * member_shares) / total_shares;
            if payout > 0 {
                token_client.transfer(
                    &env.current_contract_address(),
                    &recipients.get(i).unwrap(),
                    &payout,
                );
            }
            amounts.push_back(payout);
        }

        let counter: u32 = env.storage().instance()
            .get(&DataKey::DistributionCounter).unwrap_or(0);
        let id = counter + 1;

        let dist = Distribution {
            id,
            total_profit,
            total_shares,
            recipients: recipients.clone(),
            amounts: amounts.clone(),
            executed_at: env.ledger().timestamp(),
            period,
        };

        let mut distributions: Vec<Distribution> = env.storage().instance()
            .get(&DataKey::Distributions).unwrap_or(Vec::new(&env));
        distributions.push_back(dist);
        env.storage().instance().set(&DataKey::Distributions, &distributions);
        env.storage().instance().set(&DataKey::DistributionCounter, &id);

        env.events().publish(
            (Symbol::new(&env, "dividend_distributed"),),
            (id, total_profit, recipients.len()),
        );
        id
    }

    /// Retrieves all recorded distributions.
    ///
    /// # Authorization
    /// * None (public read-only access).
    ///
    /// # Panics
    /// * Does not panic; returns empty vector if none exist.
    ///
    /// # Events
    /// * None.
    ///
    /// # Return
    /// * Vector of all [`Distribution`] records.
    pub fn get_distributions(env: Env) -> Vec<Distribution> {
        env.storage().instance()
            .get(&DataKey::Distributions)
            .unwrap_or(Vec::new(&env))
    }

    /// Verifies caller is the registered admin.
    ///
    /// # Panics
    /// * If caller is not admin.
    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != *caller { panic!("unauthorized"); }
    }
}
