use super::*;
use ckb_hash::blake2b_256;

// use ckb_testtool::context::Context;
// use ckb_testtool::ckb_types::{
//     bytes::Bytes,
//     core::TransactionBuilder,
//     packed::*,
//     prelude::*,
// };
// use ckb_testtool::ckb_error::Error;
use trampoline_sdk::ckb_types::{self, error::Error, bytes::Bytes, prelude::*, H256};
use trampoline_sdk::chain::{MockChain, MockChainTxProvider as ChainRpc};
use trampoline_sdk::contract::*;
use trampoline_sdk::contract::{builtins::t_nft::*, generator::*};
use ckb_always_success_script::ALWAYS_SUCCESS;
use ckb_jsonrpc_types::JsonBytes;

// Should just add a Bytes type to trampoline which provides a single interface for all these
// Various byte types
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

 #[test]
 fn test_success_deploy() {
     let mut NftContract = TrampolineNFTContract::default();
     let mut chain = MockChain::default(); 
     let minter_lock_code_cell_data: Bytes =
     ckb_always_success_script::ALWAYS_SUCCESS.to_vec().into();
     let minter_lock_cell = chain.deploy_cell_with_data(minter_lock_code_cell_data);
     let minter_lock_script = chain.build_script(&minter_lock_cell, vec![1_u8].into());


        
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
