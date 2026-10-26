`sf/substreams/sink/database/v1/database.proto` is copied without modifications
from StreamingFast's substreams-sink-database-changes v4.0.0:
https://github.com/streamingfast/substreams-sink-database-changes/blob/v4.0.0/proto/sf/substreams/sink/database/v1/database.proto

The manifest imports that same official package for its descriptors. The local
copy only generates compatible Rust bindings with this repository's pinned
prost 0.11; no custom wire format or SDK-wide upgrade is introduced.
See the upstream license in `DATABASE_LICENSE`.
