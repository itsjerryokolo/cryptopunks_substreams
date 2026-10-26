use crate::{
    events,
    pb::cryptopunks as punks,
    state,
    utils::{
        constants::{CRYPTOPUNKS_CONTRACT, WRAPPEDPUNKS_CONTRACT},
        helper::{append_0x, get_traits, get_type},
        math::convert_and_divide,
    },
};
use ethabi::{
    ethereum_types::{H160, U256},
    Token,
};
use substreams_ethereum::pb::eth::v2::{
    Block, BlockHeader, Call, Log, TransactionReceipt, TransactionTrace,
};

fn addr(n: u8) -> Token {
    Token::Address(H160::repeat_byte(n))
}
fn uint(n: u64) -> Token {
    Token::Uint(U256::from(n))
}
fn address(n: u8) -> String {
    format!("0x{}", hex::encode([n; 20]))
}
fn log(name: &str, values: Vec<Token>, ordinal: u64, wrapped: bool) -> Log {
    let abi = if wrapped {
        include_bytes!("../abi/wrappedpunks.json").as_slice()
    } else {
        include_bytes!("../abi/cryptopunks.json").as_slice()
    };
    let contract = ethabi::Contract::load(abi).unwrap();
    let event = contract.event(name).unwrap();
    let mut topics = vec![event.signature().as_bytes().to_vec()];
    let mut data = Vec::new();
    for (param, value) in event.inputs.iter().zip(values) {
        if param.indexed {
            topics.push(ethabi::encode(&[value]));
        } else {
            data.push(value);
        }
    }
    Log {
        address: if wrapped {
            WRAPPEDPUNKS_CONTRACT.to_vec()
        } else {
            CRYPTOPUNKS_CONTRACT.to_vec()
        },
        topics,
        data: ethabi::encode(&data),
        ordinal,
        block_index: (ordinal / 10) as u32,
        ..Default::default()
    }
}
fn block(logs: Vec<Log>) -> Block {
    Block {
        number: 14_000_000,
        hash: vec![0xbb; 32],
        header: Some(BlockHeader {
            timestamp: Some(prost_types::Timestamp {
                seconds: 1_600_000_000,
                nanos: 0,
            }),
            ..Default::default()
        }),
        transaction_traces: vec![TransactionTrace {
            hash: vec![0xaa; 32],
            from: vec![1; 20],
            to: CRYPTOPUNKS_CONTRACT.to_vec(),
            status: 1,
            receipt: Some(TransactionReceipt {
                logs,
                ..Default::default()
            }),
            ..Default::default()
        }],
        ..Default::default()
    }
}
fn sale() -> punks::Sale {
    punks::Sale {
        from: address(1),
        to: address(2),
        token_id: 42,
        amount: "2.5".into(),
        trx_hash: format!("0x{}", "aa".repeat(32)),
        block_hash: format!("0x{}", "bb".repeat(32)),
        ordinal: 100,
        log_index: 3,
        block_number: 14_000_000,
        timestamp: 1_600_000_000,
        ..Default::default()
    }
}
fn bid() -> punks::Bid {
    punks::Bid {
        from: address(2),
        token_id: 42,
        amount: "1.25".into(),
        open: "true".into(),
        ordinal: 50,
        ..Default::default()
    }
}

#[test]
fn prefix_is_idempotent_and_lowercase() {
    assert_eq!(append_0x("0xABCD"), "0xabcd");
    assert_eq!(append_0x(&append_0x("abcd")), "0xabcd");
}

