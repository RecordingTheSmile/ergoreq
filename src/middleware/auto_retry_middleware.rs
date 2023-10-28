use super::middleware::Middleware;
use crate::middleware::middleware::Next;
use async_trait::async_trait;
use chrono::Utc;
use http::Extensions;
use reqwest::{Request, Response};
use retry_policies::{RetryDecision, RetryPolicy};

pub(crate) struct AutoRetryMiddleware(Box<dyn RetryPolicy + Send + Sync + 'static>);

impl AutoRetryMiddleware {
    pub fn new(policy: Box<dyn RetryPolicy + Send + Sync + 'static>) -> Self {
        Self(policy)
    }
}

#[async_trait]
impl Middleware for AutoRetryMiddleware {
    async fn handle(
        &self,
        req: Request,
        ext: &mut Extensions,
        next: Next<'_>,
    ) -> crate::Result<Response> {
        let mut current_retry_times = 0;
        let client = next.get_inner_client_owned();
        let origin_req = match req.try_clone() {
            Some(req) => req,
            None => return next.run(req, ext).await,
        };

        let mut response = next.run(req, ext).await;
        loop {
            if let Ok(response) = response {
                return Ok(response);
            } else {
                let error = response.unwrap_err();
                match error {
                    crate::Error::TooManyRedirect(_) => return Err(error),
                    _ => (),
                };
                current_retry_times += 1;
                match self.0.should_retry(current_retry_times) {
                    RetryDecision::Retry { execute_after } => {
                        let should_wait_for = execute_after - Utc::now();
                        if !should_wait_for.is_zero() {
                            #[cfg(not(target_arch = "wasm32"))]
                            tokio::time::sleep(
                                should_wait_for.to_std().map_err(crate::Error::from)?,
                            )
                            .await;
                            #[cfg(target_arch = "wasm32")]
                            wasm_timer::Delay::new(
                                should_wait_for.to_std().map_err(crate::Error::from)?,
                            )
                            .await
                            .expect("failed sleeping");
                        }
                        if let Some(req) = origin_req.try_clone() {
                            response = client.execute(req).await.map_err(crate::Error::from);
                        } else {
                            return Err(error);
                        }
                    }
                    RetryDecision::DoNotRetry => return Err(error),
                }
            }
        }
    }
}
