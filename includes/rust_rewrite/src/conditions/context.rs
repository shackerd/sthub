//! Variable expansion and assignment contexts used in
//! [`Condition`](super::Condition) evalulation.
//!
//! Designed as a subset of [official](https://httpd.apache.org/docs/current/mod/mod_rewrite.html#rewritecond)
//! `RewriteCond` back-references.

use std::{collections::HashMap, fmt::Debug, io, net::ToSocketAddrs};

use once_cell::sync::Lazy;
use regex_automata::meta::Regex;

static MATCHER: Lazy<Regex> = Lazy::new(|| Regex::new(r"%\{\w+\}").unwrap());

macro_rules! get {
    ($key:expr) => {
        Some($key.as_ref().map(|s| s.as_str()).unwrap_or(""))
    };
}

macro_rules! setter {
    ($key:ident, $ref:ident) => {
        #[doc = concat!("Assign value for `", stringify!($ref), "` variable")]
        pub fn $key<S: Into<String>>(mut self, $key: S) -> Self {
            self.$key = Some($key.into());
            self
        }
    };
}

/// Abstraction for `RewriteCond` variable providers
///
/// Supply objects implementing this trait to [`EngineCtx`]
/// and pass it into [`Engine::rewrite_ctx`](crate::Engine::rewrite_ctx)
/// in order to pass variables to [`Condition`](crate::Condition)
/// rules.
pub trait ContextProvider {
    fn fill(&mut self, key: &str) -> Option<&str>;
}

/// Global Context used for variable replacement in
/// [`Condition`](super::Condition) expressions.
#[derive(Default)]
pub struct EngineCtx<'a>(Vec<Box<dyn ContextProvider + 'a>>);

impl<'a> EngineCtx<'a> {
    /// Assign new sub-context to the complete [`EngineCtx`]
    pub fn push_ctx(&mut self, ctx: impl ContextProvider + 'a) -> &mut Self {
        self.0.push(Box::new(ctx));
        self
    }

    /// Assign new sub-context when building [`EngineCtx`]
    pub fn with_ctx(mut self, ctx: impl ContextProvider + 'a) -> Self {
        self.push_ctx(ctx);
        self
    }

    /// Add [`EnvCtx`] when building [`EngineCtx`]
    pub fn with_env(self) -> Self {
        self.with_ctx(EnvCtx::default())
    }

    /// Add [`DateCtx`] when building [`EngineCtx`]
    pub fn with_time(self) -> Self {
        self.with_ctx(DateCtx::new())
    }

    /// Return the equivalent value associated with the specified
    /// variable expression.
    #[inline]
    pub fn fill(&mut self, expr: &str) -> &str {
        self.0
            .iter_mut()
            .find_map(|ctx| ctx.fill(expr))
            .unwrap_or("")
    }

    /// Replace all variables within expression with data
    /// specified within with the [`EngineCtx`] and return
    /// the updated string.
    pub fn replace_all(&mut self, expr: &str) -> String {
        MATCHER
            .find_iter(expr)
            .map(|c| expr[c.range()].to_owned())
            .fold(expr.to_owned(), |acc, key| {
                let attr = key.trim_matches(|c| ['%', '{', '}'].contains(&c));
                acc.replace(&key, self.fill(attr))
            })
    }
}

/// Environment Variable Context.
///
/// Provides variables and references associated with `ENV:` prefix.
#[derive(Clone, Debug, Default)]
pub struct EnvCtx(HashMap<String, String>);

impl ContextProvider for EnvCtx {
    fn fill(&mut self, key: &str) -> Option<&str> {
        let (prefix, env) = key.split_once(':')?;
        if self.0.contains_key(key) {
            return self.0.get(key).map(|v| v.as_str());
        }
        if prefix.to_lowercase() != "env" {
            return None;
        }
        let val = std::env::var(env).ok()?;
        self.0.insert(key.to_string(), val);
        self.0.get(key).map(|v| v.as_str())
    }
}

/// All variables and references associated with `TIME_` prefix.
#[derive(Clone, Debug)]
pub struct DateCtx {
    time_year: String,
    time_month: String,
    time_day: String,
    time_hour: String,
    time_min: String,
    time_sec: String,
    time_wday: String,
    time: String,
}

