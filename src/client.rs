use crate::{args::Args, commands::Method, config::Config, output::Error, request::Request};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(30);

impl Method {
    fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
        }
    }
}

pub fn execute(request: &Request, args: &Args<'_>) -> Result<Vec<u8>, Error> {
    let config = Config::load()?;
    let env = config
        .get(args.flag("--env"))
        .ok_or("no environment configured. Run 'goodissues configure' first.")?;
    let url = env
        .base_url
        .as_deref()
        .ok_or("no base URL configured. Run 'goodissues configure --url <url>'.")?;
    let key = env.api_key.as_deref();
    if request.requires_api_key && key.is_none() {
        return Err("no API key configured. Run 'goodissues configure --api-key <key>'.".into());
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(TIMEOUT))
        .user_agent(concat!("goodissues/", env!("CARGO_PKG_VERSION")))
        .build()
        .into();
    let mut builder = ureq::http::Request::builder()
        .method(request.method.as_str())
        .uri(format!("{}{}", url.trim_end_matches('/'), request.path))
        .header("Content-Type", "application/json");
    if let Some(key) = key {
        builder = builder.header("Authorization", format!("Bearer {key}"));
    }
    // POSTs without a body still send `Content-Length: 0`; some proxies reject
    // bodyless POSTs with 411 Length Required.
    let body = match (&request.body, request.method) {
        (Some(body), _) => Some(body.as_str()),
        (None, Method::Post) => Some(""),
        (None, _) => None,
    };
    let result = match body {
        Some(body) => builder.body(body).map(|request| agent.run(request)),
        None => builder.body(()).map(|request| agent.run(request)),
    };
    let mut response = result
        .map_err(|e| format!("invalid request: {e}"))?
        .map_err(|e| format!("request to {url} failed: {e}"))?;
    let status = response.status().as_u16();
    let body = response
        .body_mut()
        .with_config()
        .limit(u64::MAX)
        .read_to_vec()
        .map_err(|e| format!("could not read the API response: {e}"))?;
    if !request.expected.contains(&status) {
        return Err(Error::Response(format!("API returned {status}"), body));
    }
    Ok(body)
}
