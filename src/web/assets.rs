use axum::{
    Router,
    body::Body,
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use http::{HeaderValue, uri::Uri};
use tower_http::services::{Redirect, ServeDir};

use crate::{
    common::{SETTINGS, constants},
    server::utils::state::AppState,
};

async fn static_header_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("Cache-Control", HeaderValue::from_static("public, no-transform"));
    response
        .headers_mut()
        .insert("X-authentik-version", HeaderValue::from_static(constants::VERSION));
    response
        .headers_mut()
        .insert("Vary", HeaderValue::from_static("X-authentik-version, Etag"));
    response
}

pub(super) fn make_router() -> Router<AppState> {
    let web_dist_assets = ServeDir::new("./web/dist/assets");

    // Root file paths, from which they should be accessed
    let mut router = Router::new()
        .nest_service("/static/authentik/", ServeDir::new("./web/authentik"))
        .nest_service("/static/dist/", ServeDir::new("./web/dist"));

    // Also serve assets folder in specific interfaces since fonts in patternfly are imported with
    // a relative path
    router = router
        .nest_service("/if/flow/:flow_slug/assets", web_dist_assets.clone())
        .nest_service("/if/admin/assets", web_dist_assets.clone())
        .nest_service("/if/user/assets", web_dist_assets.clone())
        .nest_service("/if/rac/:app_slug/assets", web_dist_assets);

    // Media files, if backend is file
    if SETTINGS.storage.media.backend == "file" {
        router = router.nest_service("/media/", ServeDir::new(SETTINGS.storage.media.file.path.clone()));
    }

    router = router
        .nest_service("/if/help/", ServeDir::new("./website/help"))
        .nest_service("/help", Redirect::<Body>::permanent(Uri::from_static("/if/help/")));

    // Static misc files
    router = router
        .route(
            "/robots.txt",
            get(|| async {
                Response::builder()
                    .status(200)
                    .header("Content-Type", "text/plain")
                    .body::<Body>(include_str!("../../web/robots.txt").into())
                    .unwrap()
            }),
        )
        .route(
            "/.well-known/security.txt",
            get(|| async {
                Response::builder()
                    .status(200)
                    .header("Content-Type", "text/plain")
                    .body::<Body>(include_str!("../../web/security.txt").into())
                    .unwrap()
            }),
        );

    router.layer(middleware::from_fn(static_header_middleware))
}
