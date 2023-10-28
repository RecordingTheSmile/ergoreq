use async_trait::async_trait;
use ergoreq::middleware::middleware::{Middleware, Next};
use ergoreq::wrappers::client_wrapper::ErgoClient;
use http::Extensions;
use reqwest::redirect::Policy;
use reqwest::{Request, Response};

struct ExampleGlobalMiddleware;

#[async_trait]
impl Middleware for ExampleGlobalMiddleware {
    async fn handle(
        &self,
        req: Request,
        ext: &mut Extensions,
        next: Next<'_>,
    ) -> ergoreq::Result<Response> {
        println!("This is global middleware, Will request: {}", req.url());
        let response = next.run(req, ext).await?;
        println!(
            "This is global middleware, Request success with status {}",
            response.status()
        );
        Ok(response)
    }
}

struct ExamplePerRequestMiddleware;

#[async_trait]
impl Middleware for ExamplePerRequestMiddleware {
    async fn handle(
        &self,
        req: Request,
        ext: &mut Extensions,
        next: Next<'_>,
    ) -> ergoreq::Result<Response> {
        println!(
            "This is per-request middleware, Will request: {}",
            req.url()
        );
        let response = next.run(req, ext).await?;
        println!(
            "This is per-request middleware, Request success with status {}",
            response.status()
        );
        Ok(response)
    }
}

#[tokio::main]
async fn main() {
    let client = reqwest::ClientBuilder::new()
        .redirect(Policy::none()) // Remember this!
        .user_agent("ergoreq/1.0")
        .build()
        .unwrap();

    let client = ErgoClient::new(client).with_middleware(ExampleGlobalMiddleware); // Add global middleware here

    client
        .get("https://crates.io")
        .with_middleware(ExamplePerRequestMiddleware)
        .send()
        .await
        .unwrap();

    /* Global middleware should execute before PerRequest middleware
      Like this:

      GlobalRequestMiddleware: Process Request
                      |
                      |
                      ↓
      PerRequestMiddleware: Process Request
                      |
                      |
                      ↓
      Send Request and Wait for Response
                      |
                      |
                      ↓
      PerRequestMiddleware: Process Response
                      |
                      |
                      ↓
      GlobalRequestMiddleware: Process Response
    */
}
