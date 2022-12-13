use crate::utils::{constants::CRYPTOPUNKS_CONTRACT, helper::append_0x, keyer::generate_id};

use std::str::FromStr;
use substreams::Hex;

use crate::pb::cryptopunks as punks;
use substreams::scalar::BigInt;
use substreams_entity_change::pb::entity::{entity_change::Operation, EntityChanges};

use substreams::store::{DeltaProto, Deltas};

// -------------------
//  Map Immutable Metadata Entities
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
//  Map Immutable Transfer Entities
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
            .change("type", "TRANSFER".to_string())
            .change("txHash", delta.new_value.trx_hash)
            .change("blockNumber", delta.new_value.block_number)
            .change("timestamp", delta.new_value.timestamp)
            .change("logNumber", delta.new_value.ordinal);
    }
}

// -------------------
//  Map Immutable Assign Entities
// -------------------

//CREATE
pub fn create_assign_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Assign>>,
) {
    for delta in deltas.deltas {
        let assignee = delta.key.as_str().split(":").last().unwrap().trim();

        let entity_id = generate_id(
            &delta.new_value.trx_hash,
            delta.new_value.ordinal.to_string().as_str(),
            "ASSIGN",
        );
        entity_changes
            .push_change(
                "Assign",
                entity_id.as_str(),
                delta.ordinal,
                Operation::Create,
            )
            .change("id", &entity_id)
            .change("from", "".to_string())
            .change("to", assignee.to_string())
            .change("nft", delta.new_value.token_id)
            .change(
                "contract",
                append_0x(&Hex(CRYPTOPUNKS_CONTRACT).to_string()),
            )
            .change("type", "ASSIGN".to_string())
            .change("txHash", delta.new_value.trx_hash)
            .change("blockNumber", delta.new_value.block_number)
            .change("timestamp", delta.new_value.timestamp)
            .change("logNumber", delta.new_value.ordinal);
    }
}

// -------------------
//  Map Immutable Ask Entities
// -------------------

//CREATE
pub fn create_ask_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Ask>>,
) {
    for delta in deltas.deltas {
        let entity_id = generate_id(
            &delta.new_value.trx_hash,
            delta.new_value.ordinal.to_string().as_str(),
            "ASK",
        );
        entity_changes
            .push_change("Ask", entity_id.as_str(), delta.ordinal, Operation::Create)
            .change("id", &entity_id)
            .change("from", delta.new_value.from)
            .change("open", delta.new_value.open)
            .change("nft", delta.new_value.token_id)
            .change("amount", delta.new_value.amount.unwrap_or(0.to_string()))
            .change("offerType", "ASK".to_string())
            .change("txHash", delta.new_value.trx_hash)
            .change("blockNumber", delta.new_value.block_number)
            .change("timestamp", delta.new_value.timestamp)
            .change("logNumber", delta.new_value.ordinal);
    }
}
