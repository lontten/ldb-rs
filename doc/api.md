# API 参考

> **状态**：已实现；公开接口与本文件保持一致。

用户只需依赖 `ldb` crate；下列类型与函数均由 `ldb` 根模块 re-export。

CRUD 采用 **入口函数 → Builder 链式配置 → `.await?`**；Builder 实现 `Future`，无需单独的 `*Options` 结构体。

```rust
use ldb::{
    connect_mysql, connect_pg, count, delete, first, get_or_insert, has, has_or_insert, insert,
    list, native_exec, native_query, prepare, query_build, update, update_by_primary_key, w,
    InsertResult, LdbError, LdbModel, MysqlConfig, MysqlVersion, OnConflict, Order,
    PgConfig, PoolConfig, TableConf, WhereBuilder,
};
```

---

## 1. 配置

### `MysqlVersion`

MySQL 服务端版本提示，影响 upsert 语法与自增主键回填策略。

```rust
pub enum MysqlVersion {
    Latest,
    V5,
    V8_0_19,
    V8_0_20,
}
```

### `MysqlConfig`

```rust
pub struct MysqlConfig {
    pub host: String,
    pub port: String,
    pub db_name: String,
    pub user: String,
    pub password: String,
    /// DSN 附加参数，如 `charset=utf8mb4`
    pub other: String,
    pub version: MysqlVersion,
}
```

### `PgConfig`

```rust
pub struct PgConfig {
    pub host: String,
    pub port: String,
    pub db_name: String,
    pub user: String,
    pub password: String,
    /// DSN 附加参数，如 `sslmode=disable TimeZone=Asia/Shanghai`
    pub other: String,
}
```

### `PoolConfig`

```rust
pub struct PoolConfig {
    /// 最小保留连接数；映射 sqlx min_connections
    pub max_idle_count: Option<u32>,
    /// 最大打开连接数；`None` 或 `0` 为不限制
    pub max_open: Option<u32>,
    pub max_lifetime: Option<std::time::Duration>,
    pub max_idle_time: Option<std::time::Duration>,
}
```

---

## 2. 连接

### `connect_mysql`

```rust
pub async fn connect_mysql(
    config: &MysqlConfig,
    pool: Option<&PoolConfig>,
) -> Result<MysqlEngine, LdbError>;
```

建立 MySQL 连接池并返回引擎句柄。

### `connect_pg`

```rust
pub async fn connect_pg(
    config: &PgConfig,
    pool: Option<&PoolConfig>,
) -> Result<PgEngine, LdbError>;
```

建立 PostgreSQL 连接池并返回引擎句柄。

### `MysqlEngine` / `PgEngine`

连接池引擎，对用户透明；CRUD 入口均接受 `&impl Engine`，无需直接操作具体类型。

---

## 3. `Engine` 与事务

### `Engine`

```rust
pub trait Engine: Send + Sync {
    fn ping(&self) -> impl Future<Output = Result<(), LdbError>> + Send;
    fn begin(&self) -> impl Future<Output = Result<Transaction, LdbError>> + Send;
}
```

| 方法 | 说明 |
|------|------|
| `ping` | 检测连接是否可用 |
| `begin` | 开启事务，返回 `Transaction` |

### `Transaction`

事务句柄；实现 `Engine`，可在事务内调用 CRUD。

```rust
pub struct Transaction { /* 内部持有 sqlx 事务 */ }

impl Transaction {
    pub fn commit(self) -> impl Future<Output = Result<(), LdbError>> + Send;
    pub fn rollback(self) -> impl Future<Output = Result<(), LdbError>> + Send;
}
```

`commit` / `rollback` 消费 `self`；未提交时 drop 将回滚。

---

## 4. 模型

### `#[derive(LdbModel)]`

过程宏，为结构体生成表元数据与列映射。

```rust
#[derive(LdbModel)]
#[ldb(table = "t_user", primary_key = "id", auto_column = "id", soft_delete = "deleted_at")]
pub struct User {
    #[db(column = "id")]
    pub id: Option<i64>,
    #[db(column = "name")]
    pub name: Option<String>,
    #[db(column = "age")]
    pub age: Option<i32>,
}
```

