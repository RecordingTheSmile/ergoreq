use ergoreq::cookie::cookie_container::ErgoCookieContainer;
use ergoreq::wrappers::client_wrapper::ErgoClient;
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
    let cookie_store = Arc::new(ErgoCookieContainer::new(false, false, false));

    // Each request will automatically set and store cookie.
    client
        .get("https://httpbin.org/cookies/set/test_cookie/test_success")
        .with_cookie_store_ref(&cookie_store)
        .send()
        .await
        .unwrap();

    client
        .get("https://httpbin.org/cookies/set/test_cookie_1/test_success_1")
        .with_cookie_store_ref(&cookie_store)
        .send()
        .await
        .unwrap();

    println!("Cookies: {:#?}", cookie_store.serialize_cookies());
}
