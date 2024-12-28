#![allow(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]
pub mod wrappers;

pub mod cookie;

pub mod middleware;

pub mod error;

pub mod utils;

pub use crate::cookie::cookie_container::ErgoCookieContainer;
pub use crate::error::Error;
pub use crate::error::Result;
pub use crate::wrappers::client_wrapper::ErgoClient;
pub use crate::wrappers::request_builder_wrapper::ErgoRequestBuilder;
pub use async_trait::async_trait;
pub use cookie as cookie_process;
pub use dashmap;
pub use http;
pub use retry_policies;
pub use url;
pub use utils::string_ext::ErgoStringToRequestExt;
pub use utils::string_url_builder::StringUrlBuilderTrait;
