#![no_std]
extern crate alloc;
mod mol;
use alloc::vec::Vec;
use core::convert::From;
pub use mol::*;
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct GenesisId(pub [u8; 32]);
impl From<Vec<u8>> for GenesisId {
    fn from(vec: Vec<u8>) -> Self {
        let mut inner = [0u8; 32];
        inner.copy_from_slice(&vec[0..32]);
        Self(inner)
    }
}

impl From<[u8; 32]> for GenesisId {
    fn from(arr: [u8; 32]) -> Self {
        Self(arr)
    }
}
