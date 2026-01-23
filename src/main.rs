// src/main.rs
use axum::extract::Extension;
use tower_http::cors::{Any, CorsLayer};

use poker_server::server::router;
use poker_server::store::InMem;

mod middleware;
use middleware::logging_middleware;

#[tokio::main]
async fn main() {
    // basic logging
    tracing_subscriber::fmt::init();

    // create in-memory store and convert to shared store
    let inmem = InMem::new_demo();
    let shared_store = inmem.into_shared();

    // build router (defined in server::router) and attach layers
    let app = router(shared_store.clone())
        .layer(Extension(logging_middleware))
        // make the store available to handlers via axum's Extension mechanism
        .layer(Extension(shared_store))
        // simple dev CORS — allow everything (change for production)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // bind and serve (use axum::Server directly)

    let addr = "0.0.0.0:3001";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on http://{addr}");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
