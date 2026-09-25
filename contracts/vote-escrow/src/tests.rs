extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token, Env,
};

const MIN_LOCK_DURATION: u32 = 100;
const MAX_LOCK_DURATION: u32 = 10_000;
const MAX_MULTIPLIER_BPS: u32 = 25_000; // 2.5x multiplier at max duration

struct Fixture {
    env: Env,
    contract_id: Address,
    admin: Address,
    token: Address,
    user: Address,
    user2: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = Address::generate(&env);
    let contract_id = env.register(VoteEscrowContract, ());

    // Initialize contract
    env.as_contract(&contract_id, || {
        VoteEscrowContract::initialize(
            env.clone(),
            admin.clone(),
            token.clone(),
            MIN_LOCK_DURATION,
            MAX_LOCK_DURATION,
            MAX_MULTIPLIER_BPS,
        );
    });

    Fixture {
        env,
        contract_id,
        admin,
        token,
        user,
        user2,
    }
}

#[test]
fn test_initialize() {
    let f = setup();
    f.env.as_contract(&f.contract_id, || {
        let stored_token: Address = f
            .env
            .storage()
            .instance()
            .get(&DataKey::LockedToken)
            .unwrap();
        assert_eq!(stored_token, f.token);

        let stored_min: u32 = f
            .env
            .storage()
            .instance()
            .get(&DataKey::MinLockDuration)
            .unwrap();
        assert_eq!(stored_min, MIN_LOCK_DURATION);

        let stored_max: u32 = f
            .env
            .storage()
            .instance()
            .get(&DataKey::MaxLockDuration)
            .unwrap();
        assert_eq!(stored_max, MAX_LOCK_DURATION);

        let total_locked: i128 = f
            .env
            .storage()
            .instance()
            .get(&DataKey::TotalLocked)
            .unwrap_or(0);
        assert_eq!(total_locked, 0);
    });
}

#[test]
fn test_create_lock_basic() {
    let f = setup();
    let amount = 1_000;
    let duration = 1_000;

    f.env.as_contract(&f.contract_id, || {
        let lock = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        assert_eq!(lock.owner, f.user);
        assert_eq!(lock.amount, amount);
        assert_eq!(lock.withdrawn, false);
        assert!(lock.initial_voting_power >= amount);
    });
}

#[test]
fn test_create_lock_updates_total() {
    let f = setup();
    let amount = 1_000;
    let duration = 1_000;

    f.env.as_contract(&f.contract_id, || {
        VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        let total_locked: i128 = f
            .env
            .storage()
            .instance()
            .get(&DataKey::TotalLocked)
            .unwrap_or(0);
        assert_eq!(total_locked, amount);
    });
}

#[test]
fn test_create_lock_min_duration() {
    let f = setup();
    let amount = 1_000;
    let duration = MIN_LOCK_DURATION;

    f.env.as_contract(&f.contract_id, || {
        let lock = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        // At minimum duration, voting power should equal amount
        assert_eq!(lock.initial_voting_power, amount);
    });
}

#[test]
fn test_create_lock_max_duration() {
    let f = setup();
    let amount = 1_000;
    let duration = MAX_LOCK_DURATION;

    f.env.as_contract(&f.contract_id, || {
        let lock = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        // At maximum duration with 2.5x multiplier, voting power should be 3.5x amount
        // (amount + 2.5x boost)
        assert_eq!(lock.initial_voting_power, amount * 350 / 100);
    });
}

#[test]
fn test_create_lock_insufficient_duration() {
    let f = setup();
    let amount = 1_000;
    let duration = MIN_LOCK_DURATION - 1;

    f.env.as_contract(&f.contract_id, || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            VoteEscrowContract::create_lock(
                f.env.clone(),
                f.user.clone(),
                amount,
                duration,
            );
        }));
        assert!(result.is_err());
    });
}

