//! Diesel `table!` 定义。

diesel::table! {
    t_user (id) {
        id -> BigInt,
        name -> Varchar,
        age -> Integer,
        status -> SmallInt,
        city -> Nullable<Varchar>,
        created_at -> Timestamp,
    }
}
