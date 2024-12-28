macro_rules! impl_string_req_method_in_trait {
    ($($method:ident),+) => {
        $(
            paste::paste!{
            #[doc = "Create a `ErgoRequestBuilder` for `" $method "` method with given String as url."]
            fn [<http_ $method>](&self, client: &crate::wrappers::client_wrapper::ErgoClient)->crate::wrappers::request_builder_wrapper::ErgoRequestBuilder{
                self.http_request(client, reqwest::Method::[<$method:upper>])
            }
    }
    )+
    };
}

/// Extension trait for `String` to create a `ErgoRequestBuilder` with given method.
pub trait ErgoStringToRequestExt {
    impl_string_req_method_in_trait!(get, post, put, delete, head, options, patch);

    fn http_request(
        &self,
        client: &crate::wrappers::client_wrapper::ErgoClient,
        method: reqwest::Method,
    ) -> crate::wrappers::request_builder_wrapper::ErgoRequestBuilder;
}

impl ErgoStringToRequestExt for String {
    fn http_request(
        &self,
        client: &crate::wrappers::client_wrapper::ErgoClient,
        method: reqwest::Method,
    ) -> crate::wrappers::request_builder_wrapper::ErgoRequestBuilder {
        client.request(method, self)
    }
}

impl ErgoStringToRequestExt for &str {
    fn http_request(
        &self,
        client: &crate::wrappers::client_wrapper::ErgoClient,
        method: reqwest::Method,
    ) -> crate::wrappers::request_builder_wrapper::ErgoRequestBuilder {
        client.request(method, *self)
    }
}

impl ErgoStringToRequestExt for &String {
    fn http_request(
        &self,
        client: &crate::wrappers::client_wrapper::ErgoClient,
        method: reqwest::Method,
    ) -> crate::wrappers::request_builder_wrapper::ErgoRequestBuilder {
        client.request(method, *self)
    }
}

#[cfg(test)]
mod test_string_ext {
    use crate::{utils::string_ext::ErgoStringToRequestExt, wrappers::client_wrapper::ErgoClient};
    use reqwest::Method;

    #[test]
    fn test_http_request() {
        let client = reqwest::Client::new();
        let client = ErgoClient::new(client);
        let request_builder = "https://crates.io".http_get(&client);
        assert_eq!(
            request_builder
                .into_inner()
                .try_clone()
                .unwrap()
                .build()
                .unwrap()
                .method(),
            &Method::GET
        )
    }
}
