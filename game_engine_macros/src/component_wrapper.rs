//! # ComponentWrapper派生宏
//!
//! 为ECS组件包装器自动生成实现。
//!
//! ## 示例
//!
//! ```
//! use game_engine_macros::ComponentWrapper;
//! use bevy_ecs::component::Component;
//!
//! #[derive(ComponentWrapper)]
//! pub struct Velocity(pub glam::Vec3);
//!
//! // 自动生成:
//! // - impl Component for Velocity
//! // - impl Deref for Velocity
//! // - impl DerefMut for Velocity
//! // - impl From<glam::Vec3> for Velocity
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn impl_component_wrapper_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // 检查是否是元组结构体
    let inner_type = match &input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Unnamed(fields),
            ..
        }) => {
            if fields.unnamed.len() != 1 {
                return syn::Error::new_spanned(&input.ident, "ComponentWrapper只支持单字段包装器")
                    .to_compile_error()
                    .into();
            }
            &fields.unnamed[0].ty
        }
        _ => {
            return syn::Error::new_spanned(&input.ident, "ComponentWrapper只支持元组结构体")
                .to_compile_error()
                .into();
        }
    };

    // 生成Component、Deref和From实现(不重新定义结构体)
    let expanded = quote! {
        // 为现有结构体添加Component derive
        impl bevy_ecs::component::Component for #struct_name {}

        // Deref实现
        impl std::ops::Deref for #struct_name {
            type Target = #inner_type;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        // DerefMut实现
        impl std::ops::DerefMut for #struct_name {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        // From实现
        impl From<#inner_type> for #struct_name {
            #[inline]
            fn from(value: #inner_type) -> Self {
                Self(value)
            }
        }
    };

    TokenStream::from(expanded)
}
