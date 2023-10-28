use std::{ops::Deref, sync::Arc};

use reqwest::{IntoUrl, Method};

use crate::middleware::middleware::Middleware;

use super::request_builder_wrapper::ErgoRequestBuilder;

///
/// `ErgoClient` is a wrapper of `reqwest::Client`
#[derive(Clone)]
pub struct ErgoClient {
    inner: reqwest::Client,
    middlewares: Vec<Arc<dyn Middleware>>,
    global_auto_redirect: u16,
}

macro_rules! impl_method_wrap {
    ($($method:ident),+) => {
        $(
            paste::paste!{
            #[doc = "Return a `ErgoRequestBuilder` for `" $method "` method."]
            pub fn $method<U: reqwest::IntoUrl>(&self,url: U)->crate::wrappers::request_builder_wrapper::ErgoRequestBuilder{
                let url_str = url.as_str().to_owned();
                crate::wrappers::request_builder_wrapper::ErgoRequestBuilder::new(self.inner.$method(url), None,url_str, self.inner.to_owned(), self.global_auto_redirect, self.middlewares.to_owned().into_boxed_slice())
        }
    }
    )+
    };
}

impl ErgoClient {
    /// Create a new `ErgoClient` with a created `reqwest::Client`
    ///
    /// # Notice
    /// `redirect_policy` in `reqwest::Client` should be set to `none`
    ///
    /// # Example
    /// ```
    /// # use ergoreq::wrappers::client_wrapper::ErgoClient;
    /// let inner_client = reqwest::Client::builder()
    /// .redirect(reqwest::redirect::Policy::none())
    /// .build()
    /// .unwrap();
    ///
    /// let client = ErgoClient::new(inner_client);
    /// ```
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            inner: client,
            middlewares: vec![],
            global_auto_redirect: 0,
        }
    }

    /// Set a global auto redirect count.
    /// This count will be passed to every request initialized by this client.
    ///
    /// This can be overwritten by each request (use [`ErgoRequestBuilder::with_max_redirection`]).
    pub fn with_auto_redirect_count(mut self, count: u16) -> Self {
        self.global_auto_redirect = count;
        self
    }

    /// Set a global middleware.
    ///
    /// This middleware will be passed to every request.
    ///
    /// # Notice
    /// Global middleware will be executed before request middleware.
    pub fn with_middleware<M>(mut self, middleware: M) -> Self
    where
        M: Middleware,
    {
        self.middlewares.push(Arc::new(middleware));
        self
    }

    /// Set a global middleware. You can hold an `Arc` for this middleware.
    ///
    /// This middleware will be passed to every request.
    ///
    /// # Notice
    /// Global middleware will be executed before request middleware.
    pub fn with_middleware_arc<M>(mut self, middleware: Arc<M>) -> Self
    where
        M: Middleware,
    {
        self.middlewares.push(middleware);
        self
    }

    impl_method_wrap!(get, post, put, patch, delete, head);

    /// Build an `ErgoRequestBuilder` with given `Method` and `Url`
    pub fn request(&self, method: Method, url: impl IntoUrl) -> ErgoRequestBuilder {
        let url_str = url.as_str().to_owned();
        ErgoRequestBuilder::new(
            self.inner.request(method, url),
            None,
            url_str,
            self.inner.to_owned(),
            self.global_auto_redirect,
            self.middlewares.to_owned().into_boxed_slice(),
        )
    }
}

impl Deref for ErgoClient {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod test_client_wrapper {
    use reqwest::Method;

    use super::ErgoClient;

    macro_rules! impl_method_test {
        ($($method:ident),+) => {
            $(
                paste::paste!{
                    #[test]
                    fn [<test_perform_ $method _request>]() {
                        let client = reqwest::Client::new();
                        let client = ErgoClient::new(client);
                        let request_builder = client.$method("https://crates.io");
                        assert_eq!(
                            request_builder
                                .into_inner()
                                .try_clone()
                                .unwrap()
                                .build()
                                .unwrap()
                                .method(),
                            &Method::[<$method:upper>]
                        )
                    }
                }
            )+
        };
    }

    impl_method_test!(get, post, put, patch, delete, head);
}