#[test]
fn maps_reject_identical_signatures_from_other_contracts() {
    let mut logs = vec![
        log("Assign", vec![addr(2), uint(42)], 10, false),
        log(
            "PunkBought",
            vec![uint(42), uint(100), addr(1), addr(2)],
            20,
            false,
        ),
        log(
            "PunkBidEntered",
            vec![uint(42), uint(100), addr(2)],
            30,
            false,
        ),
        log(
            "PunkBidWithdrawn",
            vec![uint(42), uint(100), addr(2)],
            40,
            false,
        ),
        log("PunkOffered", vec![uint(42), uint(100), addr(2)], 50, false),
        log("PunkNoLongerForSale", vec![uint(42)], 60, false),
        log("ProxyRegistered", vec![addr(2), addr(3)], 70, true),
        log("PunkTransfer", vec![addr(1), addr(2), uint(42)], 80, false),
        log("Transfer", vec![addr(1), addr(2), uint(42)], 90, true),
    ];
    for log in &mut logs {
        log.address = vec![0xff; 20];
    }
    let b = block(logs);
    assert!(events::map_assigns(b.clone()).unwrap().assigns.is_empty());
    assert!(events::map_sales(b.clone()).unwrap().sales.is_empty());
    assert!(events::map_bids(b.clone()).unwrap().bids.is_empty());
    assert!(events::map_asks(b.clone()).unwrap().asks.is_empty());
    assert!(events::map_user_proxies(b.clone())
        .unwrap()
        .user_proxies
        .is_empty());
    assert!(events::map_transfers(b.clone())
        .unwrap()
        .transfers
        .is_empty());
    assert!(events::map_wrapped_transfers(b)
        .unwrap()
        .transfers
        .is_empty());
}

#[test]
fn assignments_are_unique_and_fetch_contract_once() {
    let mut b = block(vec![
        log("Assign", vec![addr(2), uint(42)], 10, false),
        log("Assign", vec![addr(3), uint(43)], 20, false),
    ]);
    b.number = 3_919_682;
    let mut calls = 0;
    let output = events::extract_assigns(&b, || {
        calls += 1;
        Ok(punks::Contract::default())
    })
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(output.assigns.len(), 2);
    assert!(output.assigns[0].contract.is_some());
    assert!(output.assigns[1].contract.is_none());
    assert_eq!(output.assigns[0].to, address(2));
}

#[test]
fn failed_transactions_do_not_emit_events() {
    let mut b = block(vec![log("Assign", vec![addr(2), uint(42)], 10, false)]);
    b.transaction_traces[0].status = 2;
    assert!(events::map_assigns(b).unwrap().assigns.is_empty());
}

#[test]
fn malformed_event_is_ignored() {
    let mut l = log(
        "PunkBought",
        vec![uint(42), uint(100), addr(1), addr(2)],
        10,
        false,
    );
    l.data.clear();
    assert!(events::map_sales(block(vec![l])).unwrap().sales.is_empty());
}

#[test]
fn transfer_keeps_firehose_ordinal_separate_from_log_index() {
    let out = events::map_transfers(block(vec![log(
        "PunkTransfer",
        vec![addr(1), addr(2), uint(42)],
        900,
        false,
    )]))
    .unwrap();
    assert_eq!(out.transfers[0].ordinal, 900);
    assert_eq!(out.transfers[0].log_index, 90);
    assert!(!out.transfers[0].trx_hash.starts_with("0x0x"));
}

#[test]
fn wrapped_transfers_include_mints_burns_and_regular_transfers() {
    let out = events::map_wrapped_transfers(block(vec![
        log("Transfer", vec![addr(0), addr(1), uint(42)], 10, true),
        log("Transfer", vec![addr(1), addr(2), uint(42)], 20, true),
        log("Transfer", vec![addr(2), addr(0), uint(42)], 30, true),
    ]))
    .unwrap();
    assert_eq!(out.transfers.len(), 3);
    assert_eq!(
        out.transfers
            .iter()
            .map(|t| t.wrapped.as_str())
            .collect::<Vec<_>>(),
        vec!["true", "true", "false"]
    );
}

#[test]
fn ask_uses_contract_caller_not_transaction_origin() {
    let l = log("PunkOffered", vec![uint(42), uint(100), addr(0)], 10, false);
    let mut b = block(vec![l.clone()]);
    b.transaction_traces[0].to = vec![9; 20];
    b.transaction_traces[0].calls = vec![Call {
        address: CRYPTOPUNKS_CONTRACT.to_vec(),
        caller: vec![7; 20],
        logs: vec![l],
        ..Default::default()
    }];
    assert_eq!(events::map_asks(b).unwrap().asks[0].from, address(7));
}

