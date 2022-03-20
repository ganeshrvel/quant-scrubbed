use headers::authorization::Credentials;
use headers::Authorization;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tokio_tungstenite::tungstenite::http::{HeaderMap, Method};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RestfulAuthorization {
    pub username: String,
    pub password: String,
}

pub fn wss_request(
    uri: &str,
    authorization: &Option<RestfulAuthorization>,
) -> anyhow::Result<Request> {
    let request = Request::new(());
    let (mut parts, body) = request.into_parts();
    parts.method = Method::GET;
    parts.uri = uri.parse()?;

    let mut headers = HeaderMap::new();

    if let Some(a) = authorization {
        let basic = Authorization::basic(&*a.username, &*a.password);

        headers.insert(AUTHORIZATION, basic.0.encode());
    }

    parts.headers = headers;

    Ok(Request::from_parts(parts, body))
}
