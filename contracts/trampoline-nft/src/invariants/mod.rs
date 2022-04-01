// Rules:
// 1. Any nft in output that is NOT in input must have correct genesis id
// 2. For each genesis ID in input: count(input) >= count(output) -- i.e., there cannot be newly created NFTs with a pre-existing
//    genesis ID
// 3. Each nft data field >= 64 bytes
// 4. Arg field is at least 34 bytes
// 5. If args[0] == 1, then script in cell_deps with matching hash of args[2..34] must be executed
// 6. If args[0] == 0, then script in cell_deps with matching hash of args[2..34] is executed only if present
// 7. If args[1] == HashType::Data, then use cell_deps data hash, else use cell_deps type hash
use crate::error::Error;
use crate::queries::{get_all_genesis_tnft_ids, get_curr_genesis_id, map_of_genesis_ids};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use ckb_std::debug;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::packed::*,
    ckb_types::prelude::*,
    high_level::{
        self, load_cell_capacity, load_cell_data, load_cell_occupied_capacity, QueryIter,
    },
};
use tnft_schema::*;
pub const ONE_CKB: u64 = 100_000_000;
pub const MIN_NFT_DATA_SIZE: u64 = ONE_CKB * 64;

// Below function *assumes* src == Source::GroupInput or Source::GroupOutput
// This is safe because it is only called in this file & is only called with one of the above two source options
fn validate_data_size_in_source(src: Source) -> Result<(), Error> {
    let capacity_too_low_cell =
        QueryIter::new(load_cell_occupied_capacity, src).find(|cap| cap < &MIN_NFT_DATA_SIZE);
    if capacity_too_low_cell.is_some() {
        return Err(Error::DataSizeTooSmall);
    }
    Ok(())
}
pub fn minimum_data_size() -> Result<(), Error> {
    validate_data_size_in_source(Source::GroupInput)?;
    validate_data_size_in_source(Source::GroupOutput)?;
    Ok(())
}

pub fn genesis_id_conservation_and_consistency(
    ids: (Vec<GenesisId>, Vec<GenesisId>),
) -> Result<(), Error> {
    let (in_ids, out_ids) = ids;
    let curr_id = get_curr_genesis_id()?;
    let old_ids = out_ids
        .iter()
        .filter_map(|id| {
            if id != &curr_id {
                Some(id.clone())
            } else {
                None
            }
        })
        .collect::<Vec<GenesisId>>();

    let id_map = map_of_genesis_ids((in_ids, out_ids))?;

    // Commented out because should be impossible; uncomment during test for reachability
    // let in_out_count_curr_id = id_map.get(&curr_id);
    // if let Some(counts) = in_out_count_curr_id {
    //     let (in_count, out_count) = counts;
    //     if in_count != 0 {
    //         return Err(Error::InvalidGenesisIdInInput); // This should be impossible to raise this error
    //     }
    // }

    for id in old_ids {
        if let Some(counts) = id_map.get(&id) {
            let (in_count, out_count) = counts;
            if out_count > in_count {
                return Err(Error::PreExistingGenesisIdMinted);
            }
        }
    }

    Ok(())
}
