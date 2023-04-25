// @generated automatically by Diesel CLI.

diesel::table! {
    history (id) {
        id -> Integer,
        checksum -> Text,
        rate -> Bool,
        energy -> BigInt,
        time -> Timestamp,
    }
}
