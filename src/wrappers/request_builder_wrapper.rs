use core::fmt;
use http::{HeaderMap, Version};
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Body, Client, Request, RequestBuilder, Response};
use retry_policies::policies::ExponentialBackoff;
use retry_policies::RetryPolicy;
use serde::Serialize;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use crate::cookie::cookie_container::CookieContainer;

use crate::middleware::auto_redirect_middleware::AutoRedirectMiddleware;
use crate::middleware::auto_retry_middleware::AutoRetryMiddleware;
use crate::middleware::middleware::{Middleware, Next};
use crate::wrappers::client_wrapper::ErgoClient;

/// A wrapper for [`reqwest::RequestBuilder`]
pub struct ErgoRequestBuilder {
    inner: RequestBuilder,
    cookie_store: Option<Arc<dyn CookieContainer + 'static>>,
    url: String,
    retry_policy: Option<Arc<dyn RetryPolicy + Send + Sync + 'static>>,
    max_redirect_times: u16,
    client: reqwest::Client,
    client_middleware: Box<[Arc<dyn Middleware>]>,
    request_middleware: Vec<Arc<dyn Middleware>>,
    extensions: http::Extensions,
}

impl ErgoRequestBuilder {
    /// Create a new `ErgoRequestBuilder`
    pub fn new(
        raw_builder: RequestBuilder,
        cookie_store: Option<Arc<dyn CookieContainer>>,
        url: String,
        client: reqwest::Client,
        global_redirect_time: u16,
        global_retry_policy: Option<Arc<dyn RetryPolicy + Send + Sync + 'static>>,
        middlewares: Box<[Arc<dyn Middleware>]>,
    ) -> Self {
        Self {
            inner: raw_builder,
            cookie_store,
            url,
            retry_policy: global_retry_policy,
            max_redirect_times: global_redirect_time,
            client,
            client_middleware: middlewares,
            request_middleware: vec![],
            extensions: http::Extensions::new(),
        }
    }

    /// Add a per-request middleware
    pub fn with_middleware<M>(mut self, middleware: M) -> Self
    where
        M: Middleware,
    {
        self.request_middleware.push(Arc::new(middleware));
        self
    }

    /// Add a per-request middleware
    ///
    /// You can hold an `Arc` to it.
    pub fn with_middleware_arc<M>(mut self, middleware: Arc<M>) -> Self
    where
        M: Middleware,
    {
        self.request_middleware.push(middleware);
        self
    }

    /// Set `CookieStore` for this request.
    ///
    /// `Arc`-ed `CookieContainer` will be cloned.
    pub fn with_cookie_store_ref<C>(mut self, cookie_store: &Arc<C>) -> Self
    where
        C: CookieContainer + 'static,
    {
        self.cookie_store = Some(cookie_store.to_owned());
        self
    }

    /// Set `CookieStore` for this request.
    pub fn with_cookie_store<C>(mut self, cookie_store: Arc<C>) -> Self
    where
        C: CookieContainer + 'static,
    {
        self.cookie_store = Some(cookie_store);
        self
    }

    /// Set `extension` for this request.
    ///
    /// `Extensions` can be get and pass some useful information for middlewares.
    pub fn with_extension<T>(mut self, extension: T) -> Self
    where
        T: Send + Sync + 'static + Clone,
    {
        self.extensions.insert(extension);
        self
    }

