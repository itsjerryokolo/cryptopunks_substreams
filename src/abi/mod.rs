// ABI generator output is linted upstream; lint handwritten modules normally.
#![allow(clippy::all)]
pub mod cryptopunks {
    include!(concat!(env!("OUT_DIR"), "/cryptopunks.rs"));
}
pub mod cryptopunks_data {
    include!(concat!(env!("OUT_DIR"), "/cryptopunks_data.rs"));
}
pub mod wrappedpunks {
    include!(concat!(env!("OUT_DIR"), "/wrappedpunks.rs"));
}
