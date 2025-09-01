use std::{
    future::{Ready, ready},
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};

use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::Error,
    http::header::{HeaderName, HeaderValue},
    web::Data,
};

use crate::{core::configuration::Configuration, net::DEFAULT_CONF_REMOTE_PATH};

const DEFAULT_HEADER_KEY: &str = "x-unknown-header";
const DEFAULT_HEADER_VALUE: &str = "unknown-value";

pub struct HeadersMiddleware;

pub struct HeadersMiddlewareService<S> {
    service: S,
}

impl<S, T> Transform<S, ServiceRequest> for HeadersMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<T>, Error = Error> + 'static,
    S::Future: 'static,
    T: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = HeadersMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(HeadersMiddlewareService { service }))
    }
}

impl<S, B> Service<ServiceRequest> for HeadersMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let conf = req.app_data::<Data<Configuration>>().cloned();

        let remote_path = req.path().to_owned();

        let static_remote_path = conf
            .as_ref()
            .and_then(|f| f.hubs.clone())
            .and_then(|f| f._static.clone())
            .and_then(|f| f.remote_path.clone())
            .unwrap_or_else(|| "/".to_string());

        let conf_remote_path = conf
            .as_ref()
            .and_then(|f| f.hubs.clone())
            .and_then(|f| f.configuration.clone())
            .and_then(|f| f.remote_path.clone())
            .unwrap_or_else(|| DEFAULT_CONF_REMOTE_PATH.to_string());

        let global_headers = conf
            .as_ref()
            .and_then(|f| f.global.clone())
            .and_then(|f| f.headers.clone());

        let additionals_headers = match remote_path {
            // we check it first to prevent matching root path before matching /config
            f if f == conf_remote_path => conf
                .as_ref()
                .and_then(|f| f.hubs.clone())
                .and_then(|f| f.configuration.clone())
                .and_then(|f| f.headers.clone()),
            s if s.starts_with(&static_remote_path) => conf
                .as_ref()
                .and_then(|f| f.hubs.clone())
                .and_then(|f| f._static.clone())
                .and_then(|f| f.headers.clone()),
            _ => None,
        };

        // Merge headers from both static and configuration if applicable
        let headers = match (global_headers, additionals_headers) {
            (Some(mut h1), Some(h2)) => {
                h1.extend(h2);
                Some(h1)
            }
            (Some(h1), None) => Some(h1),
            (None, Some(h2)) => Some(h2),
            (None, None) => None,
        };

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            for (key, value) in headers.unwrap_or_default() {
                if key.trim().is_empty() || value.trim().is_empty() {
                    println!("Skipping invalid header: '{key}' -> '{value}'");
                    continue;
                }
                res.headers_mut().insert(
                    HeaderName::from_str(&key)
                        .unwrap_or(HeaderName::from_static(DEFAULT_HEADER_KEY)),
                    HeaderValue::from_str(&value)
                        .unwrap_or(HeaderValue::from_static(DEFAULT_HEADER_VALUE)),
                );
            }
            Ok(res.map_into_boxed_body())
        })
    }
}
