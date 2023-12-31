//!
//! # ergoreq
//!
//! ---
//! A human-centric web request client developed based on Reqwest.
//!
//! * Automatically retry support
//! * Middleware support
//! * Automatically manage cookies per-request (instead of per-client)
//! * Automatically redirect management per-request
//! * Supports all features of `reqwest`
//! * Well tested
//!
//! # Example
//! ```rust
//!     #[tokio::main]
//!     async fn main(){
//!         use std::sync::Arc;
//! // Create a reqwest client first
//!         # use std::time::Duration;
//!         # use reqwest::redirect::Policy;
//!         # use ergoreq::cookie::cookie_container::ErgoCookieContainer;
//!         # use ergoreq::wrappers::client_wrapper::ErgoClient;
//!     let client = reqwest::Client::builder()
//!             .timeout(Duration::from_secs(30))
//!             .user_agent("ergoreq/1.0")
//!             .redirect(Policy::none()) // remember to disable the redirect!
//!             .build()
//!             .unwrap();
//!
//!         // Then create the ergo client
//!         let client = ErgoClient::new(client).with_auto_redirect_count(5); // global auto redirect count
//!
//!         // Creates cookie store. You can impl a `CookieContainer` by your own.
//!         let cookie_store = Arc::new(ErgoCookieContainer::new(false, false, false));
//!
//!         // Each request will automatically set and store cookie.
//!         client
//!             .get("https://httpbin.org/cookies/set/test_cookie/test_success")
//!             .with_cookie_store_ref(&cookie_store)
//!             .send()
//!             .await
//!             .unwrap();
//!
//!         client
//!             .get("https://httpbin.org/cookies/set/test_cookie_1/test_success_1")
//!             .with_cookie_store_ref(&cookie_store)
//!             .send()
//!             .await
//!             .unwrap();
//!
//!         println!("Cookies: {:#?}", cookie_store.serialize_cookies());
//!
//!         let response = client
//!             .get("https://httpbin.org/redirect/4") // last time it will redirect to /get, so 4 represents 5 redirect times
//!             .send()
//!             .await
//!             .unwrap();
//!     
//!         println!(
//!             "Redirect {}",
//!             if response.status().is_success() {
//!                 "success"
//!             } else {
//!                 "fail"
//!             }
//!         )
//!     }
//! ```
//! More examples can be found in examples directory.
//!
//! # Requirement
//! Same as reqwest.
//!
//! # License
//! MIT
//!
//! # Thanks to
//! * [reqwest](https://github.com/seanmonstar/reqwest)
pub mod wrappers;

pub mod cookie;

pub mod middleware;

pub mod error;

pub use crate::error::Error;
pub use crate::error::Result;
pub use async_trait::async_trait;
pub use cookie as cookie_process;
pub use dashmap;
pub use http;
pub use retry_policies;
pub use url;
