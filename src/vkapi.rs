mod http;
mod long_poll;
pub use long_poll::{VkLongPoll, VkMessage};

use serde_json;

#[derive(Debug)]
pub struct VkApi {
    token: String,
    community_id: String,
    community_name: String,
    client: reqwest::Client,
}

impl VkApi {
    pub async fn new(token: String) -> crate::BotResult<Self> {
        let client = reqwest::Client::new();

        let mut communities: serde_json::Value =
            http::call_api(&client, &token, "groups.getById", &[]).await?;

        let mut community = match communities["response"].take() {
            serde_json::Value::Array(comms) if comms.len() == 0 => {
                return Err("The bot is not linked to any communities".into())
            }
            serde_json::Value::Array(mut comms) => comms.remove(0),
            resp => {
                return Err(
                    format!("Unexpected \"response\" for groups.getById: {:?}", resp).into(),
                )
            }
        };

        let community_id = community["id"].as_u64().unwrap().to_string();
        let community_name = match community["name"].take() {
            serde_json::Value::String(name) => name,
            _ => return Err("Missing community name in groups.getById response".into()),
        };

        Ok(Self {
            token,
            community_id,
            community_name,
            client,
        })
    }

    pub async fn init_long_poll<'a>(&'a self) -> crate::BotResult<VkLongPoll<'a>> {
        let state = http::call_api(
            &self.client,
            &self.token,
            "groups.getLongPollServer",
            &[("group_id", &self.community_id)],
        )
        .await?;

        Ok(VkLongPoll {
            state,
            client: &self.client,
        })
    }
}
