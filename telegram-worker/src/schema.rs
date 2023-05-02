// @generated automatically by Diesel CLI.

diesel::table! {
    usage (id) {
        id -> Integer,
        rate -> Bool,
        delivered -> BigInt,
        received -> BigInt,
        created_at -> Timestamp,
    }
}
