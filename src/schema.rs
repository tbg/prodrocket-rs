//! # Schema
//!
//! Define the tables to use with Diesel. Note that we can't use the
//! `infer_schema!` macro since that uses `information_schema` tables
//! unsupported by CockroachDB at the time of writing. `infer_schema!` is
//! somewhat scary, so perhaps this is preferable anyway.

table! {
    posts {
        id -> BigInt,
        title -> VarChar,
        body -> Text,
        published -> Bool,
    }
}
