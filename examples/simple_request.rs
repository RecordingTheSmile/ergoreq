use ergoreq::wrappers::client_wrapper::ErgoClient;
use ergoreq::ErgoStringToRequestExt;
use reqwest::redirect::Policy;
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

    let response = "https://crates.io"
        .http_get(&client)
        .with_retry_times(5) // if request meets an error, retry 5 times automatically
        // or use `.with_retry_policy()` to customize retry policy
        .with_max_redirection(10) // overwrite the global auto redirect count
        .send()
        .await
        .unwrap();

    println!("Response status: {}", response.status());
}
