# ldb

基于 **sqlx** 的轻量 async ORM，支持 **MySQL** 与 **PostgreSQL**。

- **GitHub 仓库**：`ldb-rs`
- **crates.io 发布名**：`ldb`（`cargo add ldb`）
- **当前状态**：项目骨架（workspace 空壳），**尚无 CRUD 等业务实现**

## 技术栈

| 方面 | 选择 |
|------|------|
| 底层 | **sqlx** + **tokio**（async 优先） |
| 模型映射 | `#[derive(LdbModel)]`（后续） |
| SQL | 运行时动态拼装，`sqlx::query()`，不用 `query!()` |
| 方言 | MySQL `?` / PostgreSQL `$1`，`Dialect` trait |

## Workspace 架构

本仓库是 **Cargo workspace**，包含三个 crate：

| Crate | 职责 |
|-------|------|
| **`ldb`** | 用户门面；统一 re-export |
| **`ldb-core`** | `Engine`、`Dialect`、`WhereBuilder`、`ExtraContext`、配置与错误 |
| **`ldb-macros`** | `#[derive(LdbModel)]` 过程宏 |

用户只需依赖 `ldb`：

```toml
[dependencies]
ldb = "0.1"
```

```rust
use ldb::{MysqlConfig, WhereBuilder, w, LdbModel};
```

## 构建

```bash
cargo build
cargo test --all
cargo doc --no-deps
```

## 文档

- [doc/](doc/)：用法、示例与设计说明（逐步补充）

## License

Apache-2.0
