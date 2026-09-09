#![no_std]

//! Treasury Management Module
//!
//! Manages member contributions, group administration, and treasury balances.
//! Handles USDC transfers and member lifecycle (add/remove).

use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, Symbol, Vec, String,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    GroupName,
    Members,
    Contributions(Address),
    TotalContributions,
    AssetAddress,
    IsActive,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ContributionRecord {
    pub member: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub period: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct GroupInfo {
    pub name: String,
    pub admin: Address,
    pub asset: Address,
    pub total_contributions: i128,
    pub member_count: u32,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct MemberSummary {
    pub address: Address,
    pub is_member: bool,
    pub total_contributed: i128,
    pub contribution_count: u32,
    pub last_period: u32,
    pub last_contributed_at: u64,
}

#[contract]
pub struct TreasuryContract;

#[contractimpl]
impl TreasuryContract {
    /// Initializes a new treasury group.
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
    /// * Initialized [`GroupInfo`].
    pub fn initialize(
        env: Env,
        admin: Address,
        group_name: String,
        asset: Address,
    ) -> GroupInfo {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::GroupName, &group_name);
        env.storage().instance().set(&DataKey::AssetAddress, &asset);
        env.storage().instance().set(&DataKey::TotalContributions, &0i128);
        env.storage().instance().set(&DataKey::IsActive, &true);
        env.storage().instance().set(&DataKey::Members, &Vec::<Address>::new(&env));

        GroupInfo {
            name: group_name,
            admin,
            asset,
            total_contributions: 0,
            member_count: 0,
            is_active: true,
        }
    }

    /// Adds a new member (admin-only).
    ///
    /// # Authorization
    /// * The `admin` must authorize and be registered admin.
    ///
    /// # Panics
    /// * If caller is not admin.
    ///
    /// # Events
    /// * Emits `member_added` with member address.
    ///
    /// # Return
    /// * None.
    pub fn add_member(env: Env, admin: Address, member: Address) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        let mut members: Vec<Address> = env
            .storage().instance()
            .get(&DataKey::Members)
            .unwrap_or(Vec::new(&env));

        if !members.contains(&member) {
            members.push_back(member.clone());
            env.storage().instance().set(&DataKey::Members, &members);
            env.events().publish(
                (Symbol::new(&env, "member_added"),),
                member,
            );
        }
    }

    /// Records a member contribution (member-only).
    ///
    /// # Authorization
    /// * The `member` must authorize.
    ///
    /// # Panics
    /// * If amount is non-positive.
    /// * If caller is not a member.
    ///
    /// # Events
    /// * None.
    ///
    /// # Return
    /// * None.
    pub fn contribute(env: Env, member: Address, amount: i128, period: u32) {
        member.require_auth();
        Self::require_member(&env, &member);

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let asset: Address = env.storage().instance().get(&DataKey::AssetAddress).unwrap();
        let token_client = token::Client::new(&env, &asset);
        token_client.transfer(&member, &env.current_contract_address(), &amount);

        let record = ContributionRecord {
            member: member.clone(),
            amount,
            timestamp: env.ledger().timestamp(),
            period,
        };

        let mut history: Vec<ContributionRecord> = env
            .storage().instance()
            .get(&DataKey::Contributions(member.clone()))
            .unwrap_or(Vec::new(&env));
        history.push_back(record);
        env.storage().instance().set(&DataKey::Contributions(member), &history);

        let total = env.storage().instance()
            .get(&DataKey::TotalContributions).unwrap_or(0i128) + amount;
        env.storage().instance().set(&DataKey::TotalContributions, &total);
    }

    /// Retrieves group information.
    ///
    /// # Authorization
    /// * None (public read-only).
    ///
    /// # Panics
    /// * Does not panic.
    ///
    /// # Events
    /// * None.
    ///
    /// # Return
    /// * Current [`GroupInfo`].
    pub fn get_group_info(env: Env) -> GroupInfo {
        let name: String = env.storage().instance().get(&DataKey::GroupName).unwrap();
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let asset: Address = env.storage().instance().get(&DataKey::AssetAddress).unwrap();
        let total: i128 = env.storage().instance().get(&DataKey::TotalContributions).unwrap();
        let members: Vec<Address> = env.storage().instance().get(&DataKey::Members).unwrap();
        let is_active: bool = env.storage().instance().get(&DataKey::IsActive).unwrap();

        GroupInfo {
            name,
            admin,
            asset,
            total_contributions: total,
            member_count: members.len() as u32,
            is_active,
        }
    }

    /// Verifies caller is admin.
    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != *caller { panic!("unauthorized admin"); }
    }

    /// Verifies caller is a member.
    fn require_member(env: &Env, caller: &Address) {
        let members: Vec<Address> = env.storage().instance().get(&DataKey::Members).unwrap();
        if !members.contains(caller) { panic!("not a member"); }
    }
}
