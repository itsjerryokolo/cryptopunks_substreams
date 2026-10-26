use crate::{
    abi,
    pb::cryptopunks as punks,
    utils::{
        constants::{CRYPTOPUNKS_CONTRACT, CRYPTOPUNKS_DATA_CONTRACT},
        helper::append_0x,
    },
};
use anyhow::Error;
use substreams::{scalar::BigInt, Hex};
use substreams_ethereum::rpc::RpcBatch;

pub struct PunkMetadata {
    pub attributes: String,
    /// Hex-encoded raw RGBA pixels returned by punkImage (not a PNG).
    pub image: String,
    pub svg: String,
}

pub fn get_contract_data() -> Result<punks::Contract, Error> {
    let address = CRYPTOPUNKS_CONTRACT.to_vec();
    let responses = RpcBatch::new()
        .add(abi::cryptopunks::functions::TotalSupply {}, address.clone())
        .add(abi::cryptopunks::functions::Name {}, address.clone())
        .add(abi::cryptopunks::functions::Symbol {}, address.clone())
        .add(abi::cryptopunks::functions::ImageHash {}, address)
        .execute()
        .map_err(Error::msg)?
        .responses;
    if responses.len() != 4 {
        return Err(Error::msg("incomplete contract metadata RPC batch"));
    }
    Ok(punks::Contract {
        address: append_0x(&Hex(CRYPTOPUNKS_CONTRACT).to_string()),
        total_supply: RpcBatch::decode::<_, abi::cryptopunks::functions::TotalSupply>(
            &responses[0],
        )
        .ok_or_else(|| Error::msg("totalSupply reverted or returned invalid data"))?
        .to_string(),
        name: RpcBatch::decode::<_, abi::cryptopunks::functions::Name>(&responses[1])
            .ok_or_else(|| Error::msg("name reverted or returned invalid data"))?,
        symbol: RpcBatch::decode::<_, abi::cryptopunks::functions::Symbol>(&responses[2])
            .ok_or_else(|| Error::msg("symbol reverted or returned invalid data"))?,
        image_hash: RpcBatch::decode::<_, abi::cryptopunks::functions::ImageHash>(&responses[3])
            .ok_or_else(|| Error::msg("imageHash reverted or returned invalid data"))?,
    })
}

pub fn get_punk_metadata(token_id: u64) -> Result<PunkMetadata, Error> {
    let address = CRYPTOPUNKS_DATA_CONTRACT.to_vec();
    let index = BigInt::from(token_id);
    let responses = RpcBatch::new()
        .add(
            abi::cryptopunks_data::functions::PunkAttributes {
                index: index.clone(),
            },
            address.clone(),
        )
        .add(
            abi::cryptopunks_data::functions::PunkImage {
                index: index.clone(),
            },
            address.clone(),
        )
        .add(
            abi::cryptopunks_data::functions::PunkImageSvg { index },
            address,
        )
        .execute()
        .map_err(Error::msg)?
        .responses;
    if responses.len() != 3 {
        return Err(Error::msg("incomplete punk metadata RPC batch"));
    }
    Ok(PunkMetadata {
        attributes: RpcBatch::decode::<_, abi::cryptopunks_data::functions::PunkAttributes>(
            &responses[0],
        )
        .ok_or_else(|| Error::msg(format!("punkAttributes failed for {token_id}")))?,
        image: append_0x(
            &Hex(
                RpcBatch::decode::<_, abi::cryptopunks_data::functions::PunkImage>(&responses[1])
                    .ok_or_else(|| Error::msg(format!("punkImage failed for {token_id}")))?,
            )
            .to_string(),
        ),
        svg: RpcBatch::decode::<_, abi::cryptopunks_data::functions::PunkImageSvg>(&responses[2])
            .ok_or_else(|| Error::msg(format!("punkImageSvg failed for {token_id}")))?,
    })
}
