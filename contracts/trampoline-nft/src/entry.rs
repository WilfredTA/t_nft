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

use trampoline_sdk::builtins::t_nft::{TrampolineNFT, GenesisId, ContentId};
use trampoline_sdk::schema::{SchemaPrimitiveType, BytesConversion, MolConversion};
use crate::error::Error;

const CKB_HASH_DIGEST: usize = 32;
const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

// Rules:
// 1. Any nft in output that is NOT in input must have correct genesis id
// 2. For each gen_id in input: count(input) >= count(output)
// 3. Each nft data field >= 64 bytes
pub fn main() -> Result<(), Error> {
    let mut i = 0;
    let mut input_nfts: Vec<TrampolineNFT> = vec![];
    let mut output_nfts: Vec<TrampolineNFT> = vec![];
    loop {
        match load_nft_from(i, GroupInput) {
            Ok(nft) => {
                input_nfts.push(nft);
            },
            Err(Error::DataSizeTooSmall) => {
                return Err(Error::DataSizeTooSmall);
            },
            Err(Error::IndexOutOfBound) => {
                break;
            },
            Err(e) => {
                return Err(e);
            }
        };
        i += 1;
    }
   
    i = 0;
    
    loop {
        match load_nft_from(i, GroupOutput) {
            Ok(nft) => {
                output_nfts.push(nft);
            },
            Err(Error::DataSizeTooSmall) => {
                return Err(Error::DataSizeTooSmall);
            },
            Err(Error::IndexOutOfBound) => {
                break;
            },
            Err(e) => {
                return Err(e);
            }
        };
        i += 1;
    }

    let gen_id = load_genesis_id()?;

    let minted_outputs = output_nfts.iter().filter(|nft: &&TrampolineNFT| {
        input_nfts.iter().find(|in_nft: &&TrampolineNFT| {
            in_nft.genesis_id.to_bytes() == nft.genesis_id.to_bytes()
        }).is_none()
    }).collect::<Vec<_>>();

    minted_outputs.iter().for_each(|nft| {
        assert!(nft.genesis_id.to_bytes() == gen_id.to_bytes());
    });
    Ok(())
}

fn load_genesis_id() -> Result<GenesisId, Error> {
    let genesis_seed = high_level::load_input_out_point(0, Source::Input)?;
    let seed_tx_hash = genesis_seed.tx_hash();
    let seed_idx = genesis_seed.index();
    let mut hasher = blake2b();
    hasher.update(seed_tx_hash.as_slice());
    hasher.update(seed_idx.as_slice());
    let mut expected_genesis_id = [0u8; 32];
    hasher.finalize(&mut expected_genesis_id);
    
    Ok(GenesisId::from_mol(expected_genesis_id.pack()))
}
fn load_nft_from(idx: usize, src: Source) -> Result<TrampolineNFT, Error> {
    let data = load_cell_data(idx, src)?;
    if data.len() < 64 {
        return Err(Error::DataSizeTooSmall);
    }
    let data = Bytes::from(data);
    let genesis_id_data = data[..32].into();
    let cid_data = data[32..].into();

    let nft = TrampolineNFT {
        genesis_id: GenesisId::from_bytes(genesis_id_data),
        cid: ContentId::from_bytes(cid_data)
    };
    Ok(nft)
}


pub fn blake2b() -> Blake2b {
    Blake2bBuilder::new(CKB_HASH_DIGEST)
    .personal(CKB_HASH_PERSONALIZATION)
    .build()
}
pub fn blake2b_256(s: &[u8]) -> [u8; 32] {
    let mut result = [0u8; CKB_HASH_DIGEST];
    let mut blake2b = Blake2bBuilder::new(CKB_HASH_DIGEST)
        .personal(CKB_HASH_PERSONALIZATION)
        .build();
    blake2b.update(s);
    blake2b.finalize(&mut result);
    result
}