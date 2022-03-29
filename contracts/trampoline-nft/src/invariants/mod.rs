// Rules:
// 1. Any nft in output that is NOT in input must have correct genesis id
// 2. For each genesis ID in input: count(input) >= count(output) -- i.e., there cannot be newly created NFTs with a pre-existing
//    genesis ID
// 3. Each nft data field >= 64 bytes
// 4. Arg field is at least 34 bytes
// 5. If args[0] == 1, then script in cell_deps with matching hash of args[2..34] must be executed
// 6. If args[0] == 0, then script in cell_deps with matching hash of args[2..34] is executed only if present
// 7. If args[1] == HashType::Data, then use cell_deps data hash, else use cell_deps type hash

pub fn minimum_data_size() {}

pub fn new_nfts_match_curr_genesis_id() {}

pub fn no_non_genesis_nfts_were_minted() {}

pub fn extension_is_executed() {}