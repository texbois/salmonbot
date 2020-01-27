pub async fn get_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    query: &[(&str, &str)],
) -> crate::BotResult<T> {
    let response_bytes = client.get(url).query(query).send().await?.bytes().await?;
    serde_json::from_slice(&response_bytes).map_err(|e| {
        format!(
            "Unable to deserialize response: {}\nResponse body: {}",
            e,
            std::str::from_utf8(&response_bytes).unwrap_or("invalid utf8")
        )
        .into()
    })
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
    )
    .await
}
