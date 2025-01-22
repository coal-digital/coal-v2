use coal_api::prelude::*;
use ore_api;
use steel::*;

/// Claim distributes claimable ORE from the treasury to a miner.
pub fn process_claim(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Claim::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, beneficiary_info, proof_info, mint_info, ore_proof_info, treasury_info, treasury_tokens_info, treasury_ore_tokens_info, ore_treasury_info, ore_treasury_tokens_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    beneficiary_info
        .is_writable()?
        .as_token_account()?
        .assert(|t| t.mint == *mint_info.key)?;
    let proof = proof_info
        .as_account_mut::<Proof>(&coal_api::ID)?
        .assert_mut_err(
            |p| p.authority == *signer_info.key,
            ProgramError::MissingRequiredSignature,
        )?;
    treasury_info.is_treasury()?;
    treasury_tokens_info.is_writable()?.is_treasury_tokens()?;
    token_program.is_program(&spl_token::ID)?;

    // Update miner balance.
    proof.balance = proof
        .balance
        .checked_sub(amount)
        .ok_or(OreError::ClaimTooLarge)?;

    // Transfer tokens from treasury to beneficiary.
    transfer_signed(
        treasury_info,
        treasury_tokens_info,
        beneficiary_info,
        token_program,
        amount,
        &[TREASURY],
    )?;

    // Claim remaining ORE to treasury when balance is 0.
    if proof.balance == 0 {
        let ore_proof = ore_proof_info.as_account::<Proof>(&ore_api::ID)?;
        let claim_accounts = &[
            proof_info.clone(),
            treasury_ore_tokens_info.clone(),
            ore_proof_info.clone(), 
            ore_treasury_info.clone(),
            ore_treasury_tokens_info.clone(),
            token_program.clone()
        ];
        let proof_bump = proof.bump as u8;
        
        solana_program::program::invoke_signed(
            &ore_api::sdk::claim(
                *proof_info.key, 
                *treasury_ore_tokens_info.key, 
                ore_proof.balance
            ),
            claim_accounts,
            &[&[PROOF, mint_info.key.as_ref(), signer_info.key.as_ref(), &[proof_bump]]]
        )?;
    }

    Ok(())
}
