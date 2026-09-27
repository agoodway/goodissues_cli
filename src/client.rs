use crate::{args::Args, config::Config, request::Request};

pub fn execute(request: &Request, args: &Args<'_>) -> Result<Vec<u8>, crate::output::Error> {
    let config = Config::load()
        .map_err(|_| "Error: could not load config. Run 'goodissues configure' first.")?;
    let env = config
        .get(args.flag("--env"))
        .ok_or("Error: no environment configured. Run 'goodissues configure' first.")?;
    let url = env
        .base_url
        .as_deref()
        .ok_or("Error: no base URL configured. Run 'goodissues configure --url <url>'.")?;
    let key = env
        .api_key
        .as_deref()
        .ok_or("Error: no API key configured. Run 'goodissues configure --api-key <key>'.")?;
    let client = reqwest::blocking::Client::builder()
        .build()
        .map_err(|e| format!("Error: {e}"))?;
    let method = reqwest::Method::from_bytes(request.method.as_bytes())
        .map_err(|e| format!("Error: {e}"))?;
    let mut call = client
        .request(method, format!("{url}{}", request.path))
        .bearer_auth(key)
        .header("Content-Type", "application/json");
    if let Some(body) = &request.body {
        call = call.body(body.clone());
    }
    let response = call.send().map_err(|e| format!("Error: {e}"))?;
    let status = response.status().as_u16();
    let body = response.bytes().map_err(|e| format!("Error: {e}"))?;
    if !request.expected.contains(&status) {
        let mut error = format!("Error: API returned {status}\n").into_bytes();
        error.extend_from_slice(&body);
        return Err(crate::output::Error(error));
    }
    Ok(body.to_vec())
}