#[test]
fn test_create_lock_excessive_duration() {
    let f = setup();
    let amount = 1_000;
    let duration = MAX_LOCK_DURATION + 1;

    f.env.as_contract(&f.contract_id, || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            VoteEscrowContract::create_lock(
                f.env.clone(),
                f.user.clone(),
                amount,
                duration,
            );
        }));
        assert!(result.is_err());
    });
}

#[test]
fn test_create_lock_invalid_amount() {
    let f = setup();
    let duration = 1_000;

    f.env.as_contract(&f.contract_id, || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            VoteEscrowContract::create_lock(
                f.env.clone(),
                f.user.clone(),
                0,
                duration,
            );
        }));
        assert!(result.is_err());
    });
}

#[test]
fn test_increase_lock_amount() {
    let f = setup();
    let amount = 1_000;
    let additional_amount = 500;
    let duration = 1_000;

    f.env.as_contract(&f.contract_id, || {
        let lock1 = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        let lock2 = VoteEscrowContract::increase_lock_amount(
            f.env.clone(),
            f.user.clone(),
            additional_amount,
        );

        assert_eq!(lock2.amount, amount + additional_amount);
        assert!(lock2.initial_voting_power > lock1.initial_voting_power);
    });
}

#[test]
fn test_increase_lock_amount_updates_total() {
    let f = setup();
    let amount = 1_000;
    let additional_amount = 500;
    let duration = 1_000;

    f.env.as_contract(&f.contract_id, || {
        VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        VoteEscrowContract::increase_lock_amount(
            f.env.clone(),
            f.user.clone(),
            additional_amount,
        );

        let total_locked: i128 = f
            .env
            .storage()
            .instance()
            .get(&DataKey::TotalLocked)
            .unwrap_or(0);
        assert_eq!(total_locked, amount + additional_amount);
    });
}

#[test]
fn test_increase_lock_amount_invalid_amount() {
    let f = setup();
    let amount = 1_000;
    let duration = 1_000;

    f.env.as_contract(&f.contract_id, || {
        VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            VoteEscrowContract::increase_lock_amount(
                f.env.clone(),
                f.user.clone(),
                0,
            );
        }));
        assert!(result.is_err());
    });
}

#[test]
fn test_increase_lock_amount_non_existent_lock() {
    let f = setup();
    let additional_amount = 500;

    f.env.as_contract(&f.contract_id, || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            VoteEscrowContract::increase_lock_amount(
                f.env.clone(),
                f.user.clone(),
                additional_amount,
            );
        }));
        assert!(result.is_err());
    });
}

#[test]
fn test_extend_lock() {
    let f = setup();
    let amount = 1_000;
    let duration = 1_000;
    let new_duration = 2_000;

    f.env.as_contract(&f.contract_id, || {
        let lock1 = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        let new_end_ledger = lock1.end_ledger.checked_add(1_000).unwrap();
        let lock2 = VoteEscrowContract::extend_lock(
            f.env.clone(),
            f.user.clone(),
            new_end_ledger,
        );

        assert_eq!(lock2.end_ledger, new_end_ledger);
        assert!(lock2.initial_voting_power > lock1.initial_voting_power);
    });
}

#[test]
fn test_extend_lock_invalid_end_ledger() {
    let f = setup();
    let amount = 1_000;
    let duration = 1_000;

    f.env.as_contract(&f.contract_id, || {
        let lock = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            VoteEscrowContract::extend_lock(
                f.env.clone(),
                f.user.clone(),
                lock.end_ledger,
            );
        }));
        assert!(result.is_err());
    });
}

#[test]
fn test_extend_lock_to_min_duration() {
    let f = setup();
    let amount = 1_000;
    let duration = 1_000;

    f.env.as_contract(&f.contract_id, || {
        let lock = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        let new_duration = lock.end_ledger.checked_add(MIN_LOCK_DURATION).unwrap();
        let extended_lock = VoteEscrowContract::extend_lock(
            f.env.clone(),
            f.user.clone(),
            new_duration,
        );

        assert_eq!(extended_lock.end_ledger, new_duration);
    });
}