#[test]
fn accepted_bid_recovers_buyer_and_price_without_double_counting() {
    let b = block(vec![
        log("Transfer", vec![addr(1), addr(2), uint(1)], 90, false),
        log(
            "PunkBought",
            vec![uint(42), uint(0), addr(1), addr(0)],
            100,
            false,
        ),
    ]);
    let raw = events::map_sales(b).unwrap();
    assert!(raw.sales[0].bid_accepted);
    assert_eq!(raw.sales[0].to, address(2));
    let resolved = state::resolve_sales(raw, |_| Some(bid())).unwrap();
    assert_eq!(resolved.sales.len(), 1);
    assert_eq!(resolved.sales[0].amount, "1.25");
}

#[test]
fn direct_sale_never_adds_an_existing_bid() {
    let output = state::resolve_sales(
        punks::Sales {
            sales: vec![sale()],
        },
        |_| panic!("direct sale must not look up bids"),
    )
    .unwrap();
    assert_eq!(output.sales[0].amount, "2.5");
}

#[test]
fn accepted_bid_with_missing_or_wrong_bid_fails_explicitly() {
    let mut s = sale();
    s.bid_accepted = true;
    let input = punks::Sales { sales: vec![s] };
    assert!(state::resolve_sales(input.clone(), |_| None).is_err());
    assert!(state::resolve_sales(input, |_| Some(punks::Bid {
        from: address(9),
        ..bid()
    }))
    .is_err());
}

#[test]
fn sale_closes_bid_even_without_a_bid_event_in_that_block() {
    let out = state::bid_updates(
        punks::Bids::default(),
        punks::Sales {
            sales: vec![sale()],
        },
        punks::Transfers::default(),
        |_| Some(bid()),
    );
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].open, "false");
    assert_eq!(out[0].ordinal, 100);
}

#[test]
fn state_updates_remain_in_event_order_when_bid_follows_sale() {
    let later = punks::Bid {
        ordinal: 200,
        ..bid()
    };
    let out = state::bid_updates(
        punks::Bids { bids: vec![later] },
        punks::Sales {
            sales: vec![sale()],
        },
        punks::Transfers::default(),
        |_| Some(bid()),
    );
    assert_eq!(
        out.iter()
            .map(|b| (b.ordinal, b.open.as_str()))
            .collect::<Vec<_>>(),
        vec![(100, "false"), (200, "true")]
    );
}

#[test]
fn asks_do_not_require_sales_and_sales_close_them_in_order() {
    let ask = punks::Ask {
        token_id: 42,
        open: "true".into(),
        ordinal: 50,
        ..Default::default()
    };
    assert_eq!(
        state::ask_updates(
            punks::Asks {
                asks: vec![ask.clone()]
            },
            punks::Sales::default()
        )
        .len(),
        1
    );
    let out = state::ask_updates(
        punks::Asks { asks: vec![ask] },
        punks::Sales {
            sales: vec![sale()],
        },
    );
    assert_eq!(
        out.iter().map(|a| a.open.as_str()).collect::<Vec<_>>(),
        vec!["true", "false"]
    );
}

#[test]
fn amounts_preserve_wei_and_full_uint256_precision() {
    assert_eq!(
        convert_and_divide("1").unwrap().to_string(),
        "0.000000000000000001"
    );
    assert_eq!(
        convert_and_divide("1000000000000000000")
            .unwrap()
            .to_string(),
        "1"
    );
    let max = U256::MAX.to_string();
    let scaled = convert_and_divide(&max).unwrap()
        * substreams::scalar::BigDecimal::from(1_000_000_000_000_000_000u64);
    assert_eq!(
        scaled,
        max.parse::<substreams::scalar::BigDecimal>().unwrap()
    );
    assert!(convert_and_divide("invalid").is_err());
}

#[test]
fn metadata_window_has_exactly_ten_thousand_distinct_tokens() {
    assert_eq!(events::metadata_token(13_047_090), None);
    assert_eq!(events::metadata_token(13_047_091), Some(9999));
    assert_eq!(events::metadata_token(13_057_090), Some(0));
    assert_eq!(events::metadata_token(13_057_091), None);
}

