use axum::body::Body;
use http::Uri;
use hyper::Request;
use hyper_util::{
    client::legacy::{Client, ResponseFuture, connect::HttpConnector},
    rt::TokioExecutor,
};
use hyperlocal::UnixConnector;

type UnixClient = Client<UnixConnector, Body>;
type HttpClient = Client<HttpConnector, Body>;

#[derive(Clone)]
pub enum BackendClient {
    Http(HttpClient),
    Unix(UnixClient),
}

impl BackendClient {
    pub fn new(backend_uri: &Uri) -> Self {
        match backend_uri.scheme_str() {
            Some("unix") => {
                let client = Client::builder(TokioExecutor::new()).build(UnixConnector);
                Self::Unix(client)
            }
            _ => {
                let client = Client::builder(TokioExecutor::new()).build_http();
                Self::Http(client)
            }
        }
    }

    pub fn request(&self, req: Request<Body>) -> ResponseFuture {
        match self {
            Self::Http(client) => client.request(req),
            Self::Unix(client) => client.request(req),
        }
    }
}