#[test]
fn test_withdraw() {
    let f = setup();
    let amount = 1_000;
    let duration = 100; // short duration

    f.env.as_contract(&f.contract_id, || {
        let lock = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        // Advance ledger past lock maturity
        f.env.ledger().with_sequence(lock.end_ledger + 1);

        let withdrawn_amount = VoteEscrowContract::withdraw(
            f.env.clone(),
            f.user.clone(),
        );

        assert_eq!(withdrawn_amount, amount);

        // Verify lock is marked withdrawn
        let stored_lock: Option<Lock> = f
            .env
            .storage()
            .persistent()
            .get(&DataKey::Lock(f.user.clone()));
        assert!(stored_lock.is_none());
    });
}

#[test]
fn test_withdraw_updates_total_checked() {
    let f = setup();
    let amount = 1_000;
    let duration = 100;

    f.env.as_contract(&f.contract_id, || {
        VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        let total_before: i128 = f
            .env
            .storage()
            .instance()
            .get(&DataKey::TotalLocked)
            .unwrap_or(0);
        assert_eq!(total_before, amount);

        // Advance ledger past lock maturity
        let lock: Lock = f
            .env
            .storage()
            .persistent()
            .get(&DataKey::Lock(f.user.clone()))
            .unwrap();
        f.env.ledger().with_sequence(lock.end_ledger + 1);

        VoteEscrowContract::withdraw(
            f.env.clone(),
            f.user.clone(),
        );

        let total_after: i128 = f
            .env
            .storage()
            .instance()
            .get(&DataKey::TotalLocked)
            .unwrap_or(0);
        assert_eq!(total_after, 0);
    });
}

#[test]
fn test_withdraw_before_maturity() {
    let f = setup();
    let amount = 1_000;
    let duration = 1_000;

    f.env.as_contract(&f.contract_id, || {
        let lock = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        // Try to withdraw before maturity
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            VoteEscrowContract::withdraw(
                f.env.clone(),
                f.user.clone(),
            );
        }));
        assert!(result.is_err());
    });
}

#[test]
fn test_withdraw_already_withdrawn() {
    let f = setup();
    let amount = 1_000;
    let duration = 100;

    f.env.as_contract(&f.contract_id, || {
        let lock = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        // Advance ledger past lock maturity
        f.env.ledger().with_sequence(lock.end_ledger + 1);

        VoteEscrowContract::withdraw(
            f.env.clone(),
            f.user.clone(),
        );

        // Try to withdraw again
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            VoteEscrowContract::withdraw(
                f.env.clone(),
                f.user.clone(),
            );
        }));
        assert!(result.is_err());
    });
}

#[test]
fn test_get_votes_active_lock() {
    let f = setup();
    let amount = 1_000;
    let duration = 1_000;

    f.env.as_contract(&f.contract_id, || {
        let lock = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        let votes = VoteEscrowContract::get_votes(f.env.clone(), f.user.clone());
        assert_eq!(votes, lock.initial_voting_power);
    });
}

#[test]
fn test_get_votes_decayed() {
    let f = setup();
    let amount = 1_000;
    let duration = 1_000;

    f.env.as_contract(&f.contract_id, || {
        let lock = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        // Advance to midpoint
        let midpoint = lock.start_ledger + (duration / 2);
        f.env.ledger().with_sequence(midpoint);

        let votes = VoteEscrowContract::get_votes(f.env.clone(), f.user.clone());
        assert!(votes < lock.initial_voting_power);
        assert!(votes > amount);
    });
}

#[test]
fn test_get_votes_expired() {
    let f = setup();
    let amount = 1_000;
    let duration = 100;

    f.env.as_contract(&f.contract_id, || {
        let lock = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        // Advance past maturity
        f.env.ledger().with_sequence(lock.end_ledger + 1);

        let votes = VoteEscrowContract::get_votes(f.env.clone(), f.user.clone());
        assert_eq!(votes, amount);
    });
}

