use async_trait::async_trait;
use http::{Extensions, Method};
use reqwest::{Request, Response};

use super::middleware::{Middleware, Next};

/// Perform the auto redirect for request.
pub(crate) struct AutoRedirectMiddleware(u64);

impl AutoRedirectMiddleware {
    pub fn new(max_redirect_count: u64) -> Self {
        Self(max_redirect_count)
    }
}

#[async_trait]
impl Middleware for AutoRedirectMiddleware {
    async fn handle(
        &self,
        req: Request,
        ext: &mut Extensions,
        next: Next<'_>,
    ) -> crate::error::Result<Response> {
        let mut current_redirect_count = 0;
        let origin_body = req
            .body()
            .and_then(|v| v.as_bytes())
            .and_then(|v| Some(v.to_vec()));
        let origin_headers = req.headers().to_owned();
        let origin_method = req.method().to_owned();
        let origin_url = req.url().to_owned();
        let inner_client = next.get_inner_client_owned();

        let mut response = next.run(req, ext).await?;

        if !response.status().is_redirection() {
            return Ok(response);
        }

        loop {
            if current_redirect_count >= self.0 {
                if response.status().is_redirection() {
                    return Err(crate::Error::TooManyRedirect(current_redirect_count));
                }
                break;
            }
            let new_url = response
                .headers()
                .get(http::header::LOCATION)
                .and_then(|v| v.to_str().ok());

            // if new_url is invalid or is empty, then stop trying to redirect
            let new_url_str = match new_url {
                Some(url) => url,
                None => return Ok(response),
            };

            let mut new_url: http::Uri = new_url_str
                .parse()
                .map_err(|_| crate::Error::InvalidRedirectUrl(new_url_str.to_owned()))?;

            // if host is None, then new_url may be a relative path
            if new_url.host().is_none() {
                new_url = http::Uri::builder()
                    .authority(origin_url.authority())
                    .scheme(origin_url.scheme())
                    .path_and_query(new_url_str)
                    .build()?;
            }

            let new_method = match response.status().as_u16() {
                307 | 308 => origin_method.to_owned(),
                _ => Method::GET,
            };

            let new_request = http::Request::builder()
                .uri::<http::uri::Uri>(new_url.into())
                .method(new_method.to_owned());

            let mut new_request = match new_method {
                Method::GET => new_request.body(vec![])?,
                _ => new_request.body(origin_body.to_owned().unwrap_or_default())?,
            };

            *new_request.headers_mut() = origin_headers.to_owned();

            response = inner_client.execute(new_request.try_into()?).await?;
            current_redirect_count += 1;
        }

        Ok(response)
    }
}
