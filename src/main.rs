mod auth;
mod handlers;
mod models;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "htmx_builder=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Build our application with routes
    let protected_routes = Router::new()
        .route("/", get(handlers::index))
        // Tree-based routes with paths
        .route("/rule/node/:path/add-condition-form", get(handlers::new_condition_form))
        .route("/rule/node/:path/add-condition", post(handlers::add_condition))
        .route("/rule/node/:path/add-group", post(handlers::add_group))
        .route("/rule/node/:path/operator", post(handlers::update_operator))
        .route("/rule/node/:path", axum::routing::delete(handlers::delete_node))
        // Dependent dropdown routes
        .route(
            "/rule/conditions/operators",
            get(handlers::get_operators_for_field),
        )
        .route(
            "/rule/conditions/value-input",
            get(handlers::get_value_input_for_field),
        )
        .route(
            "/rule/conditions/operators-and-right",
            get(handlers::get_operators_and_right_hint),
        )
        .route(
            "/rule/conditions/operators-for-value",
            get(handlers::get_operators_for_value),
        )
        .route("/rule/validate", post(handlers::validate_rule))
        .layer(middleware::from_fn(auth::auth_middleware));

    let public_routes = Router::new()
        .route("/login", get(handlers::login_page).post(handlers::do_login))
        .layer(middleware::from_fn(auth::public_only_middleware));

    let app = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .route("/logout", post(handlers::logout))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
