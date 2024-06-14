use super::middleware::Middleware;
use crate::middleware::middleware::Next;
use async_trait::async_trait;
use http::Extensions;
use reqwest::{Request, Response};
use retry_policies::{RetryDecision, RetryPolicy};
use std::{sync::Arc, time::SystemTime};
use tracing::instrument;

pub(crate) struct AutoRetryMiddleware(Arc<dyn RetryPolicy + Send + Sync + 'static>);

impl AutoRetryMiddleware {
    pub fn new(policy: Arc<dyn RetryPolicy + Send + Sync + 'static>) -> Self {
        Self(policy)
    }
}

#[async_trait]
impl Middleware for AutoRetryMiddleware {
    #[instrument(skip(self, ext, next))]
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
        let request_start_time = SystemTime::now();
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
                match self.0.should_retry(request_start_time, current_retry_times) {
                    RetryDecision::Retry { execute_after } => {
                        let should_wait_for = match execute_after.duration_since(SystemTime::now())
                        {
                            Ok(duration) => duration,
                            Err(_) => std::time::Duration::from_secs(0),
                        };
                        if !should_wait_for.is_zero() {
                            #[cfg(not(target_arch = "wasm32"))]
                            tokio::time::sleep(should_wait_for).await;
                            #[cfg(target_arch = "wasm32")]
                            wasm_timer::Delay::new(should_wait_for)
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
