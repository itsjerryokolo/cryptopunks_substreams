use crate::{abi, utils};
use std::str::FromStr;
use utils::constants::{CRYPTOPUNKS_CONTRACT, CRYPTOPUNKS_DATA_CONTRACT};

use substreams::scalar::BigInt;
use substreams_ethereum::rpc::RpcBatch;

pub fn get_contract_data() -> (String, String, String, String) {
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

    let tup: (String, String, String, String) = (total_supply, name, symbol, image_hash);
    tup
}

pub fn get_punk_metadata(punk: &str) -> (String, String, String) {
    let contract_address = CRYPTOPUNKS_DATA_CONTRACT.to_vec();
    let punk_index = BigInt::from_str(punk).unwrap();

    let batch_calls = RpcBatch::new();
    let responses = batch_calls
        .add(
            abi::cryptopunks_data::functions::PunkAttributes {
                index: punk_index.clone(),
            },
            contract_address.clone(),
        )
        .add(
            abi::cryptopunks_data::functions::PunkImage {
                index: punk_index.clone(),
            },
            contract_address.clone(),
        )
        .add(
            abi::cryptopunks_data::functions::PunkImageSvg {
                index: punk_index.clone(),
            },
            contract_address.clone(),
        )
        .execute()
        .unwrap()
        .responses;

    let attributes: String = match RpcBatch::decode::<
        _,
        abi::cryptopunks_data::functions::PunkAttributes,
    >(&responses[0])
    {
        Some(contract_attributes) => contract_attributes,
        None => "Attributes Call Reverted".to_string(),
    };
    let punk_image =
        match RpcBatch::decode::<_, abi::cryptopunks_data::functions::PunkImage>(&responses[1]) {
            Some(contract_image) => contract_image.iter().map(|x| x.to_string()).collect(),
            None => "Image Call Reverted".to_string(),
        };
    let punk_image_svg = match RpcBatch::decode::<_, abi::cryptopunks_data::functions::PunkImageSvg>(
        &responses[2],
    ) {
        Some(contract_image_svg) => contract_image_svg,
        None => "Symbol Call Reverted".to_string(),
    };

    let tup: (String, String, String) = (attributes, punk_image, punk_image_svg);
    tup
}
