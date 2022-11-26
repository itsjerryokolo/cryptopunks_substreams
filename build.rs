use anyhow::{Ok, Result};
use substreams_ethereum::Abigen;

fn main() -> Result<(), anyhow::Error> {
    Abigen::new("Cryptopunks", "abi/cryptopunks.json")?
        .generate()?
        .write_to_file("src/abi/cryptopunks.rs")?;

    Ok(())
}
