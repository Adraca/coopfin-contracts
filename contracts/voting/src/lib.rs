#![no_std]

//! Voting Module
//!
//! Manages governance proposals and member voting. Handles proposal lifecycle
//! (creation/voting) and integrates with treasury for membership validation.

use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, Map, Symbol, Vec, String,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    TreasuryContract,
    Proposals,
    ProposalCounter,
    Votes(u32),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Failed,
    Executed,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalType {
    LoanApproval,
    TreasurySpend,
    AddMember,
    RemoveMember,
    UpdateRule,
    General,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Proposal {
    pub id: u32,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub votes_for: u32,
    pub votes_against: u32,
    pub quorum: u32,
    pub deadline: u64,
    pub status: ProposalStatus,
    pub created_at: u64,
    pub payload: String,
}

#[contract]
pub struct VotingContract;

#[contractimpl]
impl VotingContract {
    /// Initializes voting contract with admin and treasury.
    ///
    /// # Authorization
    /// * The `admin` must authorize.
    ///
    /// # Panics
    /// * If `admin` does not authorize.
    ///
    /// # Events
    /// * None.
    ///
    /// # Return
    /// * None.
    pub fn initialize(env: Env, admin: Address, treasury: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TreasuryContract, &treasury);
        env.storage().instance().set(&DataKey::ProposalCounter, &0u32);
        env.storage().instance().set(&DataKey::Proposals, &Vec::<Proposal>::new(&env));
    }

    /// Creates a new proposal (member-only).
    ///
    /// # Authorization
    /// * The `proposer` must authorize.
    ///
    /// # Panics
    /// * If proposer is not authorized.
    ///
    /// # Events
    /// * Emits `proposal_created` with ID and proposer.
    ///
    /// # Return
    /// * Created proposal ID.
    pub fn create_proposal(
        env: Env,
        proposer: Address,
        proposal_type: ProposalType,
        title: String,
        description: String,
        voting_days: u32,
        quorum: u32,
        payload: String,
    ) -> u32 {
        proposer.require_auth();

        let counter: u32 = env.storage().instance()
            .get(&DataKey::ProposalCounter).unwrap_or(0);
        let id = counter + 1;

        let deadline = env.ledger().timestamp() + (voting_days as u64 * 86_400);

        let proposal = Proposal {
            id,
            proposer: proposer.clone(),
            proposal_type,
            title,
            description,
            votes_for: 0,
            votes_against: 0,
            quorum,
            deadline,
            status: ProposalStatus::Active,
            created_at: env.ledger().timestamp(),
            payload,
        };

        let mut proposals: Vec<Proposal> = env.storage().instance()
            .get(&DataKey::Proposals).unwrap_or(Vec::new(&env));
        proposals.push_back(proposal);
        env.storage().instance().set(&DataKey::Proposals, &proposals);
        env.storage().instance().set(&DataKey::ProposalCounter, &id);

        env.storage().persistent()
            .set(&DataKey::Votes(id), &Map::<Address, bool>::new(&env));

        env.events().publish(
            (Symbol::new(&env, "proposal_created"),),
            (id, proposer),
        );
        id
    }

    /// Casts a vote on a proposal (member-only).
    ///
    /// # Authorization
    /// * The `voter` must authorize.
    ///
    /// # Panics
    /// * If proposal is not active or deadline passed.
    ///
    /// # Events
    /// * None.
    ///
    /// # Return
    /// * None.
    pub fn vote(env: Env, voter: Address, proposal_id: u32, approve: bool) {
        voter.require_auth();

        let mut proposals: Vec<Proposal> = env.storage().instance()
            .get(&DataKey::Proposals).unwrap();
        let idx = Self::find_proposal_idx(&proposals, proposal_id);
        let mut proposal = proposals.get(idx).unwrap();

        if proposal.status != ProposalStatus::Active {
            panic!("proposal not active");
        }
        if env.ledger().timestamp() > proposal.deadline {
            panic!("voting period ended");
        }

        let mut votes: Map<Address, bool> = env.storage().persistent()
            .get(&DataKey::Votes(proposal_id)).unwrap();
        if votes.contains_key(&voter) {
            panic!("already voted");
        }
        votes.set(voter, approve);
        env.storage().persistent().set(&DataKey::Votes(proposal_id), &votes);

        if approve {
            proposal.votes_for += 1;
        } else {
            proposal.votes_against += 1;
        }
        proposals.set(idx, proposal);
        env.storage().instance().set(&DataKey::Proposals, &proposals);
    }

    /// Retrieves all proposals.
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
    /// * Vector of all [`Proposal`] records.
    pub fn get_proposals(env: Env) -> Vec<Proposal> {
        env.storage().instance()
            .get(&DataKey::Proposals)
            .unwrap_or(Vec::new(&env))
    }

    /// Finds proposal index by ID.
    fn find_proposal_idx(proposals: &Vec<Proposal>, id: u32) -> usize {
        proposals.iter().position(|p| p.id == id).unwrap()
    }
}