#[test]
fn metadata_traits_preserve_digits_and_nonhuman_first_accessory() {
    assert_eq!(
        get_traits("Female 2, Mohawk, Nerd Glasses"),
        "Mohawk,Nerd Glasses"
    );
    assert_eq!(get_traits("Male 1, 3D Glasses, Cap"), "3D Glasses,Cap");
    assert_eq!(get_traits("Alien, Headband"), "Headband");
    assert_eq!(get_type("Alien, Headband"), "Alien");
    assert_eq!(get_type("Female 2, Mohawk"), "Female");
    assert_eq!(get_traits("Female 2"), "");
    assert_eq!(get_traits(""), "");
}

#[test]
fn transfer_to_bidder_closes_bid_but_transfer_to_someone_else_does_not() {
    let transfer = punks::Transfer {
        token_id: 42,
        to: address(2),
        ordinal: 100,
        log_index: 4,
        block_number: 14_000_000,
        ..Default::default()
    };
    let out = state::bid_updates(
        punks::Bids::default(),
        punks::Sales::default(),
        punks::Transfers {
            transfers: vec![transfer.clone()],
        },
        |_| Some(bid()),
    );
    assert_eq!(out.len(), 1);
    assert_eq!(
        (out[0].open.as_str(), out[0].amount.as_str(), out[0].ordinal),
        ("false", "1.25", 100)
    );
    let out = state::bid_updates(
        punks::Bids::default(),
        punks::Sales::default(),
        punks::Transfers {
            transfers: vec![punks::Transfer {
                to: address(9),
                ..transfer
            }],
        },
        |_| Some(bid()),
    );
    assert!(out.is_empty());
}

#[test]
fn closed_bid_cannot_be_reused_by_later_sales() {
    let raw = punks::Bid {
        block_number: 10,
        ordinal: 500,
        ..bid()
    };
    // Ordinals restart each block: compare block number before ordinal.
    let reset = punks::Bid {
        block_number: 11,
        ordinal: 20,
        ..raw.clone()
    };
    assert!(state::active_bid(Some(raw.clone()), Some(reset.clone())).is_none());
    let mut accepted = sale();
    accepted.bid_accepted = true;
    assert!(state::resolve_sales(
        punks::Sales {
            sales: vec![accepted]
        },
        |_| { state::active_bid(Some(raw.clone()), Some(reset.clone())) }
    )
    .is_err());
    let changes = state::bid_updates(
        punks::Bids::default(),
        punks::Sales {
            sales: vec![sale()],
        },
        punks::Transfers::default(),
        |_| state::active_bid(Some(raw.clone()), Some(reset.clone())),
    );
    assert!(
        changes.is_empty(),
        "a later sale must not emit another closure for a cleared bid"
    );
}

#[test]
fn new_bid_after_refund_reopens_same_bidder_and_punk() {
    let reset = punks::Bid {
        block_number: 11,
        ordinal: 20,
        ..bid()
    };
    let newer = punks::Bid {
        block_number: 11,
        ordinal: 30,
        amount: "3".into(),
        ..bid()
    };
    assert_eq!(
        state::active_bid(Some(newer), Some(reset)).unwrap().amount,
        "3"
    );
}

#[test]
fn reset_for_other_bidder_or_token_does_not_clear_bid() {
    let raw = bid();
    for reset in [
        punks::Bid {
            from: address(8),
            block_number: 100,
            ..raw.clone()
        },
        punks::Bid {
            token_id: 8,
            block_number: 100,
            ..raw.clone()
        },
    ] {
        assert!(state::active_bid(Some(raw.clone()), Some(reset)).is_some());
    }
}

#[test]
fn replacements_and_withdrawals_decode_in_order_with_exact_values() {
    let raw = events::map_bids(block(vec![
        log(
            "PunkBidEntered",
            vec![uint(42), uint(100), addr(2)],
            10,
            false,
        ),
        log(
            "PunkBidEntered",
            vec![uint(42), uint(200), addr(3)],
            20,
            false,
        ),
        log(
            "PunkBidWithdrawn",
            vec![uint(42), uint(200), addr(3)],
            30,
            false,
        ),
    ]))
    .unwrap();
    assert_eq!(
        raw.bids
            .iter()
            .map(|b| (b.from.clone(), b.open.as_str(), b.ordinal))
            .collect::<Vec<_>>(),
        vec![
            (address(2), "true", 10),
            (address(3), "true", 20),
            (address(3), "false", 30)
        ]
    );
    assert_eq!(raw.bids[1].amount, "0.0000000000000002");
    assert!(state::active_bid(Some(raw.bids[2].clone()), None).is_none());
}

