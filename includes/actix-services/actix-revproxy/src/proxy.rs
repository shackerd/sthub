//! Actix-Web Proxy Utilities
use std::{collections::HashMap, path::PathBuf};

use actix_web::{
    HttpMessage, HttpResponse,
    web::{Bytes, Query},
};
use awc::{
    Client, ClientRequest,
    error::{HeaderValue, PayloadError},
    http::{
        Uri,
        header::{self, HeaderMap, HeaderName},
        uri::Scheme,
    },
};

use crate::error::{Error, UriError};

const HOP_HEADERS: [HeaderName; 9] = [
    header::CONNECTION,
    header::TE,
    header::TRAILER,
    header::PROXY_AUTHENTICATE,
    header::PROXY_AUTHORIZATION,
    header::TRANSFER_ENCODING,
    header::UPGRADE,
    header::HeaderName::from_static("keep-alive"),
    header::HeaderName::from_static("proxy-connection"),
];

/// Trait for Converting [`actix_web::HttpRequest`] to [`awc::ClientRequest`]
pub trait ClientReq {
    type Error;

    fn client_req(&self, client: &Client, url: Uri) -> Result<ClientRequest, Self::Error>;
}

/// Trait for Converting [`awc::ClientResponse`] to [`actix_web::HttpResponse`]
pub trait ServerRes {
    type Error;

    fn server_response(self) -> Result<HttpResponse, Self::Error>;
}

impl ClientReq for actix_web::HttpRequest {
    type Error = Error;

    fn client_req(&self, client: &Client, url: Uri) -> Result<ClientRequest, Self::Error> {
        let mut request = client.request_from(url, self.head());
        remove_connection_headers(request.headers_mut())?;
        remove_hop_headers(request.headers_mut());
        Ok(request.camel_case())
    }
}

impl<T> ServerRes for awc::ClientResponse<T>
where
    T: futures_core::Stream<Item = Result<Bytes, PayloadError>> + 'static,
{
    type Error = Error;

    fn server_response(mut self) -> Result<HttpResponse, Self::Error> {
        let payload = self.take_payload();

        let mut builder = actix_web::HttpResponseBuilder::new(self.status());
        for header in self.headers() {
            builder.append_header(header);
        }

        let mut http_res = match self.headers().get(header::CONTENT_LENGTH) {
            Some(value) if value.as_bytes() == b"0" => builder.body(()),
            _ => builder.streaming(payload),
        };

        remove_connection_headers(http_res.headers_mut())?;
        remove_hop_headers(http_res.headers_mut());
        Ok(http_res)
    }
}

type QueryMap = Query<HashMap<String, String>>;

#[inline]
fn get_query(uri: &Uri) -> Result<QueryMap, UriError> {
    Ok(QueryMap::from_query(uri.query().unwrap_or(""))?)
}

/// Combine Proxy URI with Specified Target URI
pub fn combine_uri(proxy: &Uri, target: &Uri) -> Result<Uri, UriError> {
    let authority = proxy.authority().ok_or(UriError::MissingAuthority)?;
    let path = PathBuf::from(proxy.path())
        .join(target.path())
        .to_str()
        .ok_or(UriError::InvalidUriPath)?
        .to_owned();

    let mut query = get_query(proxy)?;
    query.extend(get_query(target)?.into_inner());
    let query = serde_urlencoded::to_string(query.into_inner())?;

    Ok(Uri::builder()
        .scheme(proxy.scheme().cloned().unwrap_or(Scheme::HTTP))
        .authority(authority.clone())
        .path_and_query(format!("{path}?{query}").trim_end_matches('?'))
        .build()?)
}

/// Remove all "Hop by Hop" headers from request/response
#[inline]
pub fn remove_hop_headers(headers: &mut HeaderMap) {
    for header in HOP_HEADERS {
        headers.remove(header);
    }
}

/// Remove `Connection` related headers from request/response
#[inline]
pub fn remove_connection_headers(headers: &mut HeaderMap) -> Result<(), Error> {
    let Some(value) = headers.get(header::CONNECTION) else {
        return Ok(());
    };
    value
        .clone()
        .to_str()?
        .split(',')
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .for_each(|v| {
            headers.remove(v);
        });
    Ok(())
}

/// Update/Insert forward header with connection information
///
/// # Examples
///
/// ```
/// use actix_revproxy::proxy::update_forwarded;
/// use awc::{ClientRequest, http::header::X_FORWARDED_FOR};
///
/// fn update(req: &mut ClientRequest) {
///   update_forwarded(req.headers_mut(), X_FORWARDED_FOR, "1.2.3.4".to_owned());
/// }
/// ```
pub fn update_forwarded(
    headers: &mut HeaderMap,
    header: HeaderName,
    ip: String,
) -> Result<(), Error> {
    let value = match headers.get(&header) {
        None => ip,
        Some(value) => format!("{}, {ip}", value.to_str()?),
    };
    headers.insert(header, HeaderValue::from_str(&value)?);
    Ok(())
}
