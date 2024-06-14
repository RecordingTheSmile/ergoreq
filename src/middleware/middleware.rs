use crate::cookie::cookie_container::CookieContainer;
use crate::cookie::cookie_parser::ErgoCookieParser;
use async_trait::async_trait;
use http::{Extensions, HeaderValue};
use reqwest::{Request, Response};
use std::sync::Arc;
use tracing::instrument;

#[async_trait]
pub trait Middleware: 'static + Send + Sync {
    /// Handle each request and can make changes for `Request` and `Response`
    async fn handle(
        &self,
        req: Request,
        ext: &mut Extensions,
        next: Next<'_>,
    ) -> crate::error::Result<Response>;
}

/// This struct is used to execute `Request` with [`Middleware`]s
///
/// `Cookies` will be set and store automatically, so you needn't set `Cookie` header or parse cookie from `Set-Cookie` header manually.
///
/// If you set `Cookie` header manually for `Request`, it will be overwritten if you set a [`CookieContainer`]
/// for this request.
#[derive(Clone)]
pub struct Next<'a> {
    client: &'a reqwest::Client,
    middlewares: &'a [Arc<dyn Middleware>],
    cookie_store: Option<Arc<dyn CookieContainer>>,
}

impl<'a> Next<'a> {
    pub(crate) fn new(
        client: &'a reqwest::Client,
        middlewares: &'a [Arc<dyn Middleware>],
        cookie_store: Option<Arc<dyn CookieContainer>>,
    ) -> Self {
        Self {
            client,
            middlewares,
            cookie_store,
        }
    }

    #[instrument(skip_all)]
    fn store_cookies(cookie_store: Option<Arc<dyn CookieContainer>>, response: &Response) {
        if let Some(store) = cookie_store {
            let cookie_headers = response
                .headers()
                .get_all(http::header::SET_COOKIE)
                .iter()
                .filter_map(|v| v.to_str().ok());
            let parsed_cookies = ErgoCookieParser::parse_set_cookie_header(cookie_headers);
            tracing::debug!("Parsed cookies: {:?}", parsed_cookies);
            store.store_from_response(parsed_cookies, response.url());
        }
    }

    #[instrument(skip_all)]
    fn set_cookie_header(cookie_store: Option<Arc<dyn CookieContainer>>, request: &mut Request) {
        if let Some(cookie_store) = cookie_store {
            let cookie_value = cookie_store.to_header_value(request.url());
            let header_value = HeaderValue::from_str(&cookie_value.join("; "));
            if let Ok(header_value) = header_value {
                tracing::debug!("Will set cookie header: {:?}", header_value);
                request
                    .headers_mut()
                    .insert(http::header::COOKIE, header_value);
            }
        }
    }

    /// Acquired the actual response.
    ///
    /// Run this method will stop running middlewares left for this request permanently.
    #[instrument(skip(self))]
    pub async fn run_without_middleware(self, mut req: Request) -> crate::error::Result<Response> {
        Self::set_cookie_header(self.cookie_store.to_owned(), &mut req);
        let response = self
            .client
            .execute(req)
            .await
            .map_err(crate::error::Error::from)?;
        Self::store_cookies(self.cookie_store, &response);
        Ok(response)
    }

    pub fn get_inner_client_owned(&self) -> reqwest::Client {
        self.client.to_owned()
    }

    /// Pass this `Request` to next middleware, wait for `Response`
    ///
    /// You can pass some useful information by adding [`http::Extensions`] in `extensions` parameter
    #[instrument(skip(self, extensions))]
    pub async fn run(
        mut self,
        mut req: Request,
        extensions: &'a mut Extensions,
    ) -> crate::error::Result<Response> {
        if let Some((current, left)) = self.middlewares.split_first() {
            tracing::debug!("Run request with middleware");
            self.middlewares = left;
            let cookie_container = self.cookie_store.to_owned();
            Self::set_cookie_header(cookie_container.to_owned(), &mut req);
            let response = current
                .handle(req, extensions, self)
                .await
                .map_err(crate::error::Error::from)?;
            Self::store_cookies(cookie_container, &response);
            Ok(response)
        } else {
            tracing::debug!("No middleware found, will run without middleware");
            self.run_without_middleware(req).await
        }
    }
}
