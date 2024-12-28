#[cfg(test)]
mod test_build_url_with_string {
    use ergoreq::cookie::cookie_container::ErgoCookieContainer;
    use ergoreq::wrappers::client_wrapper::ErgoClient;
    use ergoreq::{ErgoStringToRequestExt, StringUrlBuilderTrait};
    use reqwest::redirect::Policy;
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_build_url() {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("ergoreq/1.0")
            .redirect(Policy::none())
            .build()
            .unwrap();

        let client = ErgoClient::new(client).with_auto_redirect_count(5);
        let cookie_store = Arc::new(ErgoCookieContainer::new_secure());
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
    }
}
