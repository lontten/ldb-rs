# ldb

基于 **sqlx** 的轻量 async ORM，支持 **MySQL** 与 **PostgreSQL**。

- **GitHub 仓库**：`ldb-rs`
- **crates.io 发布名**：`ldb`（`cargo add ldb`）
- **当前状态**：workspace 骨架；**目标 API 已在 doc 定稿，实现进行中**

## 技术栈

| 方面 | 选择 |
|------|------|
| 底层 | **sqlx** + **tokio**（async 优先） |
| 模型映射 | `#[derive(LdbModel)]` |
| SQL | 运行时动态拼装，`sqlx::query()`，不用 `query!()` |
| 方言 | MySQL `?` / PostgreSQL `$1`，`Dialect` trait |
| CRUD API | 入口函数 + Builder 链式 + `.await?` |

## Workspace 架构

本仓库是 **Cargo workspace**，包含三个 crate：

| Crate | 职责 |
|-------|------|
| **`ldb`** | 用户门面；统一 re-export |
| **`ldb-core`** | `Engine`、`Dialect`、`WhereBuilder`、CRUD Builder、配置与错误 |
| **`ldb-macros`** | `#[derive(LdbModel)]` 过程宏 |

用户只需依赖 `ldb`：

```toml
[dependencies]
ldb = "0.1"
```

```rust
use ldb::{MysqlConfig, WhereBuilder, w, LdbModel, insert};

// insert(&db, &mut user).on_conflict(...).await?
```

## 构建

```bash
cargo build
cargo test --all
cargo doc --no-deps
```

## 文档

| 文档 | 说明 |
|------|------|
| [doc/api.md](doc/api.md) | **API 参考**（目标契约，尚未实现） |
| [doc/guide.md](doc/guide.md) | **用法指南**（场景与示例） |
| [doc/architecture.md](doc/architecture.md) | 架构与设计约束 |
| [doc/roadmap.md](doc/roadmap.md) | 实现路线图 |

## License

Apache-2.0
