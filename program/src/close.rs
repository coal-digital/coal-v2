use coal_api::prelude::*;
use ore_api;
use steel::*;

/// Close closes a proof account and returns the rent to the owner.
pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, proof_info, ore_proof_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    proof_info
        .is_writable()?
        .as_account::<Proof>(&ore_api::ID)?
        .assert_err(
            |p| p.authority == *signer_info.key,
            ProgramError::MissingRequiredSignature,
        )?
        .assert(|p| p.balance == 0)?;
    system_program.is_program(&system_program::ID)?;
    
    let proof = proof_info.as_account::<Proof>(&coal_api::ID)?;
    let close_accounts = &[
        proof_info.clone(),
        ore_proof_info.clone(),
        system_program.clone()
    ];
    let proof_bump = proof.bump as u8;
    solana_program::program::invoke_signed(
        &ore_api::sdk::close(
            *proof_info.key,
        ),
        close_accounts,
        &[&[PROOF, signer_info.key.as_ref(), &[proof_bump]]]
    )?;

    // Return rent to signer.
    proof_info.close(signer_info)?;

    Ok(())
}