| Attribute | 说明 |
|-----------|------|
| `#[ldb(table = "...")]` | 表名 |
| `#[ldb(primary_key = "...")]` | 主键列，可重复指定多列 |
| `#[ldb(auto_column = "...")]` | 自增列，Insert 后回填到模型字段 |
| `#[ldb(soft_delete = "...")]` | 软删除列；查询自动过滤，删除改为写入当前时间 |
| `#[db(column = "...")]` | 字段对应列名 |

### `LdbModel`

宏生成的 trait，用户通常不手动实现。

```rust
pub trait LdbModel: Send + Sync + Sized {
    fn table_conf() -> &'static TableConf;
    fn column_meta_list() -> &'static [ColumnMeta];
}
```

### `TableConf`

```rust
pub struct TableConf {
    pub table_name: &'static str,
    pub primary_key_column_name_list: &'static [&'static str],
    pub auto_column: Option<&'static str>,
    pub soft_delete_column: Option<&'static str>,
}
```

### `ColumnMeta`

```rust
pub struct ColumnMeta {
    pub field_name: &'static str,
    pub column_name: &'static str,
}
```

### 字段规则

- 模型字段使用 `Option<T>` 表示可空列。
- **Insert / Update**：值为 `None` 的字段不参与 SQL（不写入、不更新）。
- **条件模型**（如 `UserWhere`）：仅 `Some` 字段参与 `w().model(&cond)`。

```rust
pub struct UserWhere {
    pub name: Option<String>,
    pub age: Option<i32>,
}
```

---

## 5. CRUD 入口

所有 CRUD **入口函数**返回 Builder；链式配置后以 `.await?` 执行。

```rust
// 典型用法
let result = insert(&db, &mut user)
    .on_conflict(OnConflict::UpdateKey { column_name_list: vec!["name".into()] })
    .show_sql(true)
    .await?;
```

### 入口一览

| 入口 | 返回 Builder | `.await?` 类型 |
|------|--------------|----------------|
| `insert(engine, model)` | `InsertBuilder` | `Result<InsertResult, LdbError>` |
| `update_by_primary_key(engine, model)` | `UpdateByPkBuilder` | `Result<u64, LdbError>` |
| `update(engine, patch)` | `UpdateBuilder` | `Result<u64, LdbError>` |
| `delete::<T>(engine)` | `DeleteBuilder<T>` | `Result<u64, LdbError>` |
| `first::<T>(engine)` | `FirstBuilder<T>` | `Result<Option<T>, LdbError>` |
| `list::<T>(engine)` | `ListBuilder<T>` | `Result<Vec<T>, LdbError>` |
| `has::<T>(engine)` | `HasBuilder<T>` | `Result<bool, LdbError>` |
| `count::<T>(engine)` | `CountBuilder<T>` | `Result<u64, LdbError>` |
| `get_or_insert(engine, candidate)` | `GetOrInsertBuilder<T>` | `Result<T, LdbError>` |
| `has_or_insert(engine, where_builder, candidate)` | async 函数 | `Result<bool, LdbError>` |

### 入口签名

```rust
pub fn insert<'a, E: Engine, M: LdbModel>(
    engine: &'a E,
    model: &'a mut M,
) -> InsertBuilder<'a, E, M>;

pub fn update_by_primary_key<'a, E: Engine, M: LdbModel>(
    engine: &'a E,
    model: &'a M,
) -> UpdateByPkBuilder<'a, E, M>;

pub fn update<'a, E: Engine, M: LdbModel>(
    engine: &'a E,
    patch: &'a M,
) -> UpdateBuilder<'a, E, M>;

pub fn delete<'a, T: LdbModel, E: Engine>(
    engine: &'a E,
) -> DeleteBuilder<'a, E, T>;

pub fn first<'a, T: LdbModel, E: Engine>(
    engine: &'a E,
) -> FirstBuilder<'a, E, T>;

pub fn list<'a, T: LdbModel, E: Engine>(
    engine: &'a E,
) -> ListBuilder<'a, E, T>;

pub fn has<'a, T: LdbModel, E: Engine>(
    engine: &'a E,
) -> HasBuilder<'a, E, T>;

pub fn count<'a, T: LdbModel, E: Engine>(
    engine: &'a E,
) -> CountBuilder<'a, E, T>;

pub fn get_or_insert<'a, T: LdbModel, E: Engine>(
    engine: &'a E,
    candidate: &'a mut T,
) -> GetOrInsertBuilder<'a, E, T>;
```

