mod http;
mod long_poll;
pub use long_poll::{VkLongPoll, VkMessage, VkPhoto};

use serde_json;

#[derive(Debug)]
pub struct VkApi {
    token: String,
    community_id: String,
    community_name: String,
    pub client: reqwest::Client,
}

impl VkApi {
    pub async fn new(token: String) -> crate::BotResult<Self> {
        let client = reqwest::Client::new();

        let communities =
            http::call_api::<serde_json::Value>(&client, &token, "groups.getById", &[])
                .await?
                .take();

        let mut community = match communities {
            serde_json::Value::Array(mut comms) if comms.len() == 1 => comms.remove(0),
            resp => {
                return Err(format!(
                    "The bot should be linked is linked to none or multiple communities, got: {:?}",
                    resp
                )
                .into())
            }
        };
        let community_id = match community.get("id").and_then(|id| id.as_u64()) {
            Some(id) => id.to_string(),
            _ => return Err(format!("Group item missing \"id\": {:?}", community).into()),
        };
        let community_name = match community.get_mut("name").map(|n| n.take()) {
            Some(serde_json::Value::String(name)) => name,
            _ => return Err(format!("Group item missing \"name\" {:?}", community).into()),
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

        Ok(VkLongPoll { state, api: &self })
    }

    pub async fn fetch_photo(&self, photo: &VkPhoto) -> crate::BotResult<bytes::Bytes> {
        let body = self
            .client
            .get(&photo.max_size_url)
            .send()
            .await?
            .bytes()
            .await?;
        Ok(body)
    }
}
