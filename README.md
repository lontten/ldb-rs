# ldb

基于 **sqlx** 的轻量 async ORM，支持 **MySQL** 与 **PostgreSQL**。

- **GitHub 仓库**：`ldb-rs`
- **crates.io 发布名**：`ldb`（`cargo add ldb`）
- **当前状态**：核心 CRUD、软删除、QueryBuild 与原生 SQL API 已实现

## 质量

[![CI](https://github.com/lontten/ldb-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/lontten/ldb-rs/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/lontten/ldb-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/lontten/ldb-rs)

| 指标 | 说明 |
|------|------|
| 测试 | CI 运行 `cargo test --all` 与 integration 测试 |
| 覆盖率 | CI 生成覆盖率报告并上传 Codecov（`connect` / `engine` / `exec` 除外） |
| 性能基准 | [CRUD 对比总览](https://lontten.github.io/ldb-rs/)（main 推送更新；[Criterion 详情](https://lontten.github.io/ldb-rs/report/index.html)） |

## 技术栈

| 方面 | 选择 |
|------|------|
| 底层 | **sqlx** + **tokio**（async 优先） |
| 模型映射 | `#[derive(LdbModel)]` |
| SQL | 运行时动态拼装，`sqlx::query()`，不用 `query!()` |
| 方言 | MySQL `?` / PostgreSQL `$1`，`Dialect` trait |
| CRUD API | 入口函数 + Builder 链式 + `.await?` |

## Workspace 架构

本仓库是 **Cargo workspace**，包含四个 crate：

| Crate | 职责 |
|-------|------|
| **`ldb`** | 用户门面；统一 re-export |
| **`ldb-core`** | `Engine`、`Dialect`、`WhereBuilder`、CRUD Builder、配置与错误 |
| **`ldb-macros`** | `#[derive(LdbModel)]` 过程宏 |
| **`ldb-bench`** | ldb 与常用 Rust ORM 的性能基准 |

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

完整基准含 Diesel 时：`cargo bench -p ldb-bench --bench crud_compare --features diesel`（需系统 libmysqlclient；Linux 可装 `libmysqlclient-dev`）。

## 文档

| 文档 | 说明 |
|------|------|
| [doc/api.md](doc/api.md) | **API 参考** |
| [doc/guide.md](doc/guide.md) | **用法指南**（场景与示例） |
| [doc/architecture.md](doc/architecture.md) | 架构与设计约束 |
| [doc/roadmap.md](doc/roadmap.md) | 实现路线图 |

## License

Apache-2.0
