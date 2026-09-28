use crate::utils::{constants::CRYPTOPUNKS_CONTRACT, helper::append_0x, keyer::generate_id};

use anyhow::Error;
use std::str::FromStr;
use substreams::Hex;

use crate::pb::cryptopunks as punks;
use substreams::scalar::{BigDecimal, BigInt};
use substreams_entity_change::pb::entity::{entity_change::Operation, EntityChanges};

use substreams::store::{DeltaProto, Deltas};

// -------------------
//  Map Immutable Metadata Entities
// -------------------

//CREATE
pub fn store_metadata_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Metadata>>,
) -> Result<(), Error> {
    for delta in deltas.deltas {
        let punk_id = delta.key.as_str().rsplit(':').next().unwrap().trim();

        entity_changes
            .push_change("MetaData", punk_id, delta.ordinal, Operation::Create)
            .change("id", &delta.new_value.token_id)
            .change("tokenId", BigInt::from_str(&delta.new_value.token_id)?)
            .change("punk", delta.new_value.token_id.clone())
            .change("tokenURI", delta.new_value.token_uri)
            .change("image", delta.new_value.image)
            .change("svg", delta.new_value.svg)
            .change("contractURI", delta.new_value.contract_uri)
            .change("type", delta.new_value.punk_type)
            .change("traits", delta.new_value.traits);
    }
    Ok(())
}

// -------------------
//  Map Contract Entity
// -------------------

//CREATE
pub fn store_contract_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Contract>>,
) -> Result<(), Error> {
    for delta in deltas.deltas {
        let contract_address = delta.new_value.address.clone();

        entity_changes
            .push_change(
                "Contract",
                &contract_address,
                delta.ordinal,
                Operation::Create,
            )
            .change("id", delta.new_value.address)
            .change("symbol", delta.new_value.symbol)
            .change("name", delta.new_value.name)
            .change(
                "totalSupply",
                BigInt::from_str(&delta.new_value.total_supply)?,
            )
            .change("imageHash", delta.new_value.image_hash);
    }
    Ok(())
}

// -------------------
//  Map Immutable Transfer Entities
// -------------------

//CREATE
pub fn create_transfer_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Transfer>>,
) -> Result<(), Error> {
    for delta in deltas.deltas {
        let punk_id = delta.key.as_str().rsplit(':').next().unwrap().trim();

        let entity_id = generate_id(
            &delta.new_value.trx_hash,
            delta.new_value.log_index.to_string().as_str(),
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
            .change(
                "from",
                hex::decode(delta.new_value.from.trim_start_matches("0x"))?,
            )
            .change(
                "to",
                hex::decode(delta.new_value.to.trim_start_matches("0x"))?,
            )
            .change("nft", punk_id.to_string())
            .change("wrapped", delta.new_value.wrapped.parse::<bool>()?)
            .change("type", "TRANSFER".to_string())
            .change(
                "txHash",
                hex::decode(delta.new_value.trx_hash.trim_start_matches("0x"))?,
            )
            .change(
                "blockHash",
                hex::decode(delta.new_value.block_hash.trim_start_matches("0x"))?,
            )
            .change("blockNumber", delta.new_value.block_number)
            .change("timestamp", delta.new_value.timestamp)
            .change("logNumber", u64::from(delta.new_value.log_index));
    }
    Ok(())
}

// -------------------
//  Map Immutable Assign Entities
// -------------------

//CREATE
pub fn create_assign_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Assign>>,
) -> Result<(), Error> {
    for delta in deltas.deltas {
        if !delta.key.starts_with("Punk: ") {
            continue;
        }

        let entity_id = generate_id(
            &delta.new_value.trx_hash,
            delta.new_value.log_index.to_string().as_str(),
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
            .change(
                "to",
                hex::decode(delta.new_value.to.trim_start_matches("0x"))?,
            )
            .change("nft", delta.new_value.token_id.to_string())
            .change(
                "contract",
                append_0x(&Hex(CRYPTOPUNKS_CONTRACT).to_string()),
            )
            .change("type", "ASSIGN".to_string())
            .change(
                "txHash",
                hex::decode(delta.new_value.trx_hash.trim_start_matches("0x"))?,
            )
            .change(
                "blockHash",
                hex::decode(delta.new_value.block_hash.trim_start_matches("0x"))?,
            )
            .change("blockNumber", delta.new_value.block_number)
            .change("timestamp", delta.new_value.timestamp)
            .change("logNumber", u64::from(delta.new_value.log_index));
    }
    Ok(())
}

// -------------------
//  Map Immutable Ask Entities
// -------------------

//CREATE
pub fn create_ask_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Ask>>,
) -> Result<(), Error> {
    for delta in deltas.deltas {
        if !delta.key.starts_with("Punk: ") {
            continue;
        }

        let entity_id = generate_id(
            &delta.new_value.trx_hash,
            delta.new_value.log_index.to_string().as_str(),
            "ASK",
        );

        let amount = BigDecimal::from_str(delta.new_value.amount.as_deref().unwrap_or("0"))?;
        entity_changes
            .push_change("Ask", entity_id.as_str(), delta.ordinal, Operation::Create)
            .change("id", &entity_id)
            .change(
                "from",
                hex::decode(delta.new_value.from.trim_start_matches("0x"))?,
            )
            .change("open", delta.new_value.open.parse::<bool>()?)
            .change("nft", delta.new_value.token_id.to_string())
            .change("amount", amount)
            .change("offerType", "ASK".to_string())
            .change(
                "txHash",
                hex::decode(delta.new_value.trx_hash.trim_start_matches("0x"))?,
            )
            .change(
                "blockHash",
                hex::decode(delta.new_value.block_hash.trim_start_matches("0x"))?,
            )
            .change("blockNumber", delta.new_value.block_number)
            .change("timestamp", delta.new_value.timestamp)
            .change("logNumber", u64::from(delta.new_value.log_index));
    }
    Ok(())
}

