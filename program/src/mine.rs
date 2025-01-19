use std::ops::Div;

use drillx::Solution;
use coal_api::prelude::*;
use ore_api;
use ore_boost_api;
use steel::*;

/// Mine validates hashes and increments a miner's claimable balance.
pub fn process_mine(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Mine::try_from_bytes(data)?;

    // Load accounts.
    let clock = Clock::get()?;
    let t: i64 = clock.unix_timestamp;
    
    let (required_accounts, optional_accounts) = accounts.split_at(14);
    let split_index = optional_accounts.iter().position(|acc| acc.owner.eq(&ore_boost_api::ID)).unwrap_or(optional_accounts.len());
    let (coal_optional_accounts, boost_accounts) = optional_accounts.split_at(split_index);
    let [signer_info, coal_mint_info, mint_info, bus_info, config_info, proof_info, ore_bus_info, ore_config_info, ore_proof_info, ore_directory_info, ore_reservation_info, ore_treasury_tokens_info, instructions_sysvar, slot_hashes_sysvar] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let bus = bus_info.is_bus()?.as_account_mut::<Bus>(&coal_api::ID)?;
    let config = config_info
        .is_config()?
        .as_account::<Config>(&coal_api::ID)?
        .assert_err(
            |c| c.last_reset_at.saturating_add(EPOCH_DURATION) > t,
            OreError::NeedsReset.into(),
        )?;
    let proof = proof_info
        .as_account_mut::<Proof>(&coal_api::ID)?
        .assert_mut_err(
            |p| p.miner == *signer_info.key,
            ProgramError::MissingRequiredSignature,
        )?;
    instructions_sysvar.is_sysvar(&sysvar::instructions::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

     // Submit solution to the ORE program
     let solution = Solution::new(args.digest, args.nonce);
     let hash = solution.to_hash();
     let difficulty = hash.difficulty();
     let mut boost_keys = None;
     let mut mine_accounts = vec![
         coal_mint_info.clone(),
         ore_bus_info.clone(),
         ore_config_info.clone(),
         ore_proof_info.clone(),
         instructions_sysvar.clone(),
         slot_hashes_sysvar.clone(),
     ];
     if let [boost_info, _boost_proof_info, reservation_info] = boost_accounts {
        boost_keys = Some((*boost_info.key, *reservation_info.key));
        mine_accounts.extend_from_slice(boost_accounts);
    }
    
    solana_program::program::invoke_signed(
        &ore_api::sdk::mine(
            *coal_mint_info.key,
            *proof_info.key,
            *ore_bus_info.key,
            solution,
            boost_keys,
        ),
        &mine_accounts,
        &[&[MINT, MINT_NOISE.as_slice(), &[MINT_BUMP]]]
    )?;

    let rotate_accounts = [
        proof_info.clone(),
        ore_directory_info.clone(),
        ore_proof_info.clone(),
        ore_reservation_info.clone(),
        ore_treasury_tokens_info.clone(),
    ];

    solana_program::program::invoke_signed(
        &ore_boost_api::sdk::rotate(
            *proof_info.key, 
            *ore_proof_info.key
        ),
        &rotate_accounts,
        &[&[PROOF, mint_info.key.as_ref(), signer_info.key.as_ref(), &[proof.bump as u8]]]
    )?;     

    // Normalize the difficulty and calculate the reward amount.
    //
    // The reward doubles for every bit of difficulty (leading zeros) on the hash. We use the normalized
    // difficulty so the minimum accepted difficulty pays out at the base reward rate.
    let normalized_difficulty = difficulty
        .checked_sub(config.min_difficulty as u32)
        .unwrap();
    let mut reward = config
        .base_reward_rate
        .checked_mul(2u64.checked_pow(normalized_difficulty).unwrap())
        .unwrap();

    // Apply boosts.
    //
    // Boosts are staking incentives that can multiply a miner's rewards. Up to 3 boosts can be applied
    // on any given mine operation.
    // TODO handle COAL boosts with coal_optional_accounts

    // Apply bus limit.
    //
    // Busses are limited to distributing 1 ORE per epoch. The payout amount must be capped to whatever is
    // left in the selected bus. This limits the maximum amount that will be paid out for any given hash to 1 ORE.
    let reward_actual = reward.min(bus.rewards).min(config.get_epoch_rewards().saturating_div(BUS_COUNT as u64));

    // Update balances.
    //
    // We track the theoretical rewards that would have been paid out ignoring the bus limit, so the
    // base reward rate will be updated to account for the real hashpower on the network.
    bus.theoretical_rewards = bus.theoretical_rewards.checked_add(reward).unwrap();
    bus.rewards = bus.rewards.checked_sub(reward_actual).unwrap();
    proof.balance = proof.balance.checked_add(reward_actual).unwrap();

    proof.total_hashes = proof.total_hashes.saturating_add(1);
    proof.total_rewards = proof.total_rewards.saturating_add(reward_actual);

    // Log data.
    //
    // The boost rewards are scaled down before logging to account for penalties and bus limits.
    // This return data can be used by pool operators to calculate miner and staker rewards.
    // for i in 0..3 {
    //     boost_rewards[i] = (boost_rewards[i] as u128)
    //         .checked_mul(reward_actual as u128)
    //         .unwrap()
    //         .checked_div(reward_pre_penalty as u128)
    //         .unwrap() as u64;
    // }
    MineEvent {
        balance: proof.balance,
        difficulty: difficulty as u64,
        reward: reward_actual,
    }
    .log_return();

    Ok(())
}