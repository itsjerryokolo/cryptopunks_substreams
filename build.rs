use std::{env, path::PathBuf};
use substreams_ethereum::Abigen;

fn main() -> anyhow::Result<()> {
    let out = PathBuf::from(env::var("OUT_DIR")?);
    for (name, source, target) in [
        ("Cryptopunks", "abi/cryptopunks.json", "cryptopunks.rs"),
        ("WrappedPunks", "abi/wrappedpunks.json", "wrappedpunks.rs"),
        (
            "CryptoPunksData",
            "abi/CryptoPunksData.json",
            "cryptopunks_data.rs",
        ),
    ] {
        println!("cargo:rerun-if-changed={source}");
        Abigen::new(name, source)?
            .generate()?
            .write_to_file(out.join(target))?;
    }
    println!("cargo:rerun-if-changed=proto/cryptopunks.proto");
    println!("cargo:rerun-if-changed=proto/sf/substreams/sink/database/v1/database.proto");
    prost_build::compile_protos(
        &[
            "proto/cryptopunks.proto",
            "proto/sf/substreams/sink/database/v1/database.proto",
        ],
        &["proto"],
    )?;
    Ok(())
}
