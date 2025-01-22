use coal_api::prelude::*;
use steel::*;

/// Upgrade allows a user to migrate a v1 token to a v2 token at a 1:1 exchange rate.
pub fn process_upgrade(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args
    let args = Upgrade::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts
    let [signer_info, config_info, beneficiary_info, mint_info, mint_v1_info, sender_info, treasury_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info
        .as_account::<Config>(&coal_api::ID)?
        .assert(|c| c.mint == *mint_info.key)?;
    beneficiary_info
        .is_writable()?
        .as_token_account()?
        .assert(|t| t.owner == *signer_info.key)?;
    let mint = mint_info
        .is_writable()?
        .has_address(&config.mint)?
        .as_mint()?;
    mint_v1_info
        .is_writable()?
        .has_address(&config.migration_mint)?
        .as_mint()?;
    sender_info
        .is_writable()?
        .as_token_account()?
        .assert(|t| t.owner == *signer_info.key)?
        .assert(|t| t.mint == config.migration_mint)?;
    treasury_info.is_treasury()?;
    token_program.is_program(&spl_token::ID)?;

    // Burn v1 tokens
    solana_program::program::invoke(
        &spl_token::instruction::burn(
            &spl_token::id(),
            sender_info.key,
            mint_v1_info.key,
            signer_info.key,
            &[signer_info.key],
            amount,
        )?,
        &[
            token_program.clone(),
            sender_info.clone(),
            mint_v1_info.clone(),
            signer_info.clone(),
        ],
    )?;

    // Cap at max supply.
    if mint.supply.gt(&config.max_supply) {
        return Err(OreError::MaxSupply.into());
    }

    // Mint to the beneficiary account
    mint_to_signed(
        mint_info,
        beneficiary_info,
        treasury_info,
        token_program,
        amount,
        &[TREASURY],
    )?;

    Ok(())
}