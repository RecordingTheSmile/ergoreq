#[cfg(test)]
mod test_cookie_container {
    use ergoreq::cookie::cookie_container::ErgoCookieContainer;
    use ergoreq::wrappers::client_wrapper::ErgoClient;
    use reqwest::redirect::Policy;

    use std::sync::Arc;

    #[tokio::test]
    async fn test_cookie_container_parse_and_store() {
        let client = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()
            .unwrap();

        let client = ErgoClient::new(client);
        let cookie_store = Arc::new(ErgoCookieContainer::new(false, false, false));
        client
            .get("https://httpbin.org/cookies/set/test_cookie/test_success")
            .with_cookie_store_ref(&cookie_store)
            .send()
            .await
            .unwrap();
        assert_ne!(cookie_store.serialize_cookies().len(), 0);
        let cookies = cookie_store.serialize_cookies();
        let (first_cookie, _) = cookies.first().unwrap();
        assert_eq!(first_cookie.name(), "test_cookie");
        assert_eq!(first_cookie.value(), "test_success");
    }

    #[tokio::test]
    async fn test_cookie_removal() {
        let client = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()
            .unwrap();

        let client = ErgoClient::new(client);
        let cookie_store = Arc::new(ErgoCookieContainer::new(false, false, false));
        client
            .get("https://httpbin.org/cookies/set/test_cookie/test_success")
            .with_cookie_store_ref(&cookie_store)
            .send()
            .await
            .unwrap();
        assert_ne!(cookie_store.serialize_cookies().len(), 0);
        let cookies = cookie_store.serialize_cookies();
        let (first_cookie, _) = cookies.first().unwrap();
        assert_eq!(first_cookie.name(), "test_cookie");
        assert_eq!(first_cookie.value(), "test_success");

        client
            .get("https://httpbin.org/cookies/delete")
            .query(&[("test_cookie", "")])
            .with_cookie_store_ref(&cookie_store)
            .send()
            .await
            .unwrap();

        assert_eq!(cookie_store.serialize_cookies().len(), 0);
    }
}
