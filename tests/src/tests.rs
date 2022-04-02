use std::cmp::min;
use std::sync::{Arc, Mutex};

use super::*;
use ckb_hash::blake2b_256;

use trampoline_sdk::ckb_types::packed::{CellInput, CellInputBuilder, CellOutput};
// use ckb_testtool::context::Context;
// use ckb_testtool::ckb_types::{
//     bytes::Bytes,
//     core::TransactionBuilder,
//     packed::*,
//     prelude::*,
// };
// use ckb_testtool::ckb_error::Error;
use ckb_always_success_script::ALWAYS_SUCCESS;
use ckb_jsonrpc_types::JsonBytes;
use trampoline_sdk::chain::{MockChain, MockChainTxProvider as ChainRpc};
use trampoline_sdk::ckb_types::{
    self,
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    error::Error,
    packed::*,
    prelude::*,
    H256,
};
use trampoline_sdk::contract::*;
use trampoline_sdk::contract::{builtins::t_nft::*, generator::*};
use trampoline_sdk::contract::{schema::*, ContractSource};

// TO DO
// Should just add a Bytes type to trampoline which provides a single interface for all these
// Various byte types

// ALSO: Make generator pipeline able to handle empty data so it doesn't have to be set
const MAX_CYCLES: u64 = 10_000_000;

// error numbers
const ERROR_EMPTY_ARGS: i8 = 5;

fn assert_script_error(err: Error, err_code: i8) {
    let error_string = err.to_string();
    assert!(
        error_string.contains(format!("error code {} ", err_code).as_str()),
        "error_string: {}, expected_error_code: {}",
        error_string,
        err_code
    );
}

fn generate_always_success_lock(
    args: Option<ckb_types::packed::Bytes>,
) -> ckb_types::packed::Script {
    let data: Bytes = ckb_always_success_script::ALWAYS_SUCCESS.to_vec().into();
    let data_hash = H256::from(blake2b_256(data.to_vec().as_slice()));
    ckb_types::packed::Script::default()
        .as_builder()
        .args(args.unwrap_or([0u8].pack()))
        .code_hash(data_hash.pack())
        .hash_type(ckb_types::core::ScriptHashType::Data1.into())
        .build()
}

fn gen_nft_contract() -> TrampolineNFTContract {
    let bin = Loader::default().load_binary("trampoline-nft");
    let mut contract = TrampolineNFTContract::default();
    contract.code = Some(JsonBytes::from_bytes(bin));
    contract
}

fn gen_tnft_cell_output(contract: &TrampolineNFTContract) -> CellOutput {
    let lock = contract
        .lock
        .clone()
        .unwrap_or(generate_always_success_lock(None).into());

    CellOutput::new_builder()
        .capacity(200_u64.pack())
        .type_(
            Some(ckb_types::packed::Script::from(
                contract.as_script().unwrap(),
            ))
            .pack(),
        )
        .lock(lock.into())
        .build()
}

fn generate_mock_tx(
    inputs: Vec<CellInput>,
    outputs: Vec<CellOutput>,
    outputs_data: Vec<ckb_types::packed::Bytes>,
) -> TransactionView {
    TransactionBuilder::default()
        .outputs(outputs)
        .outputs_data(outputs_data)
        .build()
}

fn genesis_id_from(input: OutPoint) -> GenesisId {
    let seed_tx_hash = input.tx_hash();
    let seed_idx = input.index();
    let mut seed = Vec::with_capacity(36);
    seed.extend_from_slice(seed_tx_hash.as_slice());
    seed.extend_from_slice(seed_idx.as_slice());
    let hash = blake2b_256(&seed);

    GenesisId::from_mol(hash.pack())
}

