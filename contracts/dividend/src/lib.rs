#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, String, Symbol, Vec,
};

/// ─── Storage TTL ─────────────────────────────────────────────────────────────
///
/// Soroban charges rent on stored entries and evicts them once their TTL
/// elapses unless bumped. Distribution history lives in instance storage, so it
/// is extended at the start of every state-changing call. Stellar closes a
/// ledger roughly every 5 seconds (one day ≈ 17_280 ledgers); instance state is
/// kept alive for ~30 days, with the threshold one day below the target so a
/// bump only pays rent when the entry is within a day of expiry.
const DAY_IN_LEDGERS: u32 = 17_280;
const INSTANCE_BUMP_LEDGERS: u32 = 30 * DAY_IN_LEDGERS;
const INSTANCE_TTL_THRESHOLD: u32 = INSTANCE_BUMP_LEDGERS - DAY_IN_LEDGERS;

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
    pub fn initialize(env: Env, admin: Address, asset: Address, treasury: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::AssetAddress, &asset);
        env.storage().instance().set(&DataKey::TreasuryContract, &treasury);
        env.storage().instance().set(&DataKey::DistributionCounter, &0u32);
        env.storage().instance()
            .set(&DataKey::Distributions, &Vec::<Distribution>::new(&env));
        Self::bump_instance(&env);
    }

    /// Distribute profit proportionally based on each member's share weight.
    ///
    /// `recipients` and `shares` must be equal length.
    /// Each member receives: `profit * (member_shares / total_shares)`
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

    pub fn get_distributions(env: Env) -> Vec<Distribution> {
        env.storage().instance()
            .get(&DataKey::Distributions)
            .unwrap_or(Vec::new(&env))
    }

    /// Extend the contract's instance-storage TTL. Called at the start of every
    /// state-changing entrypoint so distribution history is never evicted.
    fn bump_instance(env: &Env) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_BUMP_LEDGERS);
    }

    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != *caller { panic!("unauthorized"); }
    }
}
