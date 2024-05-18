#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, log, symbol_short, token, Address, Env, Symbol, Vec,
};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Init,
    Balance,
}

#[derive(Clone)]
#[contracttype]
pub enum TimeBoundKind {
    Before,
    After,
}

#[derive(Clone)]
#[contracttype]
pub struct TimeBound {
    pub kind: TimeBoundKind,
    pub timestamp: u64,
}


const COUNTER: Symbol = symbol_short!("COUNTER");

#[contract]
pub struct CrowdFunding;


#[derive(Clone)]
#[contracttype]
pub struct Campaign {
    pub creator: Address,
    pub goal: u64,
    pub deadline: u64,
    pub funds_raised: u64,
    pub contributors: Vec<Address>,
    pub contributed_amount: Vec<u64>,
}


#[contractimpl]
impl CrowdFunding {
    pub fn increment(env: Env) -> u64 {
        // Get the current count.
        let mut count: u64 = env.storage().instance().get(&COUNTER).unwrap_or(0); // If no value set, assume 0.
        log!(&env, "count: {}", count);

        // Increment the count.
        count += 1;

        // Save the count.
        env.storage().instance().set(&COUNTER, &count);

        // The contract instance will be bumped to have a lifetime of at least 100 ledgers if the current expiration lifetime at most 50.
        // If the lifetime is already more than 100 ledgers, this is a no-op. Otherwise,
        // the lifetime is extended to 100 ledgers. This lifetime bump includes the contract
        // instance itself and all entries in storage().instance(), i.e, COUNTER.
        env.storage().instance().extend_ttl(50, 100);

        // Return the count to the caller.
        count
    }

    pub fn create_campaign(
        env: Env,
        creator: Address,
        contributors: Vec<Address>,
        contributed_amount: Vec<u64>,
        goal: u64,
        deadline: u64,
    ) -> u64 {

        let campaign_id: u64 = Self::increment(env.clone()).try_into().unwrap();
        env.storage().instance().set(
            &campaign_id,
            &Campaign {
                creator: creator.clone(),
                goal,
                deadline,
                funds_raised: 0,
                contributors,
                contributed_amount,
            },
        );
        campaign_id
    }

    pub fn contribute(
        env: Env,
        from: Address,
        // token: Address,
        amount: u64,
        campaign_id: Vec<Address>,
    ) {
        let mut campaign: Campaign = env.storage().instance().get(&campaign_id.clone()).unwrap();
        campaign.funds_raised += amount;
        campaign.contributors.push_back(from);
        campaign.contributed_amount.push_back(amount);

        env.storage().instance().set(&campaign_id, &campaign);
    }

    pub fn withdraw(env: Env, user: Address, campaign_id: u64) {
        // Check if the caller is the campaign creator
        user.require_auth();
        let campaign: Campaign = env
            .storage()
            .instance()
            .get(&campaign_id)
            .unwrap_or_else(|| panic!("Campaign not found"));


        // Check if the deadline has been exceeded
        let current_time = env.ledger().timestamp();
        if current_time > campaign.deadline {
            // Deadline has been exceeded
    
            // Check if the funds_raised is greater than or equal to the goal
            if campaign.funds_raised >= campaign.goal {
                // Funds raised are sufficient, allow withdrawal
    
                // Transfer the funds_raised amount to the campaign creator
                token::Client::new(&env, &campaign.creator)
                    .transfer(&env.current_contract_address(), &user, &(campaign.funds_raised as i128));
    
                // Remove the campaign from storage
                env.storage().instance().remove(&campaign_id);
            } else {
                panic!("Funds raised are less than the goal");
            }
        } else {
            panic!("Deadline has not been exceeded yet");
        }
    }

    pub fn auto_refund(env: Env, campaign_id: u64) {
        let campaign: Campaign = env
            .storage()
            .instance()
            .get(&campaign_id)
            .unwrap_or_else(|| panic!("Campaign not found"));
    
        // Check if the deadline has been exceeded
        let current_time = env.ledger().timestamp();
        if current_time > campaign.deadline {
            // Deadline has been exceeded
    
            // Check if the funds_raised is less than the goal
            if campaign.funds_raised < campaign.goal {
                // Goal not met, refund contributors
    
                for (contributor, amount) in campaign.contributors.iter().zip(campaign.contributed_amount.iter()) {
                    // Transfer the contributed amount back to each contributor
                    token::Client::new(&env, &campaign.creator)
                        .transfer(&env.current_contract_address(), &contributor, &(amount as i128));
                }
    
                // Remove the campaign from storage
                env.storage().instance().remove(&campaign_id);
            }
        }
    }
}



