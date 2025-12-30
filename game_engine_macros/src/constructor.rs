//! # Constructor派生宏
//!
//! 自动为结构体生成构造函数。
//!
//! ## 示例
//!
//! ```
//! use game_engine_macros::Constructor;
//!
//! #[derive(Constructor)]
//! pub struct Point {
//!     x: f32,
//!     y: f32,
//! }
//!
//! // 自动生成:
//! // impl Point {
//! //     pub fn new(x: f32, y: f32) -> Self {
//! //         Self { x, y }
//! //     }
//! // }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn impl_constructor_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // 提取字段
    let fields = match &input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "Constructor目前只支持带有命名字段的结构体",
            )
            .to_compile_error()
            .into();
        }
    };

    // 生成构造函数参数
    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();

    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    // 生成构造函数实现
    let expanded = quote! {
        impl #struct_name {
            /// 自动生成的构造函数
            #[inline]
            pub fn new(#(#field_names: #field_types),*) -> Self {
                Self {
                    #(#field_names),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
