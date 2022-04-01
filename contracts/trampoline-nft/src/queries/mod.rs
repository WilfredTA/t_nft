use crate::error::Error;
use alloc::collections::BTreeMap;
use alloc::{vec, vec::Vec};
use blake2b_ref::{Blake2b, Blake2bBuilder};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::packed::{Byte32, OutPoint, Uint32},
    ckb_types::prelude::*,
    high_level::{self, load_cell_data},
};
use tnft_schema::GenesisId;
const CKB_HASH_DIGEST: usize = 32;
const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";
pub fn get_nft_data_size() {}

pub fn get_by_genesis_id(id: Byte32, source: Source) {}

pub fn count_genesis_id(id: Byte32, source: Source) {}

pub fn get_genesis_id_of(idx: usize, source: Source) -> Result<GenesisId, Error> {
    let cell = high_level::load_cell_data(idx, source)?;
    let genesis_id = cell.into();
    Ok(genesis_id)
}

pub fn get_curr_genesis_id() -> Result<GenesisId, Error> {
    let genesis_seed = high_level::load_input_out_point(0, Source::Input)?;
    let seed_tx_hash = genesis_seed.tx_hash();
    let seed_idx = genesis_seed.index();
    let mut seed = Vec::with_capacity(36);
    seed.extend_from_slice(seed_tx_hash.as_slice());
    seed.extend_from_slice(seed_idx.as_slice());
    let mut hasher = blake2b();
    hasher.update(&seed);
    let mut expected_genesis_id = [0u8; 32];
    hasher.finalize(&mut expected_genesis_id);
    Ok(expected_genesis_id.into())
}
pub type IdWithIndex = (usize, GenesisId);
pub fn get_all_genesis_tnft_ids() -> Result<(Vec<GenesisId>, Vec<GenesisId>), Error> {
    let curr_gen_id = get_curr_genesis_id()?;

    let input_gen_ids = high_level::QueryIter::new(load_cell_data, Source::GroupInput)
        .map(|cell_data| cell_data.into())
        .collect::<Vec<GenesisId>>();

    let output_gen_ids = high_level::QueryIter::new(load_cell_data, Source::GroupOutput)
        .map(|cell_data| cell_data.into())
        .collect::<Vec<GenesisId>>();
    Ok((input_gen_ids, output_gen_ids))
}

pub fn map_of_genesis_ids(
    tx_ids: (Vec<GenesisId>, Vec<GenesisId>),
) -> Result<BTreeMap<GenesisId, (usize, usize)>, Error> {
    let (input_ids, output_ids) = tx_ids;
    let mut map: BTreeMap<GenesisId, (usize, usize)> = BTreeMap::new();
    input_ids.into_iter().for_each(|id| {
        if let Some(count) = map.get_mut(&id) {
            let (in_count, out_count) = count;
            *in_count += 1;
        } else {
            map.insert(id, (1, 0));
        }
    });

    output_ids.into_iter().for_each(|id| {
        if let Some(count) = map.get_mut(&id) {
            let (in_count, out_count) = count;
            *out_count += 1;
        } else {
            map.insert(id, (0, 1));
        }
    });

    Ok(map)
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