`insert` / `get_or_insert` 的 `model` / `candidate` 为 `mut`：自增主键成功后写回对应 `Option` 字段。

### `InsertResult`

```rust
pub struct InsertResult {
    pub rows_affected: u64,
}
```

---

## 6. `WhereBuilder`

### `w`

```rust
pub fn w() -> WhereBuilder;
```

创建空的条件构建器；通过 CRUD Builder 的 `where_` 传入（`where` 为 Rust 关键字，故用下划线）。

### 方法一览

所有方法为 **consuming builder**（接收 `self`，返回 `Self`），支持链式调用。

#### 模型与主键

| 方法 | 签名 | 说明 |
|------|------|------|
| `model` | `fn model(self, cond: &impl LdbModel) -> Self` | 非 `None` 字段生成 AND 条件 |
| `primary_key` | `fn primary_key<M: LdbModel>(self, id: impl IntoSqlValue) -> Self` | 从 `TableConf` 读取单主键并生成等值条件 |
| `primary_key_model` | `fn primary_key_model<M: LdbModel>(self, model: &M) -> Self` | 从模型读取单列或多列主键并生成等值组合 |
| `filter_primary_key` | `fn filter_primary_key<M: LdbModel>(self) -> Self` | 从 `TableConf` 生成全部主键非空过滤 |

#### 比较

| 方法 | 签名 | 说明 |
|------|------|------|
| `eq` | `fn eq(self, column: &str, value: impl IntoSqlValue) -> Self` | 等于 |
| `eq_if` | `fn eq_if(self, column: &str, value: impl IntoSqlValue, enabled: bool) -> Self` | `enabled` 为 false 时跳过 |
| `not_eq` | `fn not_eq(self, column: &str, value: impl IntoSqlValue) -> Self` | 不等于 |
| `in_list` | `fn in_list(self, column: &str, values: impl IntoIterator<Item = impl IntoSqlValue>) -> Self` | IN |
| `not_in_list` | `fn not_in_list(self, column: &str, values: impl IntoIterator<Item = impl IntoSqlValue>) -> Self` | NOT IN |
| `gt` | `fn gt(self, column: &str, value: impl IntoSqlValue) -> Self` | 大于 |
| `gte` | `fn gte(self, column: &str, value: impl IntoSqlValue) -> Self` | 大于等于 |
| `lt` | `fn lt(self, column: &str, value: impl IntoSqlValue) -> Self` | 小于 |
| `lte` | `fn lte(self, column: &str, value: impl IntoSqlValue) -> Self` | 小于等于 |
| `between` | `fn between(self, column: &str, low: impl IntoSqlValue, high: impl IntoSqlValue) -> Self` | BETWEEN |

#### 空值

| 方法 | 签名 | 说明 |
|------|------|------|
| `is_null` | `fn is_null(self, column: &str) -> Self` | IS NULL |
| `is_not_null` | `fn is_not_null(self, column: &str) -> Self` | IS NOT NULL |

#### 模糊匹配

| 方法 | 签名 | 说明 |
|------|------|------|
| `like` | `fn like(self, column: &str, pattern: &str) -> Self` | LIKE |
| `like_left` | `fn like_left(self, column: &str, pattern: &str) -> Self` | 前缀匹配（`%pattern`） |
| `like_right` | `fn like_right(self, column: &str, pattern: &str) -> Self` | 后缀匹配（`pattern%`） |
| `not_like` | `fn not_like(self, column: &str, pattern: &str) -> Self` | NOT LIKE |

