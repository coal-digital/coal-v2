use drillx::Solution;
use steel::*;
use ore_api::{
    state::proof_pda as ore_proof_pda,
    consts::{
        TREASURY_ADDRESS as ORE_TREASURY_ADDRESS,
        TREASURY_TOKENS_ADDRESS as ORE_TREASURY_TOKENS_ADDRESS,
    },
};

use crate::{
    consts::*,
    instruction::*,
    state::{bus_pda, config_pda, proof_pda, treasury_pda},
};

/// Builds an auth instruction.
pub fn auth(proof: Pubkey) -> Instruction {
    Instruction {
        program_id: NOOP_PROGRAM_ID,
        accounts: vec![],
        data: proof.to_bytes().to_vec(),
    }
}

/// Builds a claim instruction.
pub fn claim(mint: Pubkey, signer: Pubkey, beneficiary: Pubkey, amount: u64) -> Instruction {
    let proof = proof_pda(mint, signer).0;
    let ore_proof: (Pubkey, u8) = ore_proof_pda(signer);

    let treasury = treasury_pda(mint);
    let treasury_tokens_address = spl_associated_token_account::get_associated_token_address(
        &treasury.0,
        &mint,
    );
    let treasury_ore_tokens_address = spl_associated_token_account::get_associated_token_address(
        &treasury.0,
        &ORE_MINT_ADDRESS,
    );

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(beneficiary, false),
            AccountMeta::new(proof, false),
            AccountMeta::new(mint, false),
            AccountMeta::new(ore_proof.0, false),
            AccountMeta::new_readonly(treasury.0, false),
            AccountMeta::new(treasury_tokens_address, false),
            AccountMeta::new(treasury_ore_tokens_address, false),
            AccountMeta::new_readonly(ORE_TREASURY_ADDRESS, false),
            AccountMeta::new(ORE_TREASURY_TOKENS_ADDRESS, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: Claim {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

/// Builds a close instruction.
pub fn close(mint: Pubkey, signer: Pubkey) -> Instruction {
    let proof = proof_pda(mint, signer).0;
    let ore_proof: (Pubkey, u8) = ore_proof_pda(signer);
    
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(proof, false),
            AccountMeta::new(ore_proof.0, false),
            AccountMeta::new_readonly(solana_program::system_program::ID, false),
        ],
        data: Close {}.to_bytes(),
    }
}

/// Builds a mine instruction.
pub fn mine(
    mint: Pubkey,
    signer: Pubkey,
    authority: Pubkey,
    coal_bus: Pubkey,
    ore_bus: Pubkey,
    solution: Solution,
    additional_accounts: Vec<Pubkey>,
) -> Instruction {
    let proof = proof_pda(mint, authority).0;
    let required_accounts = vec![
        AccountMeta::new(signer, true),
        AccountMeta::new(MINT_ADDRESS, false),
        AccountMeta::new(coal_bus, false),
        AccountMeta::new_readonly(CONFIG_ADDRESS, false),
        AccountMeta::new(ore_bus, false),
        AccountMeta::new_readonly(ORE_CONFIG_ADDRESS, false),
        AccountMeta::new(proof, false),
        AccountMeta::new_readonly(sysvar::instructions::ID, false),
        AccountMeta::new_readonly(sysvar::slot_hashes::ID, false),
    ];
    let additional_accounts = additional_accounts
        .into_iter()
        .map(|pk| AccountMeta::new_readonly(pk, false))
        .collect();
    Instruction {
        program_id: crate::ID,
        accounts: [required_accounts, additional_accounts].concat(),
        data: Mine {
            digest: solution.d,
            nonce: solution.n,
        }
        .to_bytes(),
    }
}

/// Builds an open instruction.
pub fn open(mint: Pubkey, signer: Pubkey, miner: Pubkey, payer: Pubkey) -> Instruction {
    let proof_pda: (Pubkey, u8) = proof_pda(mint, signer);
    let ore_proof_pda = ore_proof_pda(signer);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new_readonly(miner, false),
            AccountMeta::new(payer, true),
            AccountMeta::new(proof_pda.0, false),
            AccountMeta::new(ore_proof_pda.0, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new_readonly(solana_program::system_program::ID, false),
            AccountMeta::new_readonly(sysvar::slot_hashes::ID, false),
        ],
        data: Open { proof_bump: proof_pda.1 }.to_bytes(),
    }
}

