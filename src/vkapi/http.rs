pub trait Client {
    fn fetch(&self, url: &str, query: &[(&str, &str)]) -> crate::BotResult<Vec<u8>>;

    #[inline]
    fn call_api<T: serde::de::DeserializeOwned>(
        &self,
        token: &str,
        method: &str,
        query: &[(&str, &str)],
        json_response_key: Option<&str>,
    ) -> crate::BotResult<T> {
        self.get_json(
            &format!("https://api.vk.com/method/{}", method),
            &[query, &[("v", "5.103"), ("access_token", token)]].concat(),
            json_response_key,
        )
    }

    fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, &str)],
        json_response_key: Option<&str>,
    ) -> crate::BotResult<T> {
        let body = self.fetch(url, query)?;
        if let Some(key) = json_response_key {
            serde_json::from_slice::<serde_json::Value>(&body)
                .map_err(|e| json_error(url, &body, e.into()))?
                .get_mut(key)
                .ok_or_else(|| {
                    json_error(url, &body, format!("Missing response key {}", key).into())
                })
                .and_then(|r| {
                    serde_json::from_value(r.take()).map_err(|e| json_error(url, &body, e.into()))
                })
        } else {
            serde_json::from_slice(&body).map_err(|e| json_error(url, &body, e.into()))
        }
    }
}

impl Client for ureq::Agent {
    fn fetch(&self, url: &str, query: &[(&str, &str)]) -> crate::BotResult<Vec<u8>> {
        use std::io::Read;

        let mut request = self.get(url);
        for (k, v) in query {
            request.query(k, v);
        }
        let resp = request.timeout_connect(5_000).timeout_read(30_000).call();
        let mut data = Vec::new();
        resp.into_reader().read_to_end(&mut data)?;

        Ok(data)
    }
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
