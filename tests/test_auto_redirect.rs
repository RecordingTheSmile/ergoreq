#[cfg(test)]
mod test_auto_redirect {
    use ergoreq::wrappers::client_wrapper::ErgoClient;
    use ergoreq::Error;
    use reqwest::redirect::Policy;

    #[tokio::test]
    async fn test_auto_redirect() {
        let client = reqwest::ClientBuilder::new()
            .redirect(Policy::none())
            .build()
            .unwrap();

        let client = ErgoClient::new(client).with_auto_redirect_count(5);

        let response = client
            .get("https://httpbin.org/redirect/4") // last time it will redirect to /get, so 4 represents 5 redirect times
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());

        let response = client
            .get("https://httpbin.org/redirect/6") // if last time request is 302, then error with too many request
            .send()
            .await
            .unwrap_err();

        match response {
            Error::TooManyRedirect(_, counts) => {
                assert_eq!(counts, 5)
            }
            _ => panic!("response doesn't report an TooManyRedirectError"),
        }
    }

    #[tokio::test]
    async fn test_auto_redirect_with_body() {
        let client = reqwest::ClientBuilder::new()
            .redirect(Policy::none())
            .build()
            .unwrap();

        let client = ErgoClient::new(client).with_auto_redirect_count(5);

        let response = client
            .post("https://httpbin.org/redirect-to")
            .query(&[("url", "/anything"), ("status_code", "308")])
            .body("Hello, World, 308!")
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();

        assert_eq!(response["data"].as_str().unwrap(), "Hello, World, 308!");
        assert_eq!(response["method"].as_str().unwrap(), "POST");

        let response = client
            .put("https://httpbin.org/redirect-to")
            .query(&[("url", "/anything"), ("status_code", "307")])
            .body("Hello, World， 307!")
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();

        assert_eq!(response["data"].as_str().unwrap(), "Hello, World， 307!");
        assert_eq!(response["method"].as_str().unwrap(), "PUT");

        let response = client
            .post("https://httpbin.org/redirect-to")
            .query(&[("url", "/anything"), ("status_code", "302")])
            .body("Hello, World， 302!")
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();

        assert_eq!(response["data"].as_str().unwrap_or_default(), "");
        assert_eq!(response["method"].as_str().unwrap(), "GET");
    }
}