#### 逻辑组合

| 方法 | 签名 | 说明 |
|------|------|------|
| `and` | `fn and(self, other: WhereBuilder) -> Self` | AND |
| `or` | `fn or(self, other: WhereBuilder) -> Self` | OR |
| `not` | `fn not(self) -> Self` | 对当前子树取反 |

#### 原生 SQL 片段

| 方法 | 签名 | 说明 |
|------|------|------|
| `native` | `fn native(self, sql: &str, arg_list: impl IntoIterator<Item = impl IntoSqlValue>) -> Self` | 追加原生条件片段 |
| `native_if` | `fn native_if(self, enabled: bool, sql: &str, arg_list: impl IntoIterator<Item = impl IntoSqlValue>) -> Self` | 条件为 true 时追加 |

### `IntoSqlValue`

内部 trait，将 Rust 值绑定为 SQL 参数；对用户透明，`i64`、`String`、`&str`、`Option<T>` 等常见类型自动实现。

---

## 7. CRUD Builder

各 Builder 为 consuming 链式；链末实现 `Future`，直接 `.await?`。

Builder 类型由入口函数返回，**不单独导出**；用户通过入口函数与链式方法使用。

### 共用链式方法

以下方法在适用 Builder 上提供：

| 方法 | 签名 | 说明 |
|------|------|------|
| `table_name` | `fn table_name(self, name: impl Into<String>) -> Self` | 临时覆盖表名 |
| `show_sql` | `fn show_sql(self, enabled: bool) -> Self` | 执行前打印 SQL 与参数 |
| `dry_run` | `fn dry_run(self, enabled: bool) -> Self` | 只生成 SQL，不执行 |

### `InsertBuilder`

| 方法 | 说明 |
|------|------|
| `on_conflict(OnConflict)` | 唯一键冲突时的行为 |
| 共用方法 | `table_name`、`show_sql`、`dry_run` |

### `GetOrInsertBuilder`

| 方法 | 说明 |
|------|------|
| `where_(WhereBuilder)` | **必填**；用于查询是否已存在 |
| `on_conflict(OnConflict)` | 并发插入冲突时的行为 |
| 共用方法 | `table_name`、`show_sql`、`dry_run` |

### `UpdateByPkBuilder`

| 方法 | 说明 |
|------|------|
| `set_null` / `set` / `set_increment` / `set_expression` / `set_now` | 额外 SET 子句，与 `patch` 字段合并 |
| 共用方法 | `table_name`、`show_sql`、`dry_run` |

无需 `where_`；主键从 `model` 的 `Some` 字段读取。

### `UpdateBuilder`

| 方法 | 说明 |
|------|------|
| `where_(WhereBuilder)` | **必填** |
| `allow_full_table(bool)` | 无 WHERE 时是否允许全表更新（默认 false） |
| `set_null` / `set` / `set_increment` / `set_expression` / `set_now` | 额外 SET 子句 |
| 共用方法 | `table_name`、`show_sql`、`dry_run` |

### `DeleteBuilder`

| 方法 | 说明 |
|------|------|
| `where_(WhereBuilder)` | **必填**（全表删除时传空 `w()` 并 `allow_full_table(true)`） |
| `allow_full_table(bool)` | 无 WHERE 时是否允许全表删除（默认 false） |
| 共用方法 | `table_name`、`show_sql`、`dry_run` |

### `FirstBuilder` / `ListBuilder` / `HasBuilder` / `CountBuilder`

| 方法 | 说明 |
|------|------|
| `where_(WhereBuilder)` | **必填** |
| `order_by(column, Order)` | 排序（`HasBuilder` / `CountBuilder` 忽略） |
| `limit(u64)` | 分页上限（`FirstBuilder` 内部强制为 1） |
| `offset(u64)` | 分页偏移（`FirstBuilder` / `HasBuilder` 忽略） |
| 共用方法 | `table_name`、`show_sql`、`dry_run` |

