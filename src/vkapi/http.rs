pub async fn get_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    query: &[(&str, &str)],
    json_response_key: Option<&str>,
) -> crate::BotResult<T> {
    let body = client.get(url).query(query).send().await?.bytes().await?;

    if let Some(key) = json_response_key {
        serde_json::from_slice::<serde_json::Value>(&body)
            .map_err(|e| json_error(url, &body, e.into()))?
            .get_mut(key)
            .ok_or_else(|| json_error(url, &body, format!("Missing response key {}", key).into()))
            .and_then(|r| {
                serde_json::from_value(r.take()).map_err(|e| json_error(url, &body, e.into()))
            })
    } else {
        serde_json::from_slice(&body).map_err(|e| json_error(url, &body, e.into()))
    }
}

#[inline]
pub async fn call_api<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    token: &str,
    method: &str,
    query: &[(&str, &str)],
) -> crate::BotResult<T> {
    get_json(
        client,
        &format!("https://api.vk.com/method/{}", method),
        &[query, &[("v", "5.103"), ("access_token", token)]].concat(),
        Some("response"),
    )
    .await
}

fn json_error(
    url: &str,
    source: &[u8],
    error: Box<dyn std::error::Error>,
) -> Box<dyn std::error::Error> {
    format!(
        "Unable to deserialize response: {}\nRequest URL: {}\nResponse body: {}",
        error,
        url,
        std::str::from_utf8(source).unwrap_or("*invalid utf8*")
    )
    .into()
}
