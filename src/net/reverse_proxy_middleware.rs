use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web::Data,
};

pub struct ReverseProxyMiddleware;
pub struct ReverseProxyMiddlewareService<S> {
    service: S,
}

impl<S, B> Transform<S, ServiceRequest> for ReverseProxyMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type Transform = ReverseProxyMiddlewareService<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(ReverseProxyMiddlewareService { service }))
    }
}

impl<S, B> Service<ServiceRequest> for ReverseProxyMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let remote_path = req.path().to_owned();

        let conf = req
            .app_data::<Data<crate::core::configuration::Configuration>>()
            .cloned();

        let upstream = conf
            .as_ref()
            .and_then(|f| f.hubs.clone())
            .and_then(|f| f.upstream.clone());

        let matching_upstream_path = upstream.as_ref().and_then(|f| f.remote_path.clone());

        if upstream.is_none() {
            // declare here fut to avoid req moving into async block
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_boxed_body()) });
        }

        let upstream = upstream.unwrap();

        if upstream.target.is_none() || matching_upstream_path.is_none() {
            // declare here fut to avoid req moving into async block
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_boxed_body()) });
        }

        let matching_upstream_path = matching_upstream_path.as_ref().unwrap();

        let segments = remote_path.split("/");
        let first_segment = segments
            .into_iter()
            .nth(1)
            .unwrap_or("")
            .trim_start_matches('/');

        if first_segment != matching_upstream_path.trim_start_matches('/') {
            // declare here fut to avoid req moving into async block
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_boxed_body()) });
        }

        let client = awc::Client::default();

        let (http_req, _pl) = req.into_parts();

        let mut new_url = http_req.uri().to_string();

        // Remove the matching upstream path prefix
        if let Some(stripped) = new_url.strip_prefix(matching_upstream_path) {
            new_url = if stripped.is_empty() {
                "/".to_string()
            } else {
                stripped.to_string()
            };
        }

        if let Some(up) = upstream.target {
            if new_url.starts_with('/') {
                new_url = format!("{up}{new_url}");
            } else {
                new_url = format!("{up}/{new_url}");
            }
        }

        let mut forward_req = client.request_from(new_url, http_req.head());

        if let Some(addr) = http_req.peer_addr() {
            forward_req = forward_req.insert_header((
                actix_web::http::header::FORWARDED,
                format!("for={}", addr.ip()),
            ));
        }

        Box::pin(async move {
            let mut res = forward_req
                .send()
                .await
                .map_err(actix_web::error::ErrorBadGateway)?;

            let mut client_resp = actix_web::HttpResponse::build(res.status());

            for (header_name, header_value) in res.headers() {
                // prevent chunked encoding errors
                if header_name == actix_web::http::header::TRANSFER_ENCODING {
                    continue;
                }
                client_resp.append_header((header_name.clone(), header_value.clone()));
            }

            let body = res.body().limit(10_485_760).await; // Limit to 10MB
            let body = body.map_err(actix_web::error::ErrorBadGateway)?;
            Ok(ServiceResponse::new(
                http_req,
                client_resp.body(body).map_into_boxed_body(),
            ))
        })
    }
}