#[test]
fn test_get_votes_no_lock() {
    let f = setup();

    f.env.as_contract(&f.contract_id, || {
        let votes = VoteEscrowContract::get_votes(f.env.clone(), f.user.clone());
        assert_eq!(votes, 0);
    });
}

#[test]
fn test_get_past_votes_during_lock() {
    let f = setup();
    let amount = 1_000;
    let duration = 1_000;

    f.env.as_contract(&f.contract_id, || {
        let lock = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        let votes = VoteEscrowContract::get_past_votes(
            f.env.clone(),
            f.user.clone(),
            lock.start_ledger + 500,
        );
        assert!(votes > 0);
        assert!(votes <= lock.initial_voting_power);
    });
}

#[test]
fn test_get_past_votes_before_lock() {
    let f = setup();
    let amount = 1_000;
    let duration = 1_000;

    f.env.as_contract(&f.contract_id, || {
        let lock = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        let votes = VoteEscrowContract::get_past_votes(
            f.env.clone(),
            f.user.clone(),
            lock.start_ledger - 1,
        );
        assert_eq!(votes, 0);
    });
}

#[test]
fn test_get_lock_history() {
    let f = setup();
    let amount = 1_000;
    let duration = 100;

    f.env.as_contract(&f.contract_id, || {
        let lock1 = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount,
            duration,
        );

        // Withdraw first lock
        f.env.ledger().with_sequence(lock1.end_ledger + 1);
        VoteEscrowContract::withdraw(f.env.clone(), f.user.clone());

        // Create second lock
        f.env.ledger().with_sequence(lock1.end_ledger + 2);
        let lock2 = VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount * 2,
            duration,
        );

        // Withdraw second lock
        f.env.ledger().with_sequence(lock2.end_ledger + 1);
        VoteEscrowContract::withdraw(f.env.clone(), f.user.clone());

        let history = VoteEscrowContract::get_lock_history(
            f.env.clone(),
            f.user.clone(),
            0,
            10,
        );
        assert_eq!(history.len(), 2);
    });
}

#[test]
fn test_multiple_users_independent_locks() {
    let f = setup();
    let amount1 = 1_000;
    let amount2 = 2_000;
    let duration = 1_000;

    f.env.as_contract(&f.contract_id, || {
        VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            amount1,
            duration,
        );

        VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user2.clone(),
            amount2,
            duration,
        );

        let total_locked: i128 = f
            .env
            .storage()
            .instance()
            .get(&DataKey::TotalLocked)
            .unwrap_or(0);
        assert_eq!(total_locked, amount1 + amount2);

        let votes1 = VoteEscrowContract::get_votes(f.env.clone(), f.user.clone());
        let votes2 = VoteEscrowContract::get_votes(f.env.clone(), f.user2.clone());

        assert!(votes1 > amount1);
        assert!(votes2 > amount2);
        assert!(votes2 > votes1); // More locked amount = more votes
    });
}

#[test]
fn test_update_escrow_config() {
    let f = setup();
    let new_min = 50;
    let new_max = 5_000;
    let new_multiplier = 10_000;

    f.env.as_contract(&f.contract_id, || {
        VoteEscrowContract::update_escrow_config(
            f.env.clone(),
            f.admin.clone(),
            new_min,
            new_max,
            new_multiplier,
        );

        let stored_min: u32 = f
            .env
            .storage()
            .instance()
            .get(&DataKey::MinLockDuration)
            .unwrap();
        assert_eq!(stored_min, new_min);

        let stored_max: u32 = f
            .env
            .storage()
            .instance()
            .get(&DataKey::MaxLockDuration)
            .unwrap();
        assert_eq!(stored_max, new_max);

        // Create lock with new minimum duration
        VoteEscrowContract::create_lock(
            f.env.clone(),
            f.user.clone(),
            1_000,
            new_min,
        );
    });
}
