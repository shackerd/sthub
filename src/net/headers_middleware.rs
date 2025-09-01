use std::{
    future::{Ready, ready},
    pin::Pin,
    task::{Context, Poll},
};

use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::Error,
    http::header::{self, HeaderName, HeaderValue},
    web::Data,
};

use crate::core::configuration::Configuration;

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

        let headers = conf
            .and_then(|f| f.hubs.clone())
            .and_then(|f| f._static)
            .and_then(|f| f.headers);

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            for (key, value) in headers.unwrap_or_default() {
                res.headers_mut().insert(
                    HeaderName::from_lowercase(key.as_bytes())
                        .unwrap_or(HeaderName::from_static("x-unknown-header")),
                    HeaderValue::from_str(&value)
                        .unwrap_or(HeaderValue::from_static("unknown-value")),
                );
            }
            // res.headers_mut().insert(
            //     header::X_CONTENT_TYPE_OPTIONS,
            //     HeaderValue::from_static("nosniff"),
            // );
            // res.headers_mut()
            //     .insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
            // res.headers_mut().insert(
            //     header::STRICT_TRANSPORT_SECURITY,
            //     HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            // );
            // res.headers_mut().insert(
            //     header::CONTENT_SECURITY_POLICY,
            //     HeaderValue::from_static("default-src 'self'"),
            // );

            Ok(res.map_into_boxed_body())
        })
    }
}
