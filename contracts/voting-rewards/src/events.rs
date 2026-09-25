use soroban_sdk::{Address, BytesN, Env, Symbol};

pub const EPOCH_STARTED_TOPIC: &str = "EpochStarted";
pub const EPOCH_ROOT_PUBLISHED_TOPIC: &str = "EpochRootPublished";
pub const REWARD_CLAIMED_TOPIC: &str = "RewardClaimed";
pub const POOL_FUNDED_TOPIC: &str = "PoolFunded";
pub const ADMIN_SET_TOPIC: &str = "AdminSet";
pub const EPOCH_DURATION_UPDATED_TOPIC: &str = "EpochDurationUpdated";

pub fn emit_epoch_started(env: &Env, epoch_id: u64, start_ledger: u32, end_ledger: u32) {
    env.events().publish(
        (Symbol::new(env, EPOCH_STARTED_TOPIC), epoch_id),
        (start_ledger, end_ledger),
    );
}

pub fn emit_epoch_root_published(
    env: &Env,
    epoch_id: u64,
    merkle_root: &BytesN<32>,
    total_reward_amount: i128,
) {
    env.events().publish(
        (Symbol::new(env, EPOCH_ROOT_PUBLISHED_TOPIC), epoch_id),
        (merkle_root.clone(), total_reward_amount),
    );
}

pub fn emit_reward_claimed(env: &Env, epoch_id: u64, claimant: &Address, amount: i128) {
    env.events().publish(
        (Symbol::new(env, REWARD_CLAIMED_TOPIC), claimant.clone()),
        (epoch_id, amount),
    );
}

pub fn emit_pool_funded(env: &Env, funder: &Address, amount: i128) {
    env.events().publish(
        (Symbol::new(env, POOL_FUNDED_TOPIC), funder.clone()),
        amount,
    );
}

pub fn emit_admin_set(env: &Env, new_admin: &Address) {
    env.events().publish(
        (Symbol::new(env, ADMIN_SET_TOPIC), new_admin.clone()),
        (),
    );
}

pub fn emit_epoch_duration_updated(env: &Env, new_duration_ledgers: u32) {
    env.events().publish(
        Symbol::new(env, EPOCH_DURATION_UPDATED_TOPIC),
        new_duration_ledgers,
    );
}
