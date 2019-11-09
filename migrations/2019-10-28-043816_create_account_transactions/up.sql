CREATE TABLE transactions
(
    id           SERIAL PRIMARY KEY,
    txid         CHAR(64) NOT NULL,
    block_hash   CHAR(64) NOT NULL,
    block_number INTEGER  NOT NULL,
    tx_idx       INTEGER  NOT NULL,
    sender       CHAR(44) NOT NULL,
    recipient    CHAR(44) NOT NULL
);

CREATE INDEX index_by_txid ON transactions (txid);
CREATE INDEX index_by_sender ON transactions (sender);
CREATE INDEX index_by_recipient ON transactions (recipient);


CREATE TABLE account_aliases
(
    id      SERIAL PRIMARY KEY,
    address CHAR(44) NOT NULL,
    alias   TEXT     NOT NULL
);

CREATE INDEX index_by_address ON account_aliases (address);