/// Builds a reset instruction.
pub fn reset(mint: Pubkey, signer: Pubkey) -> Instruction {
    let bus_pdas = [
        bus_pda(mint, 0),
        bus_pda(mint, 1),
        bus_pda(mint, 2),
        bus_pda(mint, 3),
        bus_pda(mint, 4),
        bus_pda(mint, 5),
        bus_pda(mint, 6),
        bus_pda(mint, 7),
    ];
    let config_pda = config_pda(mint);
    let treasury_tokens_address = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &mint,
    );

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(bus_pdas[0].0, false),
            AccountMeta::new(bus_pdas[1].0, false),
            AccountMeta::new(bus_pdas[2].0, false),
            AccountMeta::new(bus_pdas[3].0, false),
            AccountMeta::new(bus_pdas[4].0, false),
            AccountMeta::new(bus_pdas[5].0, false),
            AccountMeta::new(bus_pdas[6].0, false),
            AccountMeta::new(bus_pdas[7].0, false),
            AccountMeta::new(config_pda.0, false),
            AccountMeta::new(mint, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new(treasury_tokens_address, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: Reset {}.to_bytes(),
    }
}

/// Build a stake instruction.
pub fn stake(mint: Pubkey, signer: Pubkey, sender: Pubkey, amount: u64) -> Instruction {
    let proof = proof_pda(mint, signer).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(proof, false),
            AccountMeta::new(sender, false),
            AccountMeta::new(TREASURY_TOKENS_ADDRESS, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: Stake {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

// Build an update instruction.
pub fn update(mint: Pubkey, signer: Pubkey, miner: Pubkey) -> Instruction {
    let proof = proof_pda(mint, signer).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new_readonly(miner, false),
            AccountMeta::new(proof, false),
        ],
        data: Update {}.to_bytes(),
    }
}

// Build an upgrade instruction for COAL v1 to v2.
pub fn upgrade(signer: Pubkey, beneficiary: Pubkey, sender: Pubkey, amount: u64) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(beneficiary, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(MINT_V1_ADDRESS, false),
            AccountMeta::new(sender, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: Upgrade {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

/// Builds an initialize instruction.
pub fn initialize(signer: Pubkey, mint_noise: [u8; 16]) -> Instruction {
    let mint_pda = Pubkey::find_program_address(&[MINT, mint_noise.as_slice()], &crate::ID);
    let bus_pdas = [
        bus_pda(mint_pda.0, 0),
        bus_pda(mint_pda.0, 1),
        bus_pda(mint_pda.0, 2),
        bus_pda(mint_pda.0, 3),
        bus_pda(mint_pda.0, 4),
        bus_pda(mint_pda.0, 5),
        bus_pda(mint_pda.0, 6),
        bus_pda(mint_pda.0, 7),
    ];
    let config_pda = config_pda(mint_pda.0);
    let treasury_pda = treasury_pda(mint_pda.0);
    let treasury_tokens_address = spl_associated_token_account::get_associated_token_address(
        &treasury_pda.0,
        &mint_pda.0,
    );
    let ore_treasury_tokens_address = spl_associated_token_account::get_associated_token_address(
        &treasury_pda.0,
        &ORE_MINT_ADDRESS,
    );
    let metadata_pda = Pubkey::find_program_address(
        &[
            METADATA,
            mpl_token_metadata::ID.as_ref(),
            mint_pda.0.as_ref(),
        ],
        &mpl_token_metadata::ID,
    );

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(bus_pdas[0].0, false),
            AccountMeta::new(bus_pdas[1].0, false),
            AccountMeta::new(bus_pdas[2].0, false),
            AccountMeta::new(bus_pdas[3].0, false),
            AccountMeta::new(bus_pdas[4].0, false),
            AccountMeta::new(bus_pdas[5].0, false),
            AccountMeta::new(bus_pdas[6].0, false),
            AccountMeta::new(bus_pdas[7].0, false),
            AccountMeta::new(config_pda.0, false),
            AccountMeta::new(metadata_pda.0, false),
            AccountMeta::new(mint_pda.0, false),
            AccountMeta::new(ORE_MINT_ADDRESS, false),
            AccountMeta::new(treasury_pda.0, false),
            AccountMeta::new(treasury_tokens_address, false),
            AccountMeta::new(ore_treasury_tokens_address, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(mpl_token_metadata::ID, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
        ],
        data: Initialize {
            mint_noise,
            treasury_bump: treasury_pda.1,
            mint_bump: mint_pda.1,
            metadata_name: METADATA_NAME.to_string().as_bytes()[..32].try_into().unwrap(),
            metadata_symbol: METADATA_SYMBOL.to_string().as_bytes()[..8].try_into().unwrap(),
            metadata_uri: METADATA_URI.to_string().as_bytes()[..128].try_into().unwrap(),
        }
        .to_bytes(),
    }
}
