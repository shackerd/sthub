//! HTTP Basic Auth [`Authenticator`] implementations

use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

use actix_web::{
    Error, FromRequest, HttpRequest, HttpResponse,
    http::header::{self, HeaderMap},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use md5::Digest;

mod cache;

use crate::Authenticator;

/// Re-export crypt3 crypt and Hash
pub use crypt3_rs::{Hash, crypt};

/// Basic Authentication [`Authenticator`] builder.
///
/// # Example
///
/// ```
/// use actix_web::App;
/// use actix_authn::{Authn, basic::{Basic, crypt}};
///
/// /// passwords should be generated outside of HttpServer::new
/// /// or use [`Basic::passwd`] or [`Basic::htpasswd`].
/// let passwd = crypt::bcrypt::hash("password").unwrap();
///
/// // basic authorization
/// let app = App::new()
///     .wrap(Authn::new(Basic::default().auth("admin", passwd).build()));
///
/// // basic authorization w/ cookie based session
/// // (this requires https://crates.io/crates/actix-session as well)
/// let passwd = crypt::bcrypt::hash("admin").unwrap();
/// let key = actix_web::cookie::Key::generate();
/// let store = actix_session::storage::CookieSessionStore::default();
/// let app = App::new()
///     .wrap(actix_session::SessionMiddleware::new(store, key))
///     .wrap(Authn::new(Basic::default().auth("admin", passwd).build_session()));
/// ```
#[derive(Default)]
pub struct Basic {
    realm: Option<String>,
    auth: HashMap<String, Hash>,
    cache: cache::AuthCache<bool>,
}

/// Basic Auth [`Authenticator`] implementation.
#[cfg(feature = "basic_session")]
#[derive(Clone)]
pub struct BasicAuth(Arc<Mutex<Basic>>);

/// Basic Auth with Cookie Session [`Authenticator`] implementation.
#[derive(Clone)]
pub struct BasicAuthSession(Arc<Mutex<Basic>>);

#[inline]
fn parse_authorization<'a>(headers: &'a HeaderMap, prefix: &str) -> Option<&'a str> {
    headers
        .get(header::AUTHORIZATION)
        .map(|v| v.to_str().unwrap_or_default())
        .unwrap_or_default()
        .split_once(prefix)
        .map(|(_, auth)| auth.trim())
}

impl Basic {
    /// Assign a realm to the basic authorization.
    pub fn with_realm(mut self, realm: Option<&str>) -> Self {
        self.realm = realm.map(|s| s.to_owned());
        self
    }

    /// Configure authorization cache-size.
    ///
    /// The auth-cache reduces intensive password hashing during
    /// verification by preserving outputs for specific basic-auth
    /// checksums.
    pub fn cache_size(mut self, cache_size: usize) -> Self {
        self.cache.size = cache_size;
        self
    }

    /// Supply pre-hashed [`enum@Hash`] with username as an allowed credential.
    pub fn auth(mut self, user: &str, hash: Hash) -> Self {
        self.auth.insert(user.to_owned(), hash);
        self
    }

    /// Pass a pre-hashed htpasswd string as an allowed credential.
    ///
    /// # Example
    ///
    /// ```
    /// use actix_authn::basic::Basic;
    ///
    /// Basic::default()
    ///     .passwd("admin:$5$zf2X2LFe6AL0ZBWn$y9Ox4HNZwHtZM85cNW8iaUgE7EiuNF01vNveiXnwk68");
    /// ```
    pub fn passwd(self, passwd: &str) -> Self {
        let (user, secret) = passwd
            .trim()
            .split_once(':')
            .expect("invalid htpasswd contents");
        let passwd = Hash::try_from(secret).expect("invalid passwd");
        self.auth(user, passwd)
    }

    /// Pass a pre-hashed htpasswd file as an allowed credential.
    pub fn htpasswd<P: AsRef<Path>>(self, path: P) -> Self {
        let htpasswd = std::fs::read_to_string(path).expect("failed to read htpasswd");
        self.passwd(&htpasswd)
    }

    /// Verify the supplied credentials.
    pub fn verify(&self, user: &str, secret: &str) -> bool {
        self.auth
            .get(user)
            .map(|hash| hash.verify(secret))
            .unwrap_or_default()
    }

    /// Verify `Authorzation: Basic <basic_base64>` header value.
    ///
    /// This method makes use of the credential-cache which hashes
    /// the given base64 string to store the result.
    pub fn verify_basic(&mut self, basic_base64: String) -> bool {
        let key = md5::Md5::digest(&basic_base64).to_vec();
        if let Some(entry) = self.cache.get(&key) {
            return *entry;
        }
        let Ok(auth) = STANDARD.decode(&basic_base64) else {
            return false;
        };
        let Ok(auth) = std::str::from_utf8(&auth) else {
            return false;
        };
        let Some((user, secret)) = auth.split_once(':') else {
            return false;
        };
        let verify = self.verify(user, secret);
        self.cache.insert(key, verify);
        verify
    }

    pub(crate) fn prompt(&self) -> HttpResponse {
        let realm = self.realm.as_deref().unwrap_or_default();
        let auth = format!("Basic realm={realm:?}, charset={:?}", "UTF-8");
        HttpResponse::Unauthorized()
            .insert_header((header::WWW_AUTHENTICATE, auth))
            .finish()
    }

    /// Build into [`BasicAuth`] authorizor instance.
    pub fn build(self) -> BasicAuth {
        BasicAuth(Arc::new(Mutex::new(self)))
    }

    /// Build into [`BasicAuthSession`] authorizor instance.
    #[cfg(feature = "basic_session")]
    pub fn build_session(self) -> BasicAuthSession {
        BasicAuthSession(Arc::new(Mutex::new(self)))
    }
}

impl Authenticator for BasicAuth {
    async fn authorize(&self, req: &HttpRequest) -> Result<bool, Error> {
        let Some(basic) = parse_authorization(req.headers(), "Basic ") else {
            return Ok(false);
        };
        let this = Arc::clone(&self.0);
        let basic = basic.to_owned();
        Ok(actix_web::rt::task::spawn_blocking(move || {
            this.lock().expect("failed to unlock").verify_basic(basic)
        })
        .await
        .expect("failed to spawn actix thread"))
    }

    #[inline]
    async fn prompt(&self, _req: &HttpRequest) -> Result<HttpResponse, Error> {
        Ok(self.0.lock().expect("failed to retrieve lock").prompt())
    }
}

#[cfg(feature = "basic_session")]
impl Authenticator for BasicAuthSession {
    async fn authorize(&self, req: &HttpRequest) -> Result<bool, Error> {
        let session = actix_session::Session::extract(req).await?;
        if session.get::<bool>("logged_in")?.unwrap_or_default() {
            return Ok(true);
        }

        let Some(basic) = parse_authorization(req.headers(), "Basic ") else {
            return Ok(false);
        };
        let this = Arc::clone(&self.0);
        let basic = basic.to_owned();
        let logged_in = actix_web::rt::task::spawn_blocking(move || {
            this.lock().expect("failed to unlock").verify_basic(basic)
        })
        .await
        .expect("failed to spawn actix thread");

        if logged_in {
            session.insert("logged_in", true)?;
        }
        Ok(logged_in)
    }

    #[inline]
    async fn prompt(&self, _req: &HttpRequest) -> Result<HttpResponse, Error> {
        Ok(self.0.lock().expect("failed to retrieve lock").prompt())
    }
}
