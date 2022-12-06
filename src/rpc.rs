use crate::{abi, utils};
use utils::constants::CRYPTOPUNKS_CONTRACT;

use substreams_ethereum::rpc::RpcBatch;

pub fn get_contract_data() -> Option<(String, String, String, String)> {
    let tup: Option<(String, String, String, String)>;
    let contract_address = CRYPTOPUNKS_CONTRACT.to_vec();
    let batch_calls = RpcBatch::new();
    let responses = batch_calls
        .add(
            abi::cryptopunks::functions::TotalSupply {},
            contract_address.clone(),
        )
        .add(
            abi::cryptopunks::functions::Name {},
            contract_address.clone(),
        )
        .add(
            abi::cryptopunks::functions::Symbol {},
            contract_address.clone(),
        )
        .add(
            abi::cryptopunks::functions::ImageHash {},
            contract_address.clone(),
        )
        .execute()
        .unwrap()
        .responses;

    let total_supply =
        match RpcBatch::decode::<_, abi::cryptopunks::functions::TotalSupply>(&responses[0]) {
            Some(contract_total_supply) => contract_total_supply.to_string(),
            None => "Total Supply Call Reverted".to_string(),
        };
    let name = match RpcBatch::decode::<_, abi::cryptopunks::functions::Name>(&responses[1]) {
        Some(contract_name) => contract_name.to_string(),
        None => "Name Call Reverted".to_string(),
    };
    let symbol = match RpcBatch::decode::<_, abi::cryptopunks::functions::Symbol>(&responses[2]) {
        Some(contract_symbol) => contract_symbol.to_string(),
        None => "Symbol Call Reverted".to_string(),
    };
    let image_hash =
        match RpcBatch::decode::<_, abi::cryptopunks::functions::ImageHash>(&responses[3]) {
            Some(contract_image_hash) => contract_image_hash.to_string(),
            None => "ImageHash Call Reverted".to_string(),
        };

    tup = Some((total_supply, name, symbol, image_hash));
    tup
}