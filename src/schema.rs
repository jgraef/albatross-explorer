table! {
    account_aliases (id) {
        id -> Int4,
        address -> Bpchar,
        alias -> Text,
    }
}

table! {
    transactions (id) {
        id -> Int4,
        txid -> Bpchar,
        block_hash -> Bpchar,
        block_number -> Int4,
        tx_idx -> Int4,
        sender -> Bpchar,
        recipient -> Bpchar,
    }
}

allow_tables_to_appear_in_same_query!(
    account_aliases,
    transactions,
);
