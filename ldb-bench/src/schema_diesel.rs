//! Diesel `table!` 定义。

diesel::table! {
    t_user (id) {
        id -> BigInt,
        name -> Nullable<Varchar>,
        age -> Nullable<Integer>,
    }
}
