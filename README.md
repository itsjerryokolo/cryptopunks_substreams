# Cryptopunks Substreams 

```mermaid
graph TD;
  map_assigns[map: map_assigns]
  sf.ethereum.type.v2.Block[source: sf.ethereum.type.v2.Block] --> map_assigns
  store_assigns[store: store_assigns]
  map_assigns --> store_assigns
  map_bids[map: map_bids]
  sf.ethereum.type.v2.Block[source: sf.ethereum.type.v2.Block] --> map_bids
  bids_state[store: bids_state]
  map_bids --> bids_state
  store_sales --> bids_state
  store_volume[store: store_volume]
  map_sales --> store_volume
  bids_state --> store_volume
  store_sales[store: store_sales]
  map_sales --> store_sales
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
  store_sales --> asks_state
  contract_metadata[store: contract_metadata]
  map_assigns --> contract_metadata
  map_metadata[map: map_metadata]
  sf.ethereum.type.v2.Block[source: sf.ethereum.type.v2.Block] --> map_metadata
  store_metadata[store: store_metadata]
  map_metadata --> store_metadata
  map_metadata_entities[map: map_metadata_entities]
  store_metadata -- deltas --> map_metadata_entities
  map_contract_entities[map: map_contract_entities]
  contract_metadata -- deltas --> map_contract_entities
  map_transfer_entities[map: map_transfer_entities]
  punk_state -- deltas --> map_transfer_entities
  map_assign_entities[map: map_assign_entities]
  store_assigns -- deltas --> map_assign_entities
  map_ask_entities[map: map_ask_entities]
  asks_state -- deltas --> map_ask_entities
  map_bid_entities[map: map_bid_entities]
  bids_state -- deltas --> map_bid_entities
  map_sale_entities[map: map_sale_entities]
  store_sales -- deltas --> map_sale_entities
  graph_out[map: graph_out]
  map_metadata_entities --> graph_out
  map_contract_entities --> graph_out
  map_transfer_entities --> graph_out
  map_assign_entities --> graph_out
  map_ask_entities --> graph_out
  map_bid_entities --> graph_out
  map_sale_entities --> graph_out
```