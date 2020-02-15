pub trait Client: Send + Clone + 'static {
    fn fetch(
        &self,
        url: &str,
        query: &[(&str, &str)],
        headers: &[(&str, &str)],
        body: Option<&[u8]>,
    ) -> crate::BotResult<Vec<u8>>;

    #[inline]
    fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, &str)],
        json_response_key: Option<&str>,
    ) -> crate::BotResult<T> {
        let resp = self.fetch(url, query, &[], None)?;
        extract_json(&url, &resp, json_response_key)
    }

    fn post_multipart<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        field_name: &str,
        data: &[u8],
        data_ext: &str, // thank you vk for ignoring content-type and requiring file extensions
        json_response_key: Option<&str>,
    ) -> crate::BotResult<T> {
        let boundary = "----------------------------ImbotMultipartBoundary";
        let mut body = format!(
            "--{}\r\nContent-Disposition: form-data; name=\"{}\"; filename=\"file.{}\"\r\n\r\n",
            boundary, field_name, data_ext
        )
        .into_bytes();
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n--");
        body.extend_from_slice(boundary.as_bytes());
        body.extend_from_slice(b"--");

        let content_type = format!("multipart/form-data;boundary={}", boundary);
        let resp = self.fetch(url, &[], &[("Content-Type", &content_type)], Some(&body))?;
        extract_json(&url, &resp, json_response_key)
    }
}

impl Client for ureq::Agent {
    fn fetch(
        &self,
        url: &str,
        query: &[(&str, &str)],
        headers: &[(&str, &str)],
        body: Option<&[u8]>,
    ) -> crate::BotResult<Vec<u8>> {
        use std::io::Read;

        let mut request = match body {
            Some(_) => self.post(url),
            _ => self.get(url),
        };
        for (param, v) in query {
            request.query(param, v);
        }
        for (header, v) in headers {
            request.set(header, v);
        }
        request.timeout_connect(5_000).timeout_read(30_000);
        let resp = match body {
            Some(data) => request.send_bytes(data),
            _ => request.call(),
        };
        let mut data = Vec::new();
        resp.into_reader().read_to_end(&mut data)?;

        Ok(data)
    }
}

fn extract_json<T: serde::de::DeserializeOwned>(
    url: &str,
    body: &[u8],
    response_key: Option<&str>,
) -> crate::BotResult<T> {
    println!("{} -> {}", url, std::str::from_utf8(&body).unwrap());
    if let Some(key) = response_key {
        serde_json::from_slice::<serde_json::Value>(body)
            .map_err(|e| json_error(url, body, e.into()))?
            .get_mut(key)
            .ok_or_else(|| json_error(url, body, format!("Missing response key {}", key).into()))
            .and_then(|r| {
                serde_json::from_value(r.take()).map_err(|e| json_error(url, body, e.into()))
            })
    } else {
        serde_json::from_slice(&body).map_err(|e| json_error(url, body, e.into()))
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
    use std::cell::RefCell;
    use std::collections::{HashMap, VecDeque};
    use std::iter::FromIterator;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Debug)]
    pub struct TestClient {
        fixtures: Arc<Mutex<RefCell<VecDeque<ClientFixture>>>>,
    }

    #[derive(Debug, Deserialize)]
    struct ClientFixture {
        url: String,
        query: HashMap<String, String>,
        headers: Option<HashMap<String, String>>,
        response: serde_json::Value,
    }

    impl TestClient {
        pub fn new(fixture: &str) -> Self {
            let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), fixture);
            let file = std::fs::File::open(&path).expect(&format!("Failed to open {}", path));
            let fixtures = Arc::new(serde_json::from_reader(file).unwrap());
            Self { fixtures }
        }
    }

    impl Client for TestClient {
        fn fetch(
            &self,
            url: &str,
            query: &[(&str, &str)],
            headers: &[(&str, &str)],
            body: Option<&[u8]>,
        ) -> crate::BotResult<Vec<u8>> {
            let query_map: HashMap<String, String> =
                HashMap::from_iter(query.iter().map(|(k, v)| (k.to_string(), v.to_string())));
            let header_map: HashMap<String, String> =
                HashMap::from_iter(headers.iter().map(|(k, v)| (k.to_string(), v.to_string())));
            let expected = self.fixtures.lock().unwrap().borrow_mut().pop_front();
            match expected {
                Some(ref fixture)
                    if url == fixture.url
                        && query_map == fixture.query
                        && &header_map == fixture.headers.as_ref().unwrap_or(&HashMap::new()) =>
                {
                    Ok(serde_json::to_vec(&fixture.response).unwrap())
                }
                _ => Err(format!(
                    "Expected request: {:?}, got: {:?} {:?} {:?} {:?}",
                    expected, url, query, headers, body
                )
                .into()),
            }
        }
    }
}
