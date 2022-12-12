use std::str::FromStr;

use crate::pb::cryptopunks as punks;

use substreams::scalar::BigInt;
use substreams_entity_change::pb::entity::{entity_change::Operation, EntityChanges};

use substreams::store::{DeltaProto, Deltas};

// -------------------
//  Map Metadata Entities
// -------------------

//CREATE
pub fn store_metadata_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Metadata>>,
) {
    for delta in deltas.deltas {
        let punk_id = delta.key.as_str().split(":").last().unwrap().trim();

        entity_changes
            .push_change("MetaData", punk_id, delta.ordinal, Operation::Create)
            .change("id", delta.new_value.token_id)
            .change("tokenId", BigInt::from_str(&punk_id).unwrap())
            .change("tokenURI", delta.new_value.token_uri)
            .change("image", delta.new_value.image)
            .change("svg", delta.new_value.svg)
            .change("contractURI", delta.new_value.contract_uri)
            .change("type", delta.new_value.punk_type)
            .change("traits", delta.new_value.traits);
    }
}

//CREATE
pub fn store_contract_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Contract>>,
) {
    for delta in deltas.deltas {
        let contract_address = delta.key.as_str();

        entity_changes
            .push_change(
                "Contract",
                contract_address,
                delta.ordinal,
                Operation::Create,
            )
            .change("id", delta.new_value.token_id)
            .change("symbol", BigInt::from_str(&punk_id).unwrap())
            .change("name", delta.new_value.name)
            .change("imageHash", delta.new_value.image_hash)
            .change("totalSales", delta.new_value.image)
            .change("totalSupply", delta.new_value.svg)
            .change("totalAmountTraded", delta.new_value.contract_uri);
    }
}

//UPDATE
// pub fn store_metadata_entity_change(
//     entity_changes: &mut EntityChanges,
//     deltas: Deltas<DeltaProto<punks::Metadata>>,
// ) {
//     for delta in deltas.deltas {
//         let punk_id = delta.key.as_str().split(":").last().unwrap().trim();

//         entity_changes
//             .push_change("MetaData", punk_id, delta.ordinal, Operation::Create)
//             .change(
//                 "tokenId",
//                 DeltaString {
//                     operation: delta.operation,
//                     ordinal: 0,
//                     key: punk_id.to_string(),
//                     old_value: delta.old_value.token_id,
//                     new_value: delta.new_value.token_id,
//                 },
//             )
//             .change(
//                 "tokenURI",
//                 DeltaString {
//                     operation: delta.operation,
//                     ordinal: 0,
//                     key: punk_id.to_string(),
//                     old_value: delta.old_value.token_uri,
//                     new_value: delta.new_value.token_uri,
//                 },
//             )
//             .change(
//                 "image",
//                 DeltaString {
//                     operation: delta.operation,
//                     ordinal: 0,
//                     key: punk_id.to_string(),
//                     old_value: delta.old_value.image,
//                     new_value: delta.new_value.image,
//                 },
//             )
//             .change(
//                 "contractURI",
//                 DeltaString {
//                     operation: delta.operation,
//                     ordinal: 0,
//                     key: punk_id.to_string(),
//                     old_value: delta.old_value.contract_uri,
//                     new_value: delta.new_value.contract_uri,
//                 },
//             )
//             .change(
//                 "traits",
//                 DeltaString {
//                     operation: delta.operation,
//                     ordinal: 0,
//                     key: punk_id.to_string(),
//                     new_value: delta.new_value.traits,
//                 },
//             );
//     }
// }