type NftArgs = SchemaPrimitiveType<Bytes, ckb_types::packed::Bytes>;
type NftField = ContractCellField<NftArgs, TrampolineNFT>;
#[test]
fn test_success_deploy() {
    let mut tnft_contract = gen_nft_contract();
    let mut chain = MockChain::default();
    let minter_lock_cell = chain.get_default_script_outpoint();
    let minter_lock_script = chain.build_script(&minter_lock_cell, vec![1_u8].into());

    let tx_input_cell = chain.deploy_random_cell_with_default_lock(2000, Some(vec![1_u8].into()));

    let tnft_code_cell = tnft_contract.as_code_cell();

    let tnft_code_cell_outpoint = chain.create_cell(tnft_code_cell.0, tnft_code_cell.1);
    tnft_contract.source = Some(ContractSource::Chain(
        tnft_code_cell_outpoint.clone().into(),
    ));

    let genesis_seed = genesis_id_from(tx_input_cell.clone());

    tnft_contract.add_input_rule(move |_tx| -> CellQuery {
        CellQuery {
            _query: QueryStatement::Single(CellQueryAttribute::LockHash(
                minter_lock_script
                    .clone()
                    .unwrap()
                    .calc_script_hash()
                    .into(),
            )),
            _limit: 1,
        }
    });

    tnft_contract.add_output_rule(ContractField::Data, move |ctx| -> NftField {
        let nft: NftField = ctx.load(ContractField::Data);
        if let ContractCellField::Data(nft_data) = nft {
            let mut t_nft_data = nft_data.clone();
            t_nft_data.genesis_id = genesis_seed.clone();
            NftField::Data(t_nft_data)
        } else {
            nft
        }
    });

    let chain_rpc = ChainRpc::new(chain);
    let generator = Generator::new()
        .chain_service(&chain_rpc)
        .query_service(&chain_rpc)
        .pipeline(vec![&tnft_contract]);
    let new_mint_tx = generator.generate(); //generator.pipe(tx_skeleton, Arc::new(Mutex::new(vec![])));
    let is_valid = chain_rpc.verify_tx(new_mint_tx.tx.into());
    assert!(is_valid);
}

#[test]
fn test_invalid_mismatched_genesis_id() {
    let mut tnft_contract = gen_nft_contract();
    let mut chain = MockChain::default();
    let minter_lock_cell = chain.get_default_script_outpoint();
    let minter_lock_script = chain.build_script(&minter_lock_cell, vec![1_u8].into());
    let other_lock_script = chain.build_script(&minter_lock_cell, vec![2_u8].into());

    let tx_input_cell = chain.deploy_random_cell_with_default_lock(2000, Some(vec![1_u8].into()));

    let genesis_id_seed_cell =
        chain.deploy_random_cell_with_default_lock(2000, Some(vec![2_u8].into()));

    println!(
        "TX INPUT CELL OUTPOINT: {:?}\nGENESIS ID CELL OUTPOINT {:?}",
        tx_input_cell, genesis_id_seed_cell
    );

    let tx_input_get_by_lock_hash = chain.get_cells_by_lock_hash(
        minter_lock_script
            .clone()
            .unwrap()
            .calc_script_hash()
            .into(),
    );
    let gen_id_cell_get_by_lock_hash =
        chain.get_cells_by_lock_hash(other_lock_script.clone().unwrap().calc_script_hash().into());
    println!(
        "RES OF TX_INPUT_GET {:?}\nRES OF GEN ID CELL GET {:?}",
        tx_input_get_by_lock_hash, gen_id_cell_get_by_lock_hash
    );
    let tnft_code_cell = tnft_contract.as_code_cell();

    let tnft_code_cell_outpoint = chain.create_cell(tnft_code_cell.0, tnft_code_cell.1);
    tnft_contract.source = Some(ContractSource::Chain(
        tnft_code_cell_outpoint.clone().into(),
    ));
    let genesis_seed = genesis_id_from(genesis_id_seed_cell.clone());

    let wrong_genesis_seed = genesis_id_from(tx_input_cell.clone());

    println!(
        "GEN SEED EXPECTED: {:?}\nGen SEED USED: {:?}",
        genesis_seed.to_mol(),
        wrong_genesis_seed.to_mol()
    );

    tnft_contract.add_input_rule(move |_tx| -> CellQuery {
        CellQuery {
            _query: QueryStatement::Single(CellQueryAttribute::LockHash(
                minter_lock_script
                    .clone()
                    .unwrap()
                    .calc_script_hash()
                    .into(),
            )),
            _limit: 1,
        }
    });

    tnft_contract.add_output_rule(ContractField::Data, move |ctx| -> NftField {
        let nft: NftField = ctx.load(ContractField::Data);
        if let ContractCellField::Data(nft_data) = nft {
            if ctx.idx == 0 {
                let mut t_nft_data = nft_data.clone();
                t_nft_data.genesis_id = genesis_seed.clone();
                NftField::Data(t_nft_data)
            } else {
                NftField::Data(nft_data)
            }
        } else {
            nft
        }
    });

    let chain_rpc = ChainRpc::new(chain);
    let generator = Generator::new()
        .chain_service(&chain_rpc)
        .query_service(&chain_rpc)
        .pipeline(vec![&tnft_contract]);
    let new_mint_tx = generator.generate(); //generator.pipe(tx_skeleton, Arc::new(Mutex::new(vec![])));
    let is_valid = chain_rpc.verify_tx(new_mint_tx.tx.into());
    assert!(!is_valid);
}

