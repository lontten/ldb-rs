//! `#[derive(LdbModel)]` 过程宏。

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

/// 从结构体派生 ldb 模型元数据（表名、主键、`db` 列映射）。
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
            let field_name = f.ident.as_ref().unwrap();
            let field_name_str = field_name.to_string();
            let mut column_name = field_name_str.clone();
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
            quote! {
                ::ldb_core::model::ColumnMeta {
                    field_name: #field_name_str,
                    column_name: #column_name,
                }
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
        }
    };

    expanded.into()
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
