use poker_server::server::router;
use poker_server::store::InMem;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

pub struct TestServer {
    pub addr: SocketAddr,
}

impl TestServer {
    pub async fn new() -> Self {
        // Create in-memory store
        let inmem = InMem::new_demo();
        let shared_store = inmem.into_shared();

        // Build the same app as in main.rs
        let app = router(shared_store.clone())
            .layer(tower_http::trace::TraceLayer::new_for_http())
            .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any));

        // Bind to random port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Start server in background task
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        TestServer { addr }
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

// Client helper functions
pub async fn make_client() -> reqwest::Client {
    reqwest::Client::new()
}