// -------------------
//  Map Immutable Bid Entities
// -------------------

//CREATE
pub fn create_bid_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Bid>>,
) -> Result<(), Error> {
    for delta in deltas.deltas {
        if !delta.key.starts_with("Punk: ") {
            continue;
        }

        let entity_id = generate_id(
            &delta.new_value.trx_hash,
            delta.new_value.log_index.to_string().as_str(),
            "BID",
        );

        let amount = BigDecimal::from_str(&delta.new_value.amount)?;
        entity_changes
            .push_change("Bid", entity_id.as_str(), delta.ordinal, Operation::Create)
            .change("id", &entity_id)
            .change(
                "from",
                hex::decode(delta.new_value.from.trim_start_matches("0x"))?,
            )
            .change("open", delta.new_value.open.parse::<bool>()?)
            .change("nft", delta.new_value.token_id.to_string())
            .change("amount", amount)
            .change("offerType", "BID".to_string())
            .change(
                "txHash",
                hex::decode(delta.new_value.trx_hash.trim_start_matches("0x"))?,
            )
            .change(
                "blockHash",
                hex::decode(delta.new_value.block_hash.trim_start_matches("0x"))?,
            )
            .change("blockNumber", delta.new_value.block_number)
            .change("timestamp", delta.new_value.timestamp)
            .change("logNumber", u64::from(delta.new_value.log_index));
    }
    Ok(())
}

// -------------------
//  Map Immutable Sale Entities
// -------------------

//CREATE
pub fn create_sale_entity_change(
    entity_changes: &mut EntityChanges,
    deltas: Deltas<DeltaProto<punks::Sale>>,
) -> Result<(), Error> {
    for delta in deltas.deltas {
        if !delta.key.starts_with("Punk: ") {
            continue;
        }
        let entity_id = generate_id(
            &delta.new_value.trx_hash,
            delta.new_value.log_index.to_string().as_str(),
            "SALE",
        );

        let amount = BigDecimal::from_str(&delta.new_value.amount)?;
        entity_changes
            .push_change("Sale", entity_id.as_str(), delta.ordinal, Operation::Create)
            .change("id", &entity_id)
            .change(
                "from",
                hex::decode(delta.new_value.from.trim_start_matches("0x"))?,
            )
            .change("nft", delta.new_value.token_id.to_string())
            .change("amount", amount)
            .change("type", "SALE".to_string())
            .change(
                "to",
                hex::decode(delta.new_value.to.trim_start_matches("0x"))?,
            )
            .change(
                "txHash",
                hex::decode(delta.new_value.trx_hash.trim_start_matches("0x"))?,
            )
            .change(
                "blockHash",
                hex::decode(delta.new_value.block_hash.trim_start_matches("0x"))?,
            )
            .change("blockNumber", delta.new_value.block_number)
            .change("timestamp", delta.new_value.timestamp)
            .change("logNumber", u64::from(delta.new_value.log_index));
    }
    Ok(())
}
