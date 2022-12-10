use crate::pb::cryptopunks as punks;

use substreams::{
    scalar::{BigDecimal, BigInt},
    store::{DeltaBigDecimal, DeltaBigInt, DeltaString},
};
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
        .change("tokenURI", BigInt::zero())
        .change("image", BigDecimal::zero())
        .change("svg", BigDecimal::zero())
        .change("contractURI", BigDecimal::zero())
        .change("punk", BigDecimal::zero())
        .change("traits", BigDecimal::zero());
}

pub fn store_metadata_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Metadata>>,
) {
    for delta in deltas.deltas {
        let name = delta.key.as_str().split(":").last().unwrap();

        entity_changes
            .push_change("MetaData", name, delta.ordinal, Operation::Update)
            .change(
                "tokenId",
                DeltaString {
                    operation: delta.operation,
                    ordinal: 0,
                    key: name.to_string(),
                    old_value: delta.old_value.token_id,
                    new_value: delta.new_value.token_id,
                },
            )
            .change(
                "tokenURI",
                DeltaString {
                    operation: delta.operation,
                    ordinal: 0,
                    key: name.to_string(),
                    old_value: delta.old_value.token_uri,
                    new_value: delta.new_value.token_uri,
                },
            );
    }
}
