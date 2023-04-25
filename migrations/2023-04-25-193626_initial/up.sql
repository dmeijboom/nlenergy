CREATE TABLE history (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    checksum VARCHAR NOT NULL,
    rate TINYINT NOT NULL,
    energy BIGINT NOT NULL,
    time TIMESTAMP NOT NULL
);

CREATE UNIQUE INDEX history_checksum_idx ON history(checksum);
