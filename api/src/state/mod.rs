mod bus;
mod config;
mod proof;
mod treasury;

pub use bus::*;
pub use config::*;
pub use proof::*;
pub use treasury::*;

use steel::*;

use crate::consts::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum OreAccount {
    Bus = 100,
    Config = 101,
    Proof = 102,
    Treasury = 103,
}

/// Fetch the PDA of a bus account.
pub fn bus_pda(mint: Pubkey, id: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[BUS, mint.as_ref(), &[id]], &crate::id())
}

/// Derive the PDA of the config account.
pub fn config_pda(mint: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[CONFIG, mint.as_ref()], &crate::id())
}

/// Derive the PDA of a proof account.
pub fn proof_pda(mint: Pubkey, authority: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PROOF, mint.as_ref(), authority.as_ref()], &crate::id())
}

/// Derive the PDA of the treasury account.
pub fn treasury_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TREASURY], &crate::id())
}
    