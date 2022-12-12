use crate::utils::keyer::generate_id;

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
            .push_change(
                "MetaData",
                &punk_id.to_string(),
                delta.ordinal,
                Operation::Create,
            )
            .change("id", &delta.new_value.token_id)
            .change(
                "tokenId",
                BigInt::from_str(&delta.new_value.token_id).unwrap(),
            )
            .change("tokenURI", delta.new_value.token_uri)
            .change("image", delta.new_value.image)
            .change("svg", delta.new_value.svg)
            .change("contractURI", delta.new_value.contract_uri)
            .change("type", delta.new_value.punk_type)
            .change("traits", delta.new_value.traits);
    }
}

// -------------------
//  Map Contract Entity
// -------------------

//CREATE
pub fn store_contract_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Contract>>,
) {
    for delta in deltas.deltas {
        let contract_address = delta.key.as_str().split(":").last().unwrap().trim();

        entity_changes
            .push_change(
                "Contract",
                contract_address,
                delta.ordinal,
                Operation::Create,
            )
            .change("id", delta.new_value.address)
            .change("symbol", delta.new_value.symbol)
            .change("name", delta.new_value.name)
            .change("imageHash", delta.new_value.image_hash);
    }
}

// -------------------
//  Map Transfer Entity
// -------------------

//CREATE
pub fn create_transfer_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Transfer>>,
) {
    for delta in deltas.deltas {
        let punk_id = delta.key.as_str().split(":").last().unwrap().trim();

        let entity_id = generate_id(
            &delta.new_value.trx_hash,
            delta.new_value.ordinal.to_string().as_str(),
            "TRANSFER",
        );
        entity_changes
            .push_change(
                "Transfer",
                entity_id.as_str(),
                delta.ordinal,
                Operation::Create,
            )
            .change("id", &entity_id)
            .change("from", delta.new_value.from)
            .change("to", delta.new_value.to)
            .change("nft", punk_id.to_string())
            .change("wrapped", delta.new_value.wrapped)
            .change("txHash", delta.new_value.trx_hash)
            .change("blockNumber", delta.new_value.block_number)
            .change("timestamp", delta.new_value.timestamp)
            .change("logNumber", delta.new_value.ordinal);
    }
}
