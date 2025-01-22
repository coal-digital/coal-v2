use coal_api::prelude::*;
use ore_api;
use ore_boost_api;
use steel::*;

/// Open creates a new proof account to track a miner's state.
pub fn process_open(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Open::try_from_bytes(data)?;

    // Load accounts.
    let [signer_info, config_info, miner_info, payer_info, proof_info, ore_proof_info, ore_reservation_into, mint_info, system_program, slot_hashes_info] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    config_info.is_config()?
        .as_account::<Config>(&coal_api::ID)?
        .assert(|c| c.mint == *mint_info.key)?;
    payer_info.is_signer()?;
    proof_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[PROOF, mint_info.key.as_ref(), signer_info.key.as_ref()], &coal_api::ID)?;
    mint_info.as_mint()?;
    system_program.is_program(&system_program::ID)?;
    slot_hashes_info.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Initialize proof.
    create_account::<Proof>(
        proof_info,
        system_program,
        payer_info,
        &coal_api::ID,
        &[PROOF, mint_info.key.as_ref(), signer_info.key.as_ref()],
    )?;
    let clock = Clock::get()?;
    let proof = proof_info.as_account_mut::<Proof>(&coal_api::ID)?;
    proof.authority = *signer_info.key;
    proof.balance = 0;
    proof.last_stake_at = clock.unix_timestamp;
    proof.miner = *miner_info.key;
    proof.total_hashes = 0;
    proof.total_rewards = 0;
    proof.bump = args.proof_bump as u64;


    let open_accounts = &[
        proof_info.clone(),
        mint_info.clone(),
        payer_info.clone(),
        ore_proof_info.clone(),
        system_program.clone(),
        slot_hashes_info.clone()
    ];

    solana_program::program::invoke_signed(
        &ore_api::sdk::open(
            *proof_info.key,
            // COAL mint is the miner for all tokens
            MINT_ADDRESS,
            *payer_info.key,
        ),
        open_accounts,
        &[&[PROOF, mint_info.key.as_ref(), signer_info.key.as_ref(), &[args.proof_bump]]]
    )?;


    let register_accounts = &[
        proof_info.clone(),
        payer_info.clone(),
        ore_proof_info.clone(),
        ore_reservation_into.clone(),
        system_program.clone(),
    ];
    // Register the proof with the boost program
    solana_program::program::invoke_signed(
        &ore_boost_api::sdk::register(
            *proof_info.key,
            *payer_info.key,
            *ore_proof_info.key
        ),
        register_accounts,
        &[&[PROOF, mint_info.key.as_ref(), signer_info.key.as_ref(), &[args.proof_bump]]]
    )?;

    Ok(())
}
