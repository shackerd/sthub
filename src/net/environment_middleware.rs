use actix_web::{
    Error, HttpResponse,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web,
};
use std::future::{Future, Ready, ready};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::core::configuration::Configuration;
use crate::environment;

pub struct EnvironmentMiddleware;

use actix_web::body::MessageBody;

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
        if path == "/env" {
            let conf = req.app_data::<web::Data<Configuration>>().cloned();
            let (req, _pl) = req.into_parts();
            Box::pin(async move {
                if let Some(conf) = conf {
                    let prefix = conf
                        .hubs
                        .clone()
                        .unwrap()
                        .configuration
                        .unwrap()
                        .providers
                        .unwrap()
                        .env
                        .unwrap()
                        .prefix
                        .unwrap();

                    let environment =
                        environment::JsonEnvironmentVarsTree::new(&format!("{}__", &prefix));
                    let tree = environment.build();
                    let resp = HttpResponse::Ok().json(tree);
                    Ok(ServiceResponse::new(req, resp.map_into_boxed_body()))
                } else {
                    Ok(ServiceResponse::new(
                        req,
                        HttpResponse::InternalServerError()
                            .finish()
                            .map_into_boxed_body(),
                    ))
                }
            })
        } else {
            let fut = self.service.call(req);
            Box::pin(async move { fut.await.map(|res| res.map_into_boxed_body()) })
        }
    }
}
