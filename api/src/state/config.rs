use steel::*;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Ingredient {
    pub mint: Pubkey,
    pub ratio: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Config {
    /// The token this config is for.
    pub mint: Pubkey,

    /// The mint of the migration token.
    pub migration_mint: Pubkey,

    /// Max supply of the token.
    pub max_supply: u64,

    /// The base reward rate paid out for a hash of minimum difficulty.
    pub base_reward_rate: u64,

    /// The timestamp of the last reset.
    pub last_reset_at: i64,

    /// The minimum accepted difficulty.
    pub min_difficulty: u64,

    /// The total coal balance in the treasury.
    pub total_balance: u64,

    /// The total lifetime COAL rewards distributed to miners.
    pub total_rewards: u64,

    /// The target emission rate per epoch.
    pub initial_epoch_rewards: u64,

    /// The epoch number (incremented each reset)
    pub current_epoch: u64,

    /// The number of epochs before the emissions rate decays
    /// If this is 0, the emissions rate does not decay
    pub schedule_epochs: u64,

    /// The decay rate in basis points
    pub decay_basis_points: u64,

    /// An ingredient that is wrapped in the treasury.
    pub wrapped_ingredient: Ingredient,

    /// An ingredient that is burned in the treasury.
    pub burned_ingredient: Ingredient,
}

impl Config {
    pub fn get_epoch_rewards(&self) -> u64 {
        if self.schedule_epochs == 0 {
            return self.initial_epoch_rewards;
        }

        let schedule_position = self.current_epoch / self.schedule_epochs;
        if schedule_position == 0 {
            return self.initial_epoch_rewards;
        }

        // Calculate decay factor: (1 - decay_rate)^schedule_position
        // Using basis points (10000 = 100%)
        let mut remaining: u64 = 10_000;
        for _ in 0..schedule_position {
            remaining = remaining.saturating_mul(10_000 - self.decay_basis_points) / 10_000;
        }

        // Apply decay to target rewards
        self.initial_epoch_rewards.saturating_mul(remaining) / 10_000
    }
}

account!(OreAccount, Config);
