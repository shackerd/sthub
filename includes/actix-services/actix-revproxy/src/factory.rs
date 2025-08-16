use std::{fmt::Debug, rc::Rc, str::FromStr};

use actix_service::ServiceFactory;
use actix_web::{
    Error,
    dev::{AppService, HttpServiceFactory, ResourceDef, ServiceRequest, ServiceResponse},
    guard::Guard,
};
use awc::{
    Client,
    http::{Uri, header},
};
use futures_core::future::LocalBoxFuture;

use crate::service::HeaderVec;

use super::service::{ProxyService, ProxyServiceInner};

/// Reverse Proxy service
///
/// `RevProxy` service must be registered with `App::service()` method.
///
/// # Examples
///
/// ```
/// use actix_web::App;
/// use actix_revproxy::RevProxy;
///
/// let app = App::new()
///     .service(RevProxy::new("/", "http://127.0.0.1:8080"));
/// ```
#[derive(Clone)]
pub struct RevProxy {
    mount_path: String,
    guards: Vec<Rc<dyn Guard>>,
    client: Rc<Client>,
    resolve: Uri,
    change_host: bool,
    header_up: HeaderVec,
    header_down: HeaderVec,
}

impl RevProxy {
    /// Creates new `RevProxy` instance for a specified resolution uri
    ///
    /// # Argument Order
    /// The first argument (`mount_path`) is the root URL at which the static files are served.
    /// For example, `/assets` will serve files at `example.com/assets/...`.
    ///
    /// The second argument (`uri`) is the base uri that directs where the proxy
    /// resolves at.
    pub fn new<U: TryInto<Uri>>(mount_path: &str, uri: U) -> Self
    where
        U::Error: Debug,
    {
        Self {
            mount_path: mount_path.to_owned(),
            guards: Vec::new(),
            client: Rc::new(awc::Client::new()),
            resolve: uri.try_into().expect("invalid resolution uri"),
            change_host: false,
            header_up: Vec::new(),
            header_down: Vec::new(),
        }
    }

    /// Adds a routing guard.
    ///
    /// Use this to allow multiple chained services that respond to strictly different
    /// properties of a request. Due to the way routing works, if a guard check returns true and the
    /// request starts being handled by the file service, it will not be able to back-out and try
    /// the next service, you will simply get a 404 (or 405) error response.
    ///
    /// # Examples
    /// ```
    /// use actix_web::{guard::Header, App};
    /// use actix_revproxy::RevProxy;
    ///
    /// App::new().service(
    ///     RevProxy::new("/", "http://127.0.0.1:8080")
    ///         .guard(Header("Host", "example.com"))
    /// );
    /// ```
    pub fn guard<G: Guard + 'static>(mut self, guards: G) -> Self {
        self.guards.push(Rc::new(guards));
        self
    }

    /// Overrides the actix-web-client instance used by the proxy
    ///
    /// Default is [`Client::new()`](awc::Client::new)
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = Rc::new(client);
        self
    }

    /// Configure proxy to change hostname to the upstream host
    ///
    /// Default is return the established hostname of the original request.
    pub fn change_host(mut self) -> Self {
        self.change_host = true;
        self
    }

    /// Append a header to include in the upstream request.
    pub fn upstream_header(mut self, name: &str, value: &str) -> Self {
        let Ok(name) = header::HeaderName::from_str(name) else {
            tracing::warn!("invalid upstream header name {name:?}");
            return self;
        };
        let Ok(value) = header::HeaderValue::from_str(value) else {
            tracing::warn!("invalid upstream header value {name:?}: {value:?}");
            return self;
        };
        self.header_up.push((name, value));
        self
    }

    /// Append a header to include in the downstream response.
    pub fn downstream_header(mut self, name: &str, value: &str) -> Self {
        let Ok(name) = header::HeaderName::from_str(name) else {
            tracing::warn!("invalid downstream header name {name:?}");
            return self;
        };
        let Ok(value) = header::HeaderValue::from_str(value) else {
            tracing::warn!("invalid downstream header value {name:?}: {value:?}");
            return self;
        };
        self.header_down.push((name, value));
        self
    }
}

impl HttpServiceFactory for RevProxy {
    fn register(mut self, config: &mut AppService) {
        let guards = if self.guards.is_empty() {
            None
        } else {
            let guards = std::mem::take(&mut self.guards);
            Some(
                guards
                    .into_iter()
                    .map(|guard| -> Box<dyn Guard> { Box::new(guard) })
                    .collect::<Vec<_>>(),
            )
        };

        let rdef = if config.is_root() {
            ResourceDef::root_prefix(&self.mount_path)
        } else {
            ResourceDef::prefix(&self.mount_path)
        };

        config.register_service(rdef, guards, self, None)
    }
}

impl ServiceFactory<ServiceRequest> for RevProxy {
    type Response = ServiceResponse;
    type Error = Error;
    type Config = ();
    type Service = ProxyService;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Service, Self::InitError>>;

    fn new_service(&self, _: ()) -> Self::Future {
        let inner = ProxyServiceInner {
            client: self.client.clone(),
            resolve: self.resolve.clone(),
            change_host: self.change_host,
            header_up: self.header_up.clone(),
            header_down: self.header_down.clone(),
        };
        Box::pin(async move { Ok(ProxyService(Rc::new(inner))) })
    }
}
