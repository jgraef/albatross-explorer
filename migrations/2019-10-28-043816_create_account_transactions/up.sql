CREATE TABLE transactions
(
    id         SERIAL  PRIMARY KEY,
    txid       BYTEA   NOT NULL,
    block_hash BYTEA   NOT NULL,
    tx_idx     INTEGER NOT NULL,
    sender     BYTEA   NOT NULL,
    recipient  BYTEA   NOT NULL
);

CREATE INDEX index_by_txid ON transactions (txid);
CREATE INDEX index_by_sender ON transactions (sender);
CREATE INDEX index_by_recipient ON transactions (recipient);


CREATE TABLE address_aliases
(
    id      SERIAL  PRIMARY KEY,
    address BYTEA NOT NULL,
    alias   TEXT  NOT NULL
);

CREATE INDEX index_by_address ON address_aliases (address);
