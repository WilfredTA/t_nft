// Import from `core` instead of from `std` since we are in no-std mode


use core::result::Result;


// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{vec, vec::Vec};

use blake2b_ref::{Blake2bBuilder, Blake2b};
// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use ckb_std::{
    debug,
    high_level::{load_script, load_tx_hash, load_cell_data, self, look_for_dep_with_data_hash, QueryIter},
    ckb_types::{bytes::Bytes, prelude::*},
    ckb_constants::Source::{GroupInput, GroupOutput, self},
};
use crate::error::Error;

const CKB_HASH_DIGEST: usize = 32;
const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";


pub fn main() -> Result<(), Error> {

    Ok(())
}

// fn load_genesis_id() -> Result<GenesisId, Error> {
//     let genesis_seed = high_level::load_input_out_point(0, Source::Input)?;
//     let seed_tx_hash = genesis_seed.tx_hash();
//     let seed_idx = genesis_seed.index();
//     let mut seed = Vec::with_capacity(36);
//     seed.extend_from_slice(seed_tx_hash.as_slice());
//     seed.extend_from_slice(seed_idx.as_slice());
//     let mut hasher = blake2b();
//     hasher.update(&seed);
//     let mut expected_genesis_id = [0u8; 32];
//     hasher.finalize(&mut expected_genesis_id);
    
//     Ok(GenesisId::from_mol(expected_genesis_id.pack()))
// }
// fn load_nft_from(idx: usize, src: Source) -> Result<TrampolineNFT, Error> {
//     let data = load_cell_data(idx, src)?;
//     if data.len() < 64 {
//         return Err(Error::DataSizeTooSmall);
//     }
//     let data = Bytes::from(data);
//     let genesis_id_data = data[..32].into();
//     let cid_data = data[32..].into();

//     let nft = TrampolineNFT {
//         genesis_id: GenesisId::from_bytes(genesis_id_data),
//         cid: ContentId::from_bytes(cid_data)
//     };
//     Ok(nft)
// }


// pub fn blake2b() -> Blake2b {
//     Blake2bBuilder::new(CKB_HASH_DIGEST)
//     .personal(CKB_HASH_PERSONALIZATION)
//     .build()
// }
// pub fn blake2b_256(s: &[u8]) -> [u8; 32] {
//     let mut result = [0u8; CKB_HASH_DIGEST];
//     let mut blake2b = Blake2bBuilder::new(CKB_HASH_DIGEST)
//         .personal(CKB_HASH_PERSONALIZATION)
//         .build();
//     blake2b.update(s);
//     blake2b.finalize(&mut result);
//     result
// }