// TO DO: Finish Test; currently builds a tx identical to mint
#[test]
fn test_invalid_mint_of_pre_existing_tnft() {
    let mut tnft_contract = gen_nft_contract();
    tnft_contract.output_count(3);
    let mut chain = MockChain::default();
    let minter_lock_cell = chain.get_default_script_outpoint();
    let minter_lock_script = chain.build_script(&minter_lock_cell, vec![1_u8].into());
    let other_lock_script = chain.build_script(&minter_lock_cell, vec![2_u8].into());

    //let tx_input_cell = chain.deploy_random_cell_with_default_lock(2000, Some(vec![1_u8].into()));

    let prior_genesis_id_seed_cell =
        chain.deploy_random_cell_with_default_lock(2000, Some(vec![2_u8].into()));
    let genesis_id_seed_cell =
        chain.deploy_random_cell_with_default_lock(2000, Some(vec![1_u8].into()));

    let mut pre_existing_tnft_data = TrampolineNFT::default();
    pre_existing_tnft_data.genesis_id = genesis_id_from(prior_genesis_id_seed_cell);
    tnft_contract.set_data(pre_existing_tnft_data.clone());
    let tnft_output_cell = CellOutputBuilder::default()
        .capacity(200.pack())
        .type_(
            Some(ckb_types::packed::Script::from(
                tnft_contract.as_script().unwrap(),
            ))
            .pack(),
        )
        .lock(other_lock_script.unwrap())
        .build();
    let pre_existing_tnft =
        chain.deploy_cell_output(pre_existing_tnft_data.to_bytes(), tnft_output_cell.clone());
    println!(
        "PRE_EXISTING_NFT: {:?}\nITS OUTPOINT {:?}",
        tnft_output_cell, pre_existing_tnft
    );
    // println!(
    //     "TX INPUT CELL OUTPOINT: {:?}\nGENESIS ID CELL OUTPOINT {:?}",
    //     tx_input_cell, genesis_id_seed_cell
    // );

    // let tx_input_get_by_lock_hash = chain.get_cells_by_lock_hash(
    //     minter_lock_script
    //         .clone()
    //         .unwrap()
    //         .calc_script_hash()
    //         .into(),
    // );
    // let gen_id_cell_get_by_lock_hash =
    //     chain.get_cells_by_lock_hash(other_lock_script.clone().unwrap().calc_script_hash().into());
    // println!(
    //     "RES OF TX_INPUT_GET {:?}\nRES OF GEN ID CELL GET {:?}",
    //     tx_input_get_by_lock_hash, gen_id_cell_get_by_lock_hash
    // );
    let tnft_code_cell = tnft_contract.as_code_cell();

    let tnft_code_cell_outpoint = chain.create_cell(tnft_code_cell.0, tnft_code_cell.1);
    tnft_contract.source = Some(ContractSource::Chain(
        tnft_code_cell_outpoint.clone().into(),
    ));
    let genesis_seed = genesis_id_from(genesis_id_seed_cell.clone());

    //let wrong_genesis_seed = genesis_id_from(tx_input_cell.clone());

    // println!(
    //     "GEN SEED EXPECTED: {:?}\nGen SEED USED: {:?}",
    //     genesis_seed.to_mol(),
    //     wrong_genesis_seed.to_mol()
    // );

    tnft_contract.add_input_rule(move |_tx| -> CellQuery {
        CellQuery {
            _query: QueryStatement::Single(CellQueryAttribute::LockHash(
                minter_lock_script
                    .clone()
                    .unwrap()
                    .calc_script_hash()
                    .into(),
            )),
            _limit: 1,
        }
    });
    let nft_data_hash = ckb_jsonrpc_types::Byte32::from(tnft_contract.data_hash().unwrap().pack());
    println!("NFT DATA HASH PASSED TO QUERY: {:?}", nft_data_hash);
    tnft_contract.add_input_rule(move |_tx| -> CellQuery {
        CellQuery {
            _query: QueryStatement::Single(CellQueryAttribute::DataHash(nft_data_hash.clone())),
            _limit: 1,
        }
    });

    tnft_contract.add_output_rule(ContractField::Data, move |ctx| -> NftField {
        let nft: NftField = ctx.load(ContractField::Data);
        if let ContractCellField::Data(nft_data) = nft {
            println!("CTX INDEX: {}", ctx.idx);
            if ctx.idx == 0 {
                let mut t_nft_data = nft_data.clone();
                t_nft_data.genesis_id = genesis_seed.clone();
                NftField::Data(t_nft_data)
            } else if ctx.idx == 1 || ctx.idx == 2 {
                NftField::Data(pre_existing_tnft_data.clone())
            } else {
                NftField::Data(nft_data)
            }
        } else {
            nft
        }
    });

    let chain_rpc = ChainRpc::new(chain);
    let generator = Generator::new()
        .chain_service(&chain_rpc)
        .query_service(&chain_rpc)
        .pipeline(vec![&tnft_contract]);
    let new_mint_tx = generator.generate(); //generator.pipe(tx_skeleton, Arc::new(Mutex::new(vec![])));
    let is_valid = chain_rpc.verify_tx(new_mint_tx.tx.into());
    assert!(!is_valid);
}
// #[test]
// fn test_success() {
//     // deploy contract
//     let mut context = Context::default();
//     let contract_bin: Bytes = Loader::default().load_binary("trampoline-nft");
//     let out_point = context.deploy_cell(contract_bin);

