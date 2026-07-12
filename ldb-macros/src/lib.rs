//! `#[derive(LdbModel)]` 过程宏。

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

/// 从结构体派生 ldb 模型元数据（表名、主键、`db` 列映射）及 SQL 列名关联常量。
///
/// 关联常量命名与字段一致（如 `User::name`），值为 SQL 列名；勿在自有 `impl` 中重复定义同名项。
#[proc_macro_derive(LdbModel, attributes(ldb, db))]
pub fn derive_ldb_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut table_name = None;
    let mut primary_key_list: Vec<String> = vec![];
    let mut auto_column = None;

    for attr in &input.attrs {
        if attr.path().is_ident("ldb") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("table") {
                    let value = meta.value()?;
                    let s: syn::LitStr = value.parse()?;
                    table_name = Some(s.value());
                } else if meta.path.is_ident("primary_key") {
                    let value = meta.value()?;
                    let s: syn::LitStr = value.parse()?;
                    primary_key_list.push(s.value());
                } else if meta.path.is_ident("auto_column") {
                    let value = meta.value()?;
                    let s: syn::LitStr = value.parse()?;
                    auto_column = Some(s.value());
                }
                Ok(())
            });
        }
    }

    let table_name = table_name.unwrap_or_else(|| name.to_string().to_lowercase());
    let auto_column_tokens = match &auto_column {
        Some(c) => quote! { Some(#c) },
        None => quote! { None },
    };
    let pk_tokens: Vec<_> = primary_key_list.iter().map(|s| quote! { #s }).collect();

    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => {
            return syn::Error::new_spanned(&input.ident, "LdbModel 仅支持 struct")
                .to_compile_error()
                .into();
        }
    };

    let Fields::Named(named) = fields else {
        return syn::Error::new_spanned(&input.ident, "LdbModel 需要命名字段")
            .to_compile_error()
            .into();
    };

    let field_meta: Vec<_> = named
        .named
        .iter()
        .map(|f| {
            let (field_name, column_name) = column_name_for_field(f);
            let field_name_str = field_name.to_string();
            quote! {
                ::ldb_core::model::ColumnMeta {
                    field_name: #field_name_str,
                    column_name: #column_name,
                }
            }
        })
        .collect();

    let column_const_list: Vec<_> = named
        .named
        .iter()
        .map(|f| {
            let (field_name, column_name) = column_name_for_field(f);
            quote! {
                #[doc = "SQL 列名，与结构体字段同名。"]
                pub const #field_name: &'static str = #column_name;
            }
        })
        .collect();

    let field_match_arms: Vec<_> = named
        .named
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            let field_name_str = field_name.to_string();
            let value_expr = option_field_to_sql_value(&f.ty, field_name);
            quote! {
                #field_name_str => #value_expr
            }
        })
        .collect();

    let set_field_match_arms: Vec<_> = named
        .named
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            let field_name_str = field_name.to_string();
            let assign_expr = option_field_from_sql_value(&f.ty, field_name);
            quote! {
                #field_name_str => #assign_expr
            }
        })
        .collect();

    let expanded = quote! {
        impl ::ldb_core::model::LdbModel for #name {
            fn table_conf() -> &'static ::ldb_core::model::TableConf {
                static CONF: ::ldb_core::model::TableConf = ::ldb_core::model::TableConf {
                    table_name: #table_name,
                    primary_key_column_name_list: &[#(#pk_tokens),*],
                    auto_column: #auto_column_tokens,
                };
                &CONF
            }

            fn column_meta_list() -> &'static [::ldb_core::model::ColumnMeta] {
                &[#(#field_meta),*]
            }

            fn field_sql_value(&self, field_name: &str) -> Option<::ldb_core::sql_value::SqlValue> {
                match field_name {
                    #(#field_match_arms,)*
                    _ => None,
                }
            }

            fn set_field_sql_value(
                &mut self,
                field_name: &str,
                value: ::ldb_core::sql_value::SqlValue,
            ) -> Result<(), ::ldb_core::error::LdbError> {
                match field_name {
                    #(#set_field_match_arms,)*
                    _ => Ok(()),
                }
            }
        }

        #[allow(non_upper_case_globals)]
        impl #name {
            #(#column_const_list)*
        }
    };

    expanded.into()
}

