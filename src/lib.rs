use worker::*;
use worker::wasm_bindgen::JsValue;
use std::collections::HashMap;

#[event(fetch)]
pub async fn main(mut req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    // 1. Load Routes Configuration
    // In a real deployment, this would come from `env.var("ROUTES")` or KV.
    // For this port, we'll look for a JSON string in the variable "ROUTES".
    let routes_json = env.var("ROUTES")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "{}".to_string());

    let routes: HashMap<String, String> = serde_json::from_str(&routes_json).unwrap_or_else(|_| {
        console_log!("Failed to parse ROUTES JSON: {}", routes_json);
        HashMap::new()
    });

    let url = req.url()?;
    let path = url.path();

    // 2. Find matching route
    // We look for the longest matching prefix.
    let mut matched_route = None;
    let mut longest_match_len = 0;

    for (route_path, downstream_url) in &routes {
        // Handle wildcard logic simplistically:
        // if key is "/foo/*", we match "/foo/" prefix.
        // If key is just "/foo", we match exact or prefix depending on intent.
        // Usually, "/foo/*" implies prefix match.
        let prefix = route_path.trim_end_matches('*');
        
        if path.starts_with(prefix) {
            // Ensure we match directory boundaries if it's a folder path?
            // strict logic: if prefix ends with /, check starts_with.
            // if prefix doesn't end with /, we might match "/fool" with "/foo".
            // Let's assume users configure "/foo/*" so prefix is "/foo/".
            if prefix.len() > longest_match_len {
                longest_match_len = prefix.len();
                matched_route = Some((prefix, downstream_url));
            }
        }
    }

    if let Some((prefix, target)) = matched_route {
        // 3. Construct Target URL
        let target_base = target.trim_end_matches('*');
        
        // If path is "/api/v1/users" and prefix is "/api/", remainder is "v1/users".
        // If target_base is "https://backend.com/api/", result is "https://backend.com/api/v1/users".
        
        // We need to be careful with slashes.
        // If prefix matches exactly, remainder is empty.
        
        // We'll trust the user config's trailing slashes.
        let remainder = &path[prefix.len()..];
        let new_url_str = format!("{}{}{}", 
            target_base, 
            remainder, 
            url.query().map(|q| format!("?{}", q)).unwrap_or_default()
        );
        
        let new_url = Url::parse(&new_url_str).map_err(|e| worker::Error::RustError(format!("Invalid Target URL: {}", e)))?;

        // 4. Create Proxy Request
        let proxy_headers = req.headers().clone();
        // Update Host header
        if let Some(host) = new_url.host_str() {
             proxy_headers.set("Host", host)?;
        }
        
        // Preserve method
        let method = req.method();
        let mut init = RequestInit::new();
        init.with_method(method.clone());
        init.with_headers(proxy_headers);

        // Body handling
        let should_pass_body = !matches!(req.method(), Method::Get | Method::Head);

        if should_pass_body {
             let body_bytes = req.bytes().await?;
             let js_value: JsValue = body_bytes.into();
             init.with_body(Some(js_value));
        }

        let proxy_req = Request::new_with_init(new_url.as_str(), &init)?;

        // 5. Execute Fetch
        let response = Fetch::Request(proxy_req).send().await?;
        
        Ok(response)

    } else {
        Response::error("No matching route found", 404)
    }
}
