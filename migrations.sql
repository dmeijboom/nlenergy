CREATE TABLE history (
    id SERIAL PRIMARY KEY,
    checksum VARCHAR,
    rate TINYINT,
    energy BIGINT,
    time TIMESTAMP
);
CREATE UNIQUE INDEX history_checksum_idx ON history(checksum);
