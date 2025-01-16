use steel::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct MineEvent {
    pub balance: u64,
    pub difficulty: u64,
    pub reward: u64,
}

event!(MineEvent);