#[test]
fn transfer_refund_and_later_bid_are_sorted_by_event_position() {
    let later = punks::Bid {
        ordinal: 200,
        amount: "4".into(),
        ..bid()
    };
    let out = state::bid_updates(
        punks::Bids { bids: vec![later] },
        punks::Sales::default(),
        punks::Transfers {
            transfers: vec![punks::Transfer {
                token_id: 42,
                to: address(2),
                ordinal: 100,
                ..Default::default()
            }],
        },
        |_| Some(bid()),
    );
    assert_eq!(
        out.iter()
            .map(|b| (b.ordinal, b.open.as_str()))
            .collect::<Vec<_>>(),
        vec![(100, "false"), (200, "true")]
    );
}

// Successful receipt subset from tests/fixtures/mainnet-bid-ordering-4009734.json.
// Real separate transactions: the same bidder raises Punk 4936 from .05 to .1 ETH.
#[test]
fn mainnet_same_block_bid_replacement_preserves_transaction_order() {
    let transactions = [
        (
            "7bdac29378fa07d60a3dfa149405fa0c9fe360b4d83ac77aff0d43b624b7bde2",
            4936u64,
            50_000_000_000_000_000u64,
            1076u64,
            34u32,
        ),
        (
            "fc8fa92c3297de6c38f19639bd19e91ee6d7324a696c333079777105086b8cf2",
            2891,
            100_000_000_000_000_000,
            1265,
            40,
        ),
        (
            "7e495f1f8bfc71ed15a4fd9061f03ddaf07bdf00d416498f4650ef88a3582dd5",
            4936,
            100_000_000_000_000_000,
            1302,
            41,
        ),
    ];
    let blk = Block {
        number: 4_009_734,
        hash: hex::decode("3744d5b23ea37b9398f533ed1dd71c585649e125e5510133772ba6e804d41480").unwrap(),
        header: Some(BlockHeader {
            timestamp: Some(prost_types::Timestamp { seconds: 1499817505, nanos: 0 }),
            ..Default::default()
        }),
        transaction_traces: transactions.into_iter().map(|(hash, token, wei, ordinal, block_index)| TransactionTrace {
            hash: hex::decode(hash).unwrap(),
            status: 1,
            receipt: Some(TransactionReceipt {
                logs: vec![Log {
                    address: CRYPTOPUNKS_CONTRACT.to_vec(),
                    topics: vec![
                        hex::decode("5b859394fabae0c1ba88baffe67e751ab5248d2e879028b8c8d6897b0519f56a").unwrap(),
                        hex::decode(format!("{token:064x}")).unwrap(),
                        hex::decode("00000000000000000000000022160e8a944e4f4f8aced15f54b265bfd12854e2").unwrap(),
                    ],
                    data: hex::decode(format!("{wei:064x}")).unwrap(),
                    ordinal, block_index, ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        }).collect(),
        ..Default::default()
    };
    let out = state::bid_updates(
        events::map_bids(blk).unwrap(),
        punks::Sales::default(),
        punks::Transfers::default(),
        |_| panic!("bid-only block must not need ownership reset lookups"),
    );
    assert_eq!(
        out.iter()
            .map(|b| (b.token_id, b.amount.as_str(), b.ordinal, b.log_index))
            .collect::<Vec<_>>(),
        vec![
            (4936, "0.05", 1076, 34),
            (2891, "0.1", 1265, 40),
            (4936, "0.1", 1302, 41)
        ]
    );
    assert_ne!(out[0].trx_hash, out[2].trx_hash);
    assert_eq!(out[0].from, out[2].from);
    assert_eq!(
        out[2].trx_hash,
        "0x7e495f1f8bfc71ed15a4fd9061f03ddaf07bdf00d416498f4650ef88a3582dd5"
    );
}
