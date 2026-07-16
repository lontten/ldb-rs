# 用法指南

> 示例代码对应 [api.md](api.md) 中的已实现 API。

## 1. 安装与依赖

```toml
[dependencies]
ldb = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
use ldb::{
    connect_mysql, count, delete, first, get_or_insert, has, insert, list, native_query,
    prepare, query_build, update, update_by_primary_key, w, LdbModel,
    MysqlConfig, MysqlVersion, OnConflict, Order, PoolConfig,
};
```

---

## 2. 连接数据库

### MySQL

```rust
let config = MysqlConfig {
    host: "127.0.0.1".into(),
    port: "3306".into(),
    db_name: "test".into(),
    user: "root".into(),
    password: "123456".into(),
    other: "charset=utf8mb4".into(),
    version: MysqlVersion::Latest,
};

let db = connect_mysql(&config, None).await?;
db.ping().await?;
```

### PostgreSQL

```rust
use ldb::{connect_pg, PgConfig};

let config = PgConfig {
    host: "127.0.0.1".into(),
    port: "5432".into(),
    db_name: "test".into(),
    user: "postgres".into(),
    password: "123456".into(),
    other: "sslmode=disable TimeZone=Asia/Shanghai".into(),
};

let db = connect_pg(&config, None).await?;
```

### 连接池

```rust
use std::time::Duration;

let pool = PoolConfig {
    max_idle_count: Some(10),
    max_open: Some(100),
    max_lifetime: Some(Duration::from_secs(3600)),
    max_idle_time: None,
};

let db = connect_mysql(&config, Some(&pool)).await?;
```

---

## 3. 定义模型

```rust
#[derive(Clone, Default, LdbModel)]
#[ldb(table = "t_user", primary_key = "id", auto_column = "id", soft_delete = "deleted_at")]
struct User {
    #[db(column = "id")]
    id: Option<i64>,
    #[db(column = "name")]
    name: Option<String>,
    #[db(column = "age")]
    age: Option<i32>,
}

/// 条件模型：仅 Some 字段参与 w().model(&cond)
#[derive(Default, LdbModel)]
#[ldb(table = "t_user")]
struct UserWhere {
    name: Option<String>,
    age: Option<i32>,
}
```

---

## 4. 插入

### 普通插入

```rust
let mut user = User {
    id: None,
    name: Some("tom".into()),
    age: Some(18),
};

let result = insert(&db, &mut user).await?;

// result.rows_affected == 1
// user.id 已回填（若配置了 auto_column）
```

### 冲突时 upsert

```rust
let mut user = User {
    id: None,
    name: Some("tom".into()),
    age: Some(20),
};

insert(&db, &mut user)
    .on_conflict(OnConflict::UpdateKey {
        column_name_list: vec!["name".into()],
    })
    .await?;
```

### 调试：打印 SQL / 只生成不执行

```rust
insert(&db, &mut user)
    .show_sql(true)
    .dry_run(true)
    .await?;
```

---

## 5. 更新

### 按主键更新

```rust
let user = User {
    id: Some(1),
    name: Some("tom-updated".into()),
    age: None,
};

let rows = update_by_primary_key(&db, &user)
    .show_sql(true)
    .await?;
```

### 按条件更新

```rust
let patch = User {
    id: None,
    name: Some("patched".into()),
    age: None,
};

let rows = update(&db, &patch)
    .where_(
        w()
            .eq("id", 1)
            .in_list("id", [1i64, 2, 3])
            .gt("id", 0)
            .is_null("age"),
    )
    .set_null("age")
    .set_increment("login_count", 1)
    .set_expression("name", "UPPER(name)")
    .set_now("updated_at")
    .show_sql(true)
    .await?;
```

---

## 6. 删除

### 按主键

```rust
let rows = delete::<User>(&db)
    .where_(w().primary_key::<User>(1))
    .show_sql(true)
    .await?;
```

### 按模型条件

```rust
let cond = UserWhere {
    name: Some("tom".into()),
    age: None,
};

let rows = delete::<User>(&db)
    .where_(w().model(&cond))
    .await?;
```

### 按字段条件

```rust
let rows = delete::<User>(&db)
    .where_(w().eq("name", "orphan"))
    .await?;
```

### 全表删除（需显式允许）

```rust
// 默认会返回 FullTableOpNotAllowed
let rows = delete::<User>(&db)
    .where_(w())
    .allow_full_table(true)
    .await?;
```

---

## 7. 查询

### 单条

```rust
let user = first::<User>(&db)
    .where_(w().primary_key::<User>(1))
    .show_sql(true)
    .await?;

// user: Option<User>，无记录时为 None
```

### 列表

```rust
let cond = UserWhere {
    name: Some("tom".into()),
    age: None,
};

let user_list = list::<User>(&db)
    .where_(w().model(&cond))
    .order_by("id", Order::Desc)
    .limit(10)
    .await?;
```

### 是否存在

```rust
let exists = has::<User>(&db)
    .where_(w().eq("name", "tom"))
    .await?;
```

### 计数

```rust
let n = count::<User>(&db)
    .where_(w().gt("id", 0))
    .await?;
```

### 不存在则插入

```rust
let mut candidate = User {
    id: None,
    name: Some("new-user".into()),
    age: Some(18),
};

let user = get_or_insert(&db, &mut candidate)
    .where_(w().eq("name", "new-user"))
    .await?;
```

---

## 8. 条件构造

```rust
let wb = w()
    .eq_if("age", 18, true)
    .not_eq("name", "blocked")
    .in_list("id", [1i64, 2, 3])
    .like("name", "%tom%")
    .or(w().is_null("age"));

let users = list::<User>(&db)
    .where_(wb)
    .order_by("id", Order::Asc)
    .limit(10)
    .offset(0)
    .await?;
```

### 原生 SQL 片段

```rust
let wb = w()
    .eq("status", 1)
    .native("created_at > ?", ["2024-01-01"]);

let users = list::<User>(&db)
    .where_(wb)
    .await?;
```

---

## 9. 临时换表

各 CRUD Builder 均支持 `table_name`，覆盖模型 `#[ldb(table = "...")]` 中的表名：

```rust
insert(&db, &mut user)
    .table_name("t_user_backup")
    .await?;

list::<User>(&db)
    .where_(w().eq("name", "tom"))
    .table_name("t_user_archive")
    .await?;
```

---

## 10. 事务

```rust
let mut tx = db.begin().await?;

let mut user = User {
    id: None,
    name: Some("tx-user".into()),
    age: Some(25),
};

insert(&tx, &mut user).await?;

let patch = User {
    id: user.id,
    name: Some("tx-user-updated".into()),
    age: None,
};
update_by_primary_key(&tx, &patch).await?;

tx.commit().await?;
// 出错时可 tx.rollback().await?
```

事务内 CRUD 与连接池用法相同：第一个参数为 `&impl Engine`（此处为 `&Transaction`），链式调用方式不变。

---

## 11. 扩展查询

```rust
let user_list = query_build::<User, _>(&db)
    .where_(w().gt("id", 0))
    .order_by("id", Order::Desc)
    .page(1, 20)
    .await?;

let user_list = native_query::<User, _>(
    &db,
    "SELECT id, name, age, deleted_at FROM t_user WHERE age > ?",
    [18],
).await?;

let statement = prepare(&db, "SELECT id, name, age, deleted_at FROM t_user WHERE id = ?").await?;
let user_list = statement.query::<User>([1]).await?;
```

软删除模型的查询、更新与删除会自动应用 `deleted_at IS NULL`；需要访问全部
记录或执行物理删除时使用 `.skip_soft_delete(true)`。
