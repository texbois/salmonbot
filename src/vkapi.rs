mod http;
mod long_poll;
pub use http::Client;
pub use long_poll::{VkLongPoll, VkLongPollState, VkMessage, VkPhoto};

use serde_json;

pub struct VkApi<C: Client> {
    pub client: C,
    token: String,
    community_id: String,
    community_name: String,
}

impl<C: Client> std::fmt::Display for VkApi<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Vk community bot ({}, id {})",
            self.community_name, self.community_id
        )
    }
}

impl<C: Client> VkApi<C> {
    pub async fn new(client: C, token: String) -> crate::BotResult<Self> {
        let communities: serde_json::Value =
            http::call_api(&client, &token, "groups.getById", &[], Some("response")).await?;

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
            client,
            token,
            community_id,
            community_name,
        })
    }

    pub async fn init_long_poll<'a>(&'a self) -> crate::BotResult<VkLongPoll<'a, C>> {
        VkLongPoll::init(self).await
    }

    pub async fn init_long_poll_state(&self) -> crate::BotResult<VkLongPollState> {
        http::call_api(
            &self.client,
            &self.token,
            "groups.getLongPollServer",
            &[("group_id", &self.community_id)],
            Some("response"),
        )
        .await
    }

    pub async fn send_message(&self, peer_id: i64, text: &str) -> crate::BotResult<()> {
        let random_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis()
            .to_string();
        let resp: serde_json::Value = http::call_api(
            &self.client,
            &self.token,
            "messages.send",
            &[
                ("peer_id", &peer_id.to_string()),
                ("message", text),
                ("random_id", &random_id),
            ],
            None,
        )
        .await?;

        if let Some(error) = resp.get("error") {
            Err(format!("messages.send returned an error: {}", error).into())
        } else {
            Ok(())
        }
    }

    pub async fn fetch_photo(&self, photo: &VkPhoto) -> crate::BotResult<bytes::Bytes> {
        self.client.get_bytes(&photo.max_size_url, &[]).await
    }
}
