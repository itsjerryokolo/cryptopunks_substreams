# Cryptopunks Substreams 


```mermaid
graph TD;
  map_assigns[map: map_assigns]
  sf.ethereum.type.v2.Block[source: sf.ethereum.type.v2.Block] --> map_assigns
  store_assigns[store: store_assigns]
  map_assigns --> store_assigns
  punks_assignees[store: punks_assignees]
  map_assigns --> punks_assignees
  map_bids[map: map_bids]
  sf.ethereum.type.v2.Block[source: sf.ethereum.type.v2.Block] --> map_bids
  bids_state[store: bids_state]
  map_bids --> bids_state
  store_punk_sales --> bids_state
  store_all_punks[store: store_all_punks]
  map_assigns --> store_all_punks
  store_total_volume[store: store_total_volume]
  map_sales --> store_total_volume
  bids_state --> store_total_volume
  store_punk_volume[store: store_punk_volume]
  map_sales --> store_punk_volume
  bids_state --> store_punk_volume
  store_punk_sales[store: store_punk_sales]
  map_sales --> store_punk_sales
  map_sales[map: map_sales]
  sf.ethereum.type.v2.Block[source: sf.ethereum.type.v2.Block] --> map_sales
  map_transfers[map: map_transfers]
  sf.ethereum.type.v2.Block[source: sf.ethereum.type.v2.Block] --> map_transfers
  punk_state[store: punk_state]
  map_transfers --> punk_state
  store_user_proxies --> punk_state
  map_user_proxies[map: map_user_proxies]
  sf.ethereum.type.v2.Block[source: sf.ethereum.type.v2.Block] --> map_user_proxies
  map_wrapped_transfers[map: map_wrapped_transfers]
  sf.ethereum.type.v2.Block[source: sf.ethereum.type.v2.Block] --> map_wrapped_transfers
  store_user_proxies[store: store_user_proxies]
  map_user_proxies --> store_user_proxies
  map_asks[map: map_asks]
  sf.ethereum.type.v2.Block[source: sf.ethereum.type.v2.Block] --> map_asks
  asks_state[store: asks_state]
  map_asks --> asks_state
  sf.ethereum.type.v2.Block[source: sf.ethereum.type.v2.Block] --> asks_state
  contract_metadata[store: contract_metadata]
  store_assigns --> contract_metadata
```