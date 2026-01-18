# ergoreq

A human-centric web request client developed based on Reqwest.

* Friendly Url building
* Automatically retry support
* Middleware support
* Automatically manage cookies per-request (instead of per-client)
* Automatically redirect management per-request
* Supports all features of `reqwest`
* `tracing` support
* Well tested

# Example

```rust
    use ergoreq::{ErgoClient, ErgoCookieContainer, StringUrlBuilderTrait, ErgoStringToRequestExt};
use reqwest::redirect::Policy;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Create a reqwest client first
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("ergoreq/1.0")
        .redirect(Policy::none()) // remember to disable the redirect!
        .build()
        .unwrap();

    // Then create the ergo client
    let client = ErgoClient::new(client).with_auto_redirect_count(5); // global auto redirect count

    // Creates cookie store. You can impl a `CookieContainer` by your own.
    let cookie_store = Arc::new(ErgoCookieContainer::new_secure());

    // Each request will automatically set and store cookie.
    // You can build url by string directly
    "https://httpbin.org"
        .add_url_segment("cookies")
        .add_url_segment("set")
        .add_url_segment("test_cookie")
        .add_url_segment("test_success")
        .http_get(&client)
        .with_cookie_store_ref(&cookie_store)
        .send()
        .await
        .unwrap();

    // Or with batch url building
    "https://httpbin.org"
        .add_url_segments(&["cookies", "set", "test_cookie_1", "test_success_1"])
        .http_get(&client)
        .with_cookie_store_ref(&cookie_store)
        .send()
        .await
        .unwrap();

    // Traditional way to build url
    client
        .get("https://httpbin.org/cookies/set/test_cookie_1/test_success_1")
        .with_cookie_store_ref(&cookie_store)
        .send()
        .await
        .unwrap();

    println!("Cookies: {:#?}", cookie_store.serialize_cookies());

    let response = client
        .get("https://httpbin.org/redirect/4") // last time it will redirect to /get, so 4 represents 5 redirect times
        .send()
        .await
        .unwrap();

    println!(
        "Redirect {}",
        if response.status().is_success() {
            "success"
        } else {
            "fail"
        }
    )
}
```

More examples can be found in [examples](examples) directory.

# Requirement

The current version supports `reqwest` version `^0.13`.

# License

MIT

# Thanks to

* [reqwest](https://github.com/seanmonstar/reqwest)