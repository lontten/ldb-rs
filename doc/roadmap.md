# 实现路线图

> API 契约见 [api.md](api.md)；用法示例见 [guide.md](guide.md)。

## 阶段总览

| 阶段 | 内容 | 对照文档 |
|------|------|----------|
| Commit 1 ✅ | workspace 骨架、模块 stub | — |
| Commit 2 ✅ | 连接 API + `MysqlEngine` / `PgEngine` | api §1–3, guide §2 |
| Commit 3 ✅ | `LdbModel` 宏 + `TableConf` / `ColumnMeta` | api §4, guide §3 |
| Commit 4 ✅ | `WhereBuilder` 全方法 + `to_where_sql` | api §6, guide §8 |
| Commit 5 ✅ | CRUD Builder 类型；删除 `ExtraContext` / `e()` | api §5、§7 |
| Commit 6 ✅ | Insert / Update / Delete 入口与 `Future` 实现 | api §5, guide §4–6 |
| Commit 7 ✅ | Select 族 + 事务 | api §5, guide §7、10 |
| Commit 8 ✅ | `LdbError` 补全 + 集成测试 | api §8 |

## Commit 2：连接

- 实现 `connect_mysql` / `connect_pg`
- `MysqlConfig` / `PgConfig` 构建 DSN
- `PoolConfig` 映射到 `sqlx` 连接池选项
- `MysqlEngine` / `PgEngine` 实现 `Engine::ping`、`Engine::begin`

## Commit 3：模型宏

- `ldb-macros` 解析 `#[ldb(...)]` 与 `#[db(column = ...)]`
- 生成 `LdbModel`、`TableConf`、`ColumnMeta` 静态元数据
- 支持条件模型在 `w().model()` 中的字段过滤

## Commit 4：WhereBuilder

- 实现 api.md 中全部 `WhereBuilder` 方法
- `IntoSqlValue` trait 与参数绑定
- `Engine::to_where_sql` 或内部等价逻辑

## Commit 5：CRUD Builder

- 实现 `InsertBuilder`、`UpdateBuilder`、`DeleteBuilder`、`ListBuilder` 等
- 各 Builder 链式方法与 `impl Future`
- 实现 `OnConflict`、`Order` 枚举
- 移除 `extra.rs` 中的 `ExtraContext` / `e()` 公开导出
- 同步更新 `.cursor/rules/project-conventions.mdc` 术语

## Commit 6：写操作

- `insert`、`update_by_primary_key`、`update`、`delete` 入口函数
- 自增主键回填、`dry_run` / `show_sql`
- MySQL / PG upsert（`OnConflict`）

## Commit 7：读操作与事务

- `first`、`list`、`has`、`count`、`get_or_insert` 入口与 Builder
- Builder 排序与分页（`order_by` / `limit` / `offset`）
- `Transaction` 持有 `sqlx::Transaction`；`commit` / `rollback`

## Commit 8：错误与测试

- `LdbError` 变体与 `thiserror` 消息
- 针对 MySQL / PG 的集成测试（需本地或 CI 数据库）

## 扩展能力

- QueryBuild ✅
- NativeQuery / NativeExec ✅
- Prepare / StmtQuery ✅
- 软删除 ✅
- Hook（后续）
