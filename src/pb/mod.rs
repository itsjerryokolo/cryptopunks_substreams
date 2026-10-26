pub mod cryptopunks {
    include!(concat!(env!("OUT_DIR"), "/eth.cryptopunks.v1.rs"));
}

pub mod database {
    include!(concat!(
        env!("OUT_DIR"),
        "/sf.substreams.sink.database.v1.rs"
    ));
}
