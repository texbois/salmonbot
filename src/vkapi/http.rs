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

#[cfg(test)]
pub use test_client::TestClient;

#[cfg(test)]
pub mod test_client {
    use crate::vkapi::http::Client;
    use serde_derive::Deserialize;
    use std::collections::{HashMap, VecDeque};
    use std::iter::FromIterator;

    #[derive(Debug)]
    pub struct TestClient {
        fixtures: std::cell::RefCell<VecDeque<ClientFixture>>,
    }

    #[derive(Debug, Deserialize)]
    struct ClientFixture {
        url: String,
        query: HashMap<String, String>,
        response: serde_json::Value,
    }

    impl TestClient {
        pub fn new(fixture: &str) -> Self {
            let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), fixture);
            let file = std::fs::File::open(&path).expect(&format!("Failed to open {}", path));
            let fixtures = serde_json::from_reader(file).unwrap();
            Self { fixtures }
        }
    }

    impl Client for TestClient {
        fn fetch(&self, url: &str, query: &[(&str, &str)]) -> crate::BotResult<Vec<u8>> {
            let expected = self.fixtures.borrow_mut().pop_front();
            let query_map: HashMap<String, String> =
                HashMap::from_iter(query.iter().map(|(k, v)| (k.to_string(), v.to_string())));
            if let Some(ref fixture) = expected {
                if fixture.url == url && fixture.query == query_map {
                    return Ok(serde_json::to_vec(&fixture.response).unwrap());
                }
            }
            Err(format!(
                "Expected request: {:?}, got: {:?} {:?}",
                expected, url, query
            )
            .into())
        }
    }
}
