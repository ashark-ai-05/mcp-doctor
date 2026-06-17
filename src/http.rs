use anyhow::Context;
use serde_json::{Value, json};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HttpResult {
    pub initialize: Value,
    pub tools: Value,
}

pub fn connect_http(url: &str, timeout: Duration) -> anyhow::Result<HttpResult> {
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .context("build HTTP client")?;
    let initialize = json!({
        "jsonrpc":"2.0",
        "id":1,
        "method":"initialize",
        "params":{
            "protocolVersion":"2025-06-18",
            "capabilities":{},
            "clientInfo":{"name":"mcp-doctor","version":env!("CARGO_PKG_VERSION")}
        }
    });
    let initialize_response = post_json(&client, url, &initialize)?;
    let tools = json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}});
    let tools_response = post_json(&client, url, &tools)?;
    Ok(HttpResult {
        initialize: initialize_response,
        tools: tools_response,
    })
}

fn post_json(
    client: &reqwest::blocking::Client,
    url: &str,
    payload: &Value,
) -> anyhow::Result<Value> {
    let response = client
        .post(url)
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .json(payload)
        .send()
        .with_context(|| format!("POST {url}"))?;
    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("HTTP MCP endpoint returned status {status}");
    }
    let value = response
        .json::<Value>()
        .context("parse HTTP MCP JSON response")?;
    Ok(value)
}