    /// Remove a kind of `extension` for this request.
    pub fn remove_extension<T>(mut self) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.extensions.remove::<T>();
        self
    }

    /// Get a mutable `extension` if it is set for this request.
    /// Otherwise, `None` is returned.
    pub fn get_extension_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.extensions.get_mut()
    }

    /// Get the inner [`RequestBuilder`]
    pub fn into_inner(self) -> RequestBuilder {
        self.inner
    }

    /// See [`RequestBuilder::from_parts`]
    pub fn from_parts(client: Client, request: Request) -> Self {
        let url = request.url().to_string();
        Self {
            inner: RequestBuilder::from_parts(client.to_owned(), request),
            cookie_store: None,
            url,
            retry_policy: None,
            max_redirect_times: 0,
            client,
            client_middleware: Box::new([]),
            request_middleware: vec![],
            extensions: http::Extensions::new(),
        }
    }

    /// See [`RequestBuilder::header`]
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.inner = self.inner.header(key, value);
        self
    }

    /// Set `retry_times` to this request
    ///
    /// If you don't want to retry, set this to `0`
    ///
    /// ## Notice
    /// `Retry` **will be unavailable** if `body` of this request is `stream`
    pub fn with_retry_times(mut self, retry_times: u16) -> Self {
        if retry_times == 0 {
            self.retry_policy = None;
        } else {
            self.retry_policy = Some(Arc::new(
                ExponentialBackoff::builder().build_with_max_retries(retry_times.into()),
            ));
        }

        self
    }

    /// Set custom `retry_policy` to this request
    ///
    /// ## Notice
    /// `Retry` **will be unavailable** if `body` of this request is `stream`
    pub fn with_retry_policy<T>(mut self, retry_policy: T) -> Self
    where
        T: RetryPolicy + Send + Sync + 'static,
    {
        self.retry_policy = Some(Arc::new(retry_policy));
        self
    }

    /// Set `max_redirect_times` to this request.
    ///
    /// If you don't want to redirect, set this to `0`
    ///
    /// ## Notice
    /// `AutoRedirection` **will not copy `body`** if `body` of this request is `stream`
    pub fn with_max_redirection(mut self, max_redirection: u16) -> Self {
        self.max_redirect_times = max_redirection;
        self
    }

    /// See [`RequestBuilder::headers`]
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.inner = self.inner.headers(headers);
        self
    }

    /// See [`RequestBuilder::basic_auth`]
    pub fn basic_auth<U, P>(mut self, username: U, password: Option<P>) -> Self
    where
        U: fmt::Display,
        P: fmt::Display,
    {
        self.inner = self.inner.basic_auth(username, password);
        self
    }

    /// See [`RequestBuilder::bearer_auth`]
    pub fn bearer_auth<T>(mut self, token: T) -> Self
    where
        T: fmt::Display,
    {
        self.inner = self.inner.bearer_auth(token);
        self
    }

    /// See [`RequestBuilder::body`]
    pub fn body<T: Into<Body>>(mut self, body: T) -> Self {
        self.inner = self.inner.body(body);
        self
    }

    /// See [`RequestBuilder::timeout`]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.inner = self.inner.timeout(timeout);
        self
    }

    /// See [`RequestBuilder::query`]
    pub fn query<T: Serialize + ?Sized>(mut self, query: &T) -> Self {
        self.inner = self.inner.query(query);
        self
    }

    /// See [`RequestBuilder::version`]
    pub fn version(mut self, version: Version) -> Self {
        self.inner = self.inner.version(version);
        self
    }

    /// See [`RequestBuilder::form`]
    pub fn form<T: Serialize + ?Sized>(mut self, form: &T) -> Self {
        self.inner = self.inner.form(form);
        self
    }

    /// See [`RequestBuilder::json`]
    pub fn json<T: Serialize + ?Sized>(mut self, json: &T) -> Self {
        self.inner = self.inner.json(json);
        self
    }

    /// See [`RequestBuilder::fetch_mode_no_cors`]
    pub fn fetch_mode_no_cors(mut self) -> Self {
        self.inner = self.inner.fetch_mode_no_cors();
        self
    }

    /// See [`RequestBuilder::multipart`]
    ///
    /// ## Notice
    /// Due to multipart body is a `stream`, `Retry` will be unavailable if request method is not `GET`,
    /// and `AutoRedirect` will not copy body
    /// in 308, 307
    pub fn multipart(mut self, multipart: reqwest::multipart::Form) -> Self {
        self.inner = self.inner.multipart(multipart);
        self
    }

    /// See [`RequestBuilder::build`]
    pub fn build(self) -> reqwest::Result<Request> {
        let mut build_result = self.inner.build()?;
        if let Some(cookie_store) = self.cookie_store {
            let url = build_result.url();
            let cookie_header = cookie_store.to_header_value(url);
            if let Ok(cookie_header) = HeaderValue::from_str(&cookie_header.join("; ")) {
                let headers = build_result.headers_mut();
                headers.insert(http::header::COOKIE, cookie_header);
            }

            Ok(build_result)
        } else {
            Ok(build_result)
        }
    }

    /// See [`RequestBuilder::build_split`]
    pub fn build_split(self) -> (ErgoClient, reqwest::Result<Request>) {
        let (client, build_result) = self.inner.build_split();
        if let Ok(mut build_result) = build_result {
            if let Some(cookie_store) = self.cookie_store {
                let url = build_result.url();
                let cookie_header = cookie_store.to_header_value(url);
                if let Ok(cookie_header) = HeaderValue::from_str(&cookie_header.join("; ")) {
                    let headers = build_result.headers_mut();
                    headers.insert(http::header::COOKIE, cookie_header);
                }

                (ErgoClient::new(client), Ok(build_result))
            } else {
                (ErgoClient::new(client), Ok(build_result))
            }
        } else {
            (ErgoClient::new(client), build_result)
        }
    }

    /// See [`RequestBuilder::send`]
    ///
    /// Please notice that this method returns `ergoreq::error::Result` instead of
    /// `reqwest::Error`
    pub fn send(self) -> impl Future<Output = crate::error::Result<Response>> {
        async move {
            let mut my_self = self;
            my_self
                .request_middleware
                .splice(0..0, my_self.client_middleware.iter().map(|v| v.to_owned()));

            // judge if insert AutoRedirect middleware is needed
            if my_self.max_redirect_times > 0 {
                let redirect_middleware =
                    AutoRedirectMiddleware::new(my_self.max_redirect_times.into());
                my_self
                    .request_middleware
                    .push(Arc::new(redirect_middleware));
            }

            // judge if insert AutoRetry middleware is needed
            if let Some(policy) = my_self.retry_policy {
                let retry_middleware = AutoRetryMiddleware::new(policy);
                my_self.request_middleware.push(Arc::new(retry_middleware))
            }

            let next = Next::new(
                &my_self.client,
                &my_self.request_middleware,
                my_self.cookie_store,
            );
            let result = next
                .run(my_self.inner.build()?, &mut my_self.extensions)
                .await
                .map_err(crate::error::Error::from)?;
            Ok(result)
        }
    }

    /// See [`RequestBuilder::try_clone`]
    ///
    /// Please notice that this method returns `ErgoRequestBuilder` instead of `reqwest::RequestBuilder`
    pub fn try_clone(&self) -> Option<Self> {
        self.inner.try_clone().and_then(|v| {
            Some(ErgoRequestBuilder::new(
                v,
                self.cookie_store.to_owned(),
                self.url.to_owned(),
                self.client.to_owned(),
                self.max_redirect_times,
                self.retry_policy.to_owned(),
                self.client_middleware.to_owned(),
            ))
        })
    }

    /// Get the inner `cookie_store`.
    pub fn get_cookie_store(&self) -> Option<Arc<dyn CookieContainer + 'static>> {
        self.cookie_store.to_owned()
    }
}