### `OnConflict`

```rust
pub enum OnConflict {
    /// 冲突时不做写入
    DoNothing,
    /// 按指定唯一键列执行 upsert 更新
    UpdateKey { column_name_list: Vec<String> },
    /// 冲突时更新所有非主键列
    UpdateAll,
}
```

### `Order`

```rust
pub enum Order {
    Asc,
    Desc,
}
```

### Builder 能力矩阵

| 能力 | Insert | GetOrInsert | UpdateByPk | Update | Delete | First/List | Has/Count |
|------|--------|-------------|------------|--------|--------|------------|-----------|
| `where_` | — | ✓ | — | ✓ | ✓ | ✓ | ✓ |
| `on_conflict` | ✓ | ✓ | — | — | — | — | — |
| `allow_full_table` | — | — | — | ✓ | ✓ | — | — |
| `set_*` | — | — | ✓ | ✓ | — | — | — |
| `order_by` / `limit` / `offset` | — | — | — | — | — | ✓ | 部分 |
| `table_name` / `show_sql` / `dry_run` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

支持软删除的查询、更新与删除 Builder 另提供 `skip_soft_delete(bool)`。

---

## 8. `LdbError`

```rust
pub enum LdbError {
    /// 连接不可用或已关闭
    NotConnected,
    /// 需要 WHERE 条件但未提供
    WhereRequired,
    /// 无 WHERE 的全表操作且未显式允许
    FullTableOpNotAllowed,
    /// 模型字段与列映射错误
    ModelMapping(String),
    /// SQL 生成或方言改写失败
    SqlBuild(String),
    /// sqlx 执行错误
    Sqlx(#[from] sqlx::Error),
    /// 功能尚未实现（开发阶段）
    NotImplemented,
}
```

---

## 9. 导出一览

`ldb` crate 根模块公开导出：

| 分类 | 符号 |
|------|------|
| 连接 | `connect_mysql`, `connect_pg`, `MysqlEngine`, `PgEngine`, `MysqlConfig`, `PgConfig`, `PoolConfig`, `MysqlVersion` |
| 引擎 | `Engine`, `Transaction` |
| 模型 | `LdbModel`, `TableConf`, `ColumnMeta` |
| CRUD 入口 | `insert`, `update`, `update_by_primary_key`, `delete`, `first`, `list`, `has`, `count`, `get_or_insert`, `has_or_insert`, `InsertResult` |
| 扩展查询 | `query_build`, `native_query`, `native_exec`, `prepare`, `StmtQuery` |
| 条件 | `w`, `WhereBuilder` |
| 枚举 | `OnConflict`, `Order` |
| 错误 | `LdbError` |

宏：`#[derive(LdbModel)]`（由 `ldb-macros` 提供，经 `ldb` 重导出）。

CRUD Builder（`InsertBuilder`、`ListBuilder` 等）由入口函数返回，不单独导出。

---

## 10. 扩展查询 API

```rust
pub fn query_build<'a, T: LdbModel, E: Engine>(engine: &'a E) -> QueryBuild<'a, E, T>;

pub async fn native_query<T: LdbModel + Default, E: Engine>(
    engine: &E,
    sql: impl Into<String>,
    arg_list: impl IntoIterator<Item = impl IntoSqlValue>,
) -> Result<Vec<T>, LdbError>;

pub async fn native_exec<E: Engine>(
    engine: &E,
    sql: impl Into<String>,
    arg_list: impl IntoIterator<Item = impl IntoSqlValue>,
) -> Result<u64, LdbError>;

pub async fn prepare<'a, E: Engine>(
    engine: &'a E,
    sql: impl Into<String>,
) -> Result<PreparedStatement<'a, E>, LdbError>;
```

`QueryBuild` 支持 `select`、`from`、`inner_join`、`where_`、`order_by`、
`limit`、`offset`，并以 `list`、`first`、`count`、`page` 执行。
`PreparedStatement::query::<T>(args)` 返回可直接 `.await?` 的 `StmtQuery`。
