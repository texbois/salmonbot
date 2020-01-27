use crate::vkapi::http::get_json;
use serde_derive::Deserialize;
use std::future::Future;

pub struct VkLongPoll<'a> {
    pub state: VkLongPollState,
    pub client: &'a reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct VkLongPollState {
    key: String,
    server: String,
    ts: u64,
}

#[derive(Debug)]
pub struct VkMessage {}

impl<'a> VkLongPoll<'a> {
    pub async fn poll<F, R>(&mut self, mut callback: F) -> crate::BotResult<()>
    where
        F: FnMut(VkMessage) -> R,
        R: Future<Output = crate::BotResult<()>>,
    {
        let mut resp: serde_json::Value = get_json(
            self.client,
            &format!("https://{}", self.state.server),
            &[
                ("act", "a_check"),
                ("key", &self.state.key),
                ("ts", &self.state.ts.to_string()),
                ("wait", "25"),
            ],
        )
        .await?;

        self.state.ts = resp["ts"]
            .as_u64()
            .ok_or(format!("Long poll response is missing \"ts\": {:?}", resp))?;

        match resp["updates"].take() {
            serde_json::Value::Array(updates) => {
                for u in updates.into_iter().filter_map(try_parse_message) {
                    callback(u).await?;
                }
            }
            _ => {
                return Err(format!("Long poll response is missing \"updates\": {:?}", resp).into())
            }
        }

        Ok(())
    }
}

fn try_parse_message(update: serde_json::Value) -> Option<VkMessage> {
    None
}
