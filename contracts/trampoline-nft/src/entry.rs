// Import from `core` instead of from `std` since we are in no-std mode

use core::result::Result;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{vec, vec::Vec};

use blake2b_ref::{Blake2b, Blake2bBuilder};
// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use crate::error::Error;
use crate::invariants::{genesis_id_conservation_and_consistency, minimum_data_size};
use crate::queries::get_all_genesis_tnft_ids;
use ckb_std::{
    ckb_constants::Source::{self, GroupInput, GroupOutput},
    ckb_types::{bytes::Bytes, prelude::*},
    debug,
    high_level::{
        self, load_cell_data, load_script, load_tx_hash, look_for_dep_with_data_hash, QueryIter,
    },
};
const CKB_HASH_DIGEST: usize = 32;
const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub fn main() -> Result<(), Error> {
    let tnfts = get_all_genesis_tnft_ids()?;
    genesis_id_conservation_and_consistency(tnfts)?;
    minimum_data_size()?;
    Ok(())
}
