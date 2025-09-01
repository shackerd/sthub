use crate::core::configuration::Configuration;
use crate::environment;
use actix_web::body::MessageBody;
use actix_web::{
    Error, HttpResponse,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web,
};
use std::future::{Future, Ready, ready};
use std::pin::Pin;
use std::task::{Context, Poll};

const DEFAULT_PREFIX: &str = "STHUB__";
const DEFAULT_REMOTE_PATH: &str = "/env";

pub struct EnvironmentMiddleware;

impl<S, B> Transform<S, ServiceRequest> for EnvironmentMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = EnvConfigMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(EnvConfigMiddlewareService { service }))
    }
}

pub struct EnvConfigMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for EnvConfigMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_owned();

        let conf = req.app_data::<web::Data<Configuration>>().cloned();

        let conf_remote_path = conf
            .as_ref()
            .and_then(|f| f.hubs.clone())
            .and_then(|f| f.configuration)
            .and_then(|f| f.remote_path)
            .unwrap_or(DEFAULT_REMOTE_PATH.to_string());

        let prefix = conf
            .and_then(|f| f.hubs.clone())
            .and_then(|f| f.configuration)
            .and_then(|f| f.providers)
            .and_then(|f| f.env)
            .and_then(|f| f.prefix)
            .unwrap_or(DEFAULT_PREFIX.to_string());

        if path == conf_remote_path {
            let (req, _pl) = req.into_parts();
            Box::pin(async move {
                let environment =
                    environment::JsonEnvironmentVarsTree::new(&format!("{}__", &prefix));
                let tree = environment.build();
                let resp = HttpResponse::Ok().json(tree);
                Ok(ServiceResponse::new(req, resp.map_into_boxed_body()))
            })
        } else {
            let fut = self.service.call(req);
            Box::pin(async move { fut.await.map(|res| res.map_into_boxed_body()) })
        }
    }
}
