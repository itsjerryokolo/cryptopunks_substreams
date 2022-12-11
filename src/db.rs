use crate::pb::cryptopunks as punks;

use substreams::scalar::BigInt;
use substreams::store::DeltaString;
use substreams_entity_change::pb::entity::{entity_change::Operation, EntityChanges};

use substreams::store::{DeltaProto, Deltas};

// -------------------
//  Map Metadata Entities
// -------------------
pub fn map_metadata_entity_change(entity_changes: &mut EntityChanges) {
    entity_changes
        .push_change("MetaData", "0", 1, Operation::Create)
        .change("id", "0".to_string())
        .change("tokenId", BigInt::zero())
        .change("tokenURI", "".to_string())
        .change("image", "".to_string())
        .change("svg", "".to_string())
        .change("contractURI", "".to_string())
        .change("punk", "".to_string())
        .change("traits", "".to_string());
}

pub fn store_metadata_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Metadata>>,
) {
    for delta in deltas.deltas {
        let punk_id = delta.key.as_str().split(":").last().unwrap();

        entity_changes
            .push_change("MetaData", punk_id, delta.ordinal, Operation::Update)
            .change(
                "tokenId",
                DeltaString {
                    operation: delta.operation,
                    ordinal: 0,
                    key: punk_id.to_string(),
                    old_value: delta.old_value.token_id,
                    new_value: delta.new_value.token_id,
                },
            )
            .change(
                "tokenURI",
                DeltaString {
                    operation: delta.operation,
                    ordinal: 0,
                    key: punk_id.to_string(),
                    old_value: delta.old_value.token_uri,
                    new_value: delta.new_value.token_uri,
                },
            )
            .change(
                "image",
                DeltaString {
                    operation: delta.operation,
                    ordinal: 0,
                    key: punk_id.to_string(),
                    old_value: delta.old_value.image,
                    new_value: delta.new_value.image,
                },
            )
            .change(
                "contractURI",
                DeltaString {
                    operation: delta.operation,
                    ordinal: 0,
                    key: punk_id.to_string(),
                    old_value: delta.old_value.contract_uri,
                    new_value: delta.new_value.contract_uri,
                },
            )
            .change(
                "traits",
                DeltaString {
                    operation: delta.operation,
                    ordinal: 0,
                    key: punk_id.to_string(),
                    old_value: delta.old_value.traits,
                    new_value: delta.new_value.traits,
                },
            );
    }
}
