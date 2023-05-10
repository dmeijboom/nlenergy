CREATE TABLE usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    rate TINYINT NOT NULL,
    delivered BIGINT NOT NULL,
    received BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL
);