fn column_name_for_field(f: &syn::Field) -> (syn::Ident, String) {
    let field_name = f.ident.as_ref().unwrap().clone();
    let mut column_name = field_name.to_string();
    for attr in &f.attrs {
        if attr.path().is_ident("db") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("column") {
                    let value = meta.value()?;
                    let s: syn::LitStr = value.parse()?;
                    column_name = s.value();
                }
                Ok(())
            });
        }
    }
    (field_name, column_name)
}

fn option_field_to_sql_value(ty: &Type, field_ident: &syn::Ident) -> proc_macro2::TokenStream {
    let Type::Path(type_path) = ty else {
        return quote! { None };
    };
    let Some(segment) = type_path.path.segments.last() else {
        return quote! { None };
    };
    if segment.ident != "Option" {
        return quote! { None };
    }
    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        let inner_str = quote!(#inner).to_string().replace(' ', "");
        return match inner_str.as_str() {
            "i64" | "i32" | "u64" | "u32" => {
                quote! { self.#field_ident.map(|v| ::ldb_core::sql_value::SqlValue::I64(v as i64)) }
            }
            "String" => quote! {
                self.#field_ident.as_ref().map(|v| ::ldb_core::sql_value::SqlValue::String(v.clone()))
            },
            "bool" => quote! {
                self.#field_ident.map(|v| ::ldb_core::sql_value::SqlValue::Bool(v))
            },
            "f64" => quote! {
                self.#field_ident.map(|v| ::ldb_core::sql_value::SqlValue::F64(v))
            },
            _ => quote! { None },
        };
    }
    quote! { None }
}

fn option_field_from_sql_value(ty: &Type, field_ident: &syn::Ident) -> proc_macro2::TokenStream {
    let Type::Path(type_path) = ty else {
        return quote! { Ok(()) };
    };
    let Some(segment) = type_path.path.segments.last() else {
        return quote! { Ok(()) };
    };
    if segment.ident != "Option" {
        return quote! { Ok(()) };
    }
    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        let inner_str = quote!(#inner).to_string().replace(' ', "");
        let mismatch = quote! {
            Err(::ldb_core::error::LdbError::ModelMapping(format!(
                "字段 `{}` 类型不匹配",
                field_name
            )))
        };
        return match inner_str.as_str() {
            "i64" => quote! {
                match value {
                    ::ldb_core::sql_value::SqlValue::Null => {
                        self.#field_ident = None;
                        Ok(())
                    }
                    ::ldb_core::sql_value::SqlValue::I64(n) => {
                        self.#field_ident = Some(n);
                        Ok(())
                    }
                    _ => #mismatch,
                }
            },
            "i32" | "u64" | "u32" => quote! {
                match value {
                    ::ldb_core::sql_value::SqlValue::Null => {
                        self.#field_ident = None;
                        Ok(())
                    }
                    ::ldb_core::sql_value::SqlValue::I64(n) => {
                        self.#field_ident = Some(n as #inner);
                        Ok(())
                    }
                    _ => #mismatch,
                }
            },
            "String" => quote! {
                match value {
                    ::ldb_core::sql_value::SqlValue::Null => {
                        self.#field_ident = None;
                        Ok(())
                    }
                    ::ldb_core::sql_value::SqlValue::String(s) => {
                        self.#field_ident = Some(s);
                        Ok(())
                    }
                    _ => #mismatch,
                }
            },
            "bool" => quote! {
                match value {
                    ::ldb_core::sql_value::SqlValue::Null => {
                        self.#field_ident = None;
                        Ok(())
                    }
                    ::ldb_core::sql_value::SqlValue::Bool(b) => {
                        self.#field_ident = Some(b);
                        Ok(())
                    }
                    _ => #mismatch,
                }
            },
            "f64" => quote! {
                match value {
                    ::ldb_core::sql_value::SqlValue::Null => {
                        self.#field_ident = None;
                        Ok(())
                    }
                    ::ldb_core::sql_value::SqlValue::F64(n) => {
                        self.#field_ident = Some(n);
                        Ok(())
                    }
                    ::ldb_core::sql_value::SqlValue::I64(n) => {
                        self.#field_ident = Some(n as f64);
                        Ok(())
                    }
                    _ => #mismatch,
                }
            },
            _ => quote! { Ok(()) },
        };
    }
    quote! { Ok(()) }
}
