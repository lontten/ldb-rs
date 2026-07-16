//! `#[derive(LdbModel)]` 展开与元数据测试。

use ldb::{LdbModel, w};

#[derive(LdbModel)]
#[ldb(table = "t_user", primary_key = "id", auto_column = "id")]
struct DeriveUser {
    #[db(column = "id")]
    id: Option<i64>,
    name: Option<String>,
    age: Option<i32>,
}

#[test]
fn derive_column_name_const_default() {
    assert_eq!(DeriveUser::name, "name");
    assert_eq!(DeriveUser::age, "age");
}

#[test]
fn derive_column_name_const_db_column() {
    assert_eq!(DeriveUser::id, "id");
}

#[test]
fn derive_column_name_const_in_where() {
    let (sql, args) = w().eq(DeriveUser::name, "x").to_sql().unwrap();
    assert_eq!(sql, "name = ?");
    assert!(matches!(args[0], ldb::SqlValue::String(ref s) if s == "x"));
}

#[test]
fn derive_column_name_const_mapped_column() {
    #[derive(LdbModel)]
    #[ldb(table = "t_mapped")]
    struct MappedUser {
        #[db(column = "user_id")]
        id: Option<i64>,
    }
    assert_eq!(MappedUser::id, "user_id");
}

#[test]
fn derive_table_conf() {
    let conf = DeriveUser::table_conf();
    assert_eq!(conf.table_name, "t_user");
    assert_eq!(conf.primary_key_column_name_list, &["id"]);
    assert_eq!(conf.auto_column, Some("id"));
}

#[test]
fn derive_column_meta() {
    let columns = DeriveUser::column_meta_list();
    assert_eq!(columns.len(), 3);
    assert_eq!(columns[0].column_name, "id");
    assert_eq!(columns[1].field_name, "name");
}

#[test]
fn derive_field_sql_value() {
    let user = DeriveUser {
        id: Some(1),
        name: Some("tom".into()),
        age: None,
    };
    assert!(matches!(
        user.field_sql_value("name"),
        Some(ldb::SqlValue::String(s)) if s == "tom"
    ));
    assert!(user.field_sql_value("age").is_none());
}

#[test]
fn derive_option_i32_field() {
    #[derive(LdbModel)]
    #[ldb(table = "t_score")]
    struct Score {
        value: Option<i32>,
    }
    let row = Score { value: Some(9) };
    assert!(matches!(
        row.field_sql_value("value"),
        Some(ldb::SqlValue::I64(9))
    ));
}

#[test]
fn derive_soft_delete_and_extended_types() {
    #[derive(LdbModel)]
    #[ldb(table = "t_event", soft_delete = "deleted_at")]
    struct Event {
        deleted_at: Option<chrono::DateTime<chrono::Utc>>,
        token: Option<uuid::Uuid>,
        payload: Option<Vec<u8>>,
        sequence: Option<u64>,
    }
    assert_eq!(Event::table_conf().soft_delete_column, Some("deleted_at"));
    let event = Event {
        deleted_at: None,
        token: Some(uuid::Uuid::nil()),
        payload: Some(vec![1, 2]),
        sequence: Some(u64::MAX),
    };
    assert!(matches!(
        event.field_sql_value("token"),
        Some(ldb::SqlValue::Uuid(_))
    ));
    assert!(matches!(
        event.field_sql_value("payload"),
        Some(ldb::SqlValue::Bytes(_))
    ));
    assert_eq!(
        event.field_sql_value("sequence"),
        Some(ldb::SqlValue::U64(u64::MAX))
    );
}
