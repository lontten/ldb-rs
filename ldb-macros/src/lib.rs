//! `#[derive(LdbModel)]` 过程宏 crate（骨架阶段尚未实现）。

use proc_macro::TokenStream;
use quote::quote;

/// 从结构体派生 ldb 模型元数据（表名、主键、`db` 列映射）。
#[proc_macro_derive(LdbModel)]
pub fn derive_ldb_model(_input: TokenStream) -> TokenStream {
    TokenStream::from(quote! {
        compile_error!("LdbModel derive 尚未实现");
    })
}