//     // prepare scripts
//     let lock_script = context
//         .build_script(&out_point, Bytes::from(vec![42]))
//         .expect("script");
//     let lock_script_dep = CellDep::new_builder()
//         .out_point(out_point)
//         .build();

//     // prepare cells
//     let input_out_point = context.create_cell(
//         CellOutput::new_builder()
//             .capacity(1000u64.pack())
//             .lock(lock_script.clone())
//             .build(),
//         Bytes::new(),
//     );
//     let input = CellInput::new_builder()
//         .previous_output(input_out_point)
//         .build();
//     let outputs = vec![
//         CellOutput::new_builder()
//             .capacity(500u64.pack())
//             .lock(lock_script.clone())
//             .build(),
//         CellOutput::new_builder()
//             .capacity(500u64.pack())
//             .lock(lock_script)
//             .build(),
//     ];

//     let outputs_data = vec![Bytes::new(); 2];

//     // build transaction
//     let tx = TransactionBuilder::default()
//         .input(input)
//         .outputs(outputs)
//         .outputs_data(outputs_data.pack())
//         .cell_dep(lock_script_dep)
//         .build();
//     let tx = context.complete_tx(tx);

//     // run
//     let cycles = context
//         .verify_tx(&tx, MAX_CYCLES)
//         .expect("pass verification");
//     println!("consume cycles: {}", cycles);
// }

// #[test]
// fn test_empty_args() {
//     // deploy contract
//     let mut context = Context::default();
//     let contract_bin: Bytes = Loader::default().load_binary("trampoline-nft");
//     let out_point = context.deploy_cell(contract_bin);

//     // prepare scripts
//     let lock_script = context
//         .build_script(&out_point, Default::default())
//         .expect("script");
//     let lock_script_dep = CellDep::new_builder()
//         .out_point(out_point)
//         .build();

//     // prepare cells
//     let input_out_point = context.create_cell(
//         CellOutput::new_builder()
//             .capacity(1000u64.pack())
//             .lock(lock_script.clone())
//             .build(),
//         Bytes::new(),
//     );
//     let input = CellInput::new_builder()
//         .previous_output(input_out_point)
//         .build();
//     let outputs = vec![
//         CellOutput::new_builder()
//             .capacity(500u64.pack())
//             .lock(lock_script.clone())
//             .build(),
//         CellOutput::new_builder()
//             .capacity(500u64.pack())
//             .lock(lock_script)
//             .build(),
//     ];

//     let outputs_data = vec![Bytes::new(); 2];

//     // build transaction
//     let tx = TransactionBuilder::default()
//         .input(input)
//         .outputs(outputs)
//         .outputs_data(outputs_data.pack())
//         .cell_dep(lock_script_dep)
//         .build();
//     let tx = context.complete_tx(tx);

//     // run
//     let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
//     assert_script_error(err, ERROR_EMPTY_ARGS);
// }