impl DateCtx {
    pub fn new() -> Self {
        let date = chrono::Local::now();
        Self {
            time_year: date.format("%Y").to_string(),
            time_month: date.format("%m").to_string(),
            time_day: date.format("%d").to_string(),
            time_hour: date.format("%H").to_string(),
            time_min: date.format("%M").to_string(),
            time_sec: date.format("%S").to_string(),
            time_wday: date.format("%A").to_string(),
            time: date.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

impl Default for DateCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextProvider for DateCtx {
    fn fill(&mut self, key: &str) -> Option<&str> {
        match key {
            "TIME_YEAR" => Some(self.time_year.as_str()),
            "TIME_MONTH" => Some(self.time_month.as_str()),
            "TIME_DAY" => Some(self.time_day.as_str()),
            "TIME_HOUR" => Some(self.time_hour.as_str()),
            "TIME_MIN" => Some(self.time_min.as_str()),
            "TIME_SEC" => Some(self.time_sec.as_str()),
            "TIME_WDAY" => Some(self.time_wday.as_str()),
            "TIME" => Some(self.time.as_str()),
            _ => None,
        }
    }
}

/// All variables and references associated with `SERVER_` prefix
/// and other server attributes.
#[derive(Clone, Debug, Default)]
pub struct ServerCtx {
    document_root: Option<String>,
    server_addr: Option<String>,
    server_admin: Option<String>,
    server_name: Option<String>,
    server_port: Option<String>,
    server_protocol: Option<String>,
    server_software: Option<String>,
}

impl ServerCtx {
    setter!(document_root, DOCUMENT_ROOT);
    setter!(server_admin, SERVER_ADMIN);
    setter!(server_name, SERVER_NAME);
    setter!(server_protocol, SERVER_PROTOCOL);
    setter!(server_software, SERVER_SOFTWARE);

    /// Assign value for `SERVER_ADDR`, and `SERVER_PORT` variables.
    pub fn server_addr<A: ToSocketAddrs>(mut self, server_addr: A) -> io::Result<Self> {
        let addr = server_addr
            .to_socket_addrs()?
            .next()
            .expect("missing socket address");
        self.server_addr = Some(addr.to_string());
        self.server_name = Some(self.server_name.unwrap_or_else(|| addr.ip().to_string()));
        self.server_port = Some(addr.port().to_string());
        Ok(self)
    }

    /// Assign value for `SERVER_ADDR`, and `SERVER_PORT` variables if address is Some.
    pub fn maybe_server_addr<A: ToSocketAddrs>(self, server_addr: Option<A>) -> io::Result<Self> {
        match server_addr {
            Some(addr) => self.server_addr(addr),
            None => Ok(self),
        }
    }
}

impl ContextProvider for ServerCtx {
    fn fill(&mut self, key: &str) -> Option<&str> {
        match key {
            "DOCUMENT_ROOT" => get!(self.document_root),
            "SERVER_ADMIN" => get!(self.server_admin),
            "SERVER_ADDR" => get!(self.server_addr),
            "SERVER_NAME" => get!(self.server_name),
            "SERVER_PORT" => get!(self.server_port),
            "SERVER_PROTOCOL" => get!(self.server_protocol),
            "SERVER_SOFTWARE" => get!(self.server_software),
            _ => None,
        }
    }
}

/// All variables and references associated with `REMOTE_` prefix
/// and other request variables.
#[derive(Clone, Debug, Default)]
pub struct RequestCtx {
    auth_type: Option<String>,
    ipv6: Option<String>,
    path_info: Option<String>,
    query_string: Option<String>,
    remote_addr: Option<String>,
    remote_host: Option<String>,
    remote_port: Option<String>,
    request_method: Option<String>,
    request_uri: Option<String>,
}

impl RequestCtx {
    setter!(auth_type, AUTH_TYPE);
    setter!(ipv6, IPV6);
    setter!(path_info, PATH_INFO);
    setter!(query_string, QUERY_STRING);
    setter!(request_method, REQUEST_METHOD);
    setter!(request_uri, REQUEST_URI);

    /// Assign value for `REMOTE_ADDR`, `REMOTE_HOST`, and `REMOTE_PORT` variables.
    pub fn remote_addr<A: ToSocketAddrs>(mut self, remote_addr: A) -> io::Result<Self> {
        let addr = remote_addr
            .to_socket_addrs()?
            .next()
            .expect("missing socket address");
        self.remote_addr = Some(addr.to_string());
        self.remote_host = Some(addr.ip().to_string());
        self.remote_port = Some(addr.port().to_string());
        Ok(self)
    }

    /// Assign value for `REMOTE_ADDR`, `REMOTE_HOST`, and `REMOTE_PORT`
    /// variables if address is Some.
    pub fn maybe_remote_addr<A: ToSocketAddrs>(self, remote_addr: Option<A>) -> io::Result<Self> {
        match remote_addr {
            Some(addr) => self.remote_addr(addr),
            None => Ok(self),
        }
    }
}

impl ContextProvider for RequestCtx {
    fn fill(&mut self, key: &str) -> Option<&str> {
        match key {
            "AUTH_TYPE" => get!(self.auth_type),
            "IPV6" => get!(self.ipv6),
            "PATH_INFO" => get!(self.path_info),
            "QUERY_STRING" => get!(self.query_string),
            "REMOTE_ADDR" => get!(self.remote_addr),
            "REMOTE_HOST" => get!(self.remote_host),
            "REMOTE_PORT" => get!(self.remote_port),
            "REQUEST_METHOD" => get!(self.request_method),
            "REQUEST_URI" => get!(self.request_uri),
            _ => None,
        }
    }
}
