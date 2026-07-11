---
name: rust-verify
description: Rust 代码修改完成后的验证流程：fmt、test、clippy
---

# Rust 验证流程

完成 Rust 代码修改、准备提交、或用户要求验证时执行。

## 步骤

1. `cargo fmt --all --check`；若格式不对则 `cargo fmt --all` 并说明改动
2. `cargo test --all`
3. 可选：`cargo clippy --all-targets --all-features -- -D warnings`（若环境可用）
4. 可选：`cargo doc --no-deps`（公开 API 变更时）
5. 任一失败则修复后重跑，不声称任务完成
6. 向用户汇报：通过项、失败项、修复内容

## 注意

- 除测试外，生产代码避免 `unwrap()` / `expect()`
- workspace 根目录执行，覆盖 `ldb` / `ldb-core` / `ldb-macros` 全部成员
