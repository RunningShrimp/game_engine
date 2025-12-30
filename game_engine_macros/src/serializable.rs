//! # Serializable派生宏
//!
//! 自动生成序列化和反序列化方法。
//!
//! ## 示例
//!
//! ```
//! use game_engine_macros::Serializable;
//!
//! #[derive(Serializable, serde::Serialize, serde::Deserialize)]
//! pub struct GameState {
//!     score: u32,
//!     level: u32,
//! }
//!
//! // 自动生成:
//! // impl GameState {
//! //     pub fn serialize(&self) -> Result<Vec<u8>, SerializationError>;
//! //     pub fn deserialize(data: &[u8]) -> Result<Self, SerializationError>;
//! // }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn impl_serializable_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // 生成Serializable trait实现 (使用bincode 1.3 API)
    let expanded = quote! {
        impl #struct_name {
            /// 序列化到二进制格式
            pub fn serialize(&self) -> Result<Vec<u8>, crate::error::SerializationError> {
                bincode::serialize(self)
                    .map_err(|e| crate::error::SerializationError::Encode(e.to_string()))
            }

            /// 从二进制格式反序列化
            pub fn deserialize(data: &[u8]) -> Result<Self, crate::error::SerializationError>
            where
                Self: Sized,
            {
                bincode::deserialize(data)
                    .map_err(|e| crate::error::SerializationError::Decode(e.to_string()))
            }
        }
    };

    TokenStream::from(expanded)
}
