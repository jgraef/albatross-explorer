table! {
    address_aliases (id) {
        id -> Int4,
        address -> Bytea,
        alias -> Text,
    }
}

table! {
    transactions (id) {
        id -> Int4,
        txid -> Bytea,
        block_hash -> Bytea,
        tx_idx -> Int4,
        sender -> Bytea,
        recipient -> Bytea,
    }
}

allow_tables_to_appear_in_same_query!(
    address_aliases,
    transactions,
);
