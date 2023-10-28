use ergoreq::wrappers::client_wrapper::ErgoClient;
use reqwest::redirect::Policy;

#[tokio::main]
async fn main() {
    let client = reqwest::ClientBuilder::new()
        .redirect(Policy::none()) // Remember this!
        .build()
        .unwrap();

    let client = ErgoClient::new(client).with_auto_redirect_count(5);

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
