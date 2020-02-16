use crate::vkapi::{Client, VkApi};
use crate::BotResult;

#[derive(Debug)]
pub struct VkOutboundMessage {
    peer_id: i64,
    text: String,
    attachment: Option<String>,
}

pub trait VkMessagesApi {
    fn send(&self, peer_id: i64, text: &str, attachment: Option<&str>) -> BotResult<()>;
}

impl<C: Client> VkMessagesApi for VkApi<C> {
    fn send(&self, peer_id: i64, text: &str, attachment: Option<&str>) -> BotResult<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time_now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let random_id = time_now.as_millis().to_string();
        let resp: serde_json::Value = self.call_api(
            "messages.send",
            &[
                ("peer_id", &peer_id.to_string()),
                ("message", text),
                ("random_id", &random_id),
                ("attachment", attachment.unwrap_or_default()),
            ],
            None,
        )?;
        match resp.get("error") {
            Some(e) => Err(format!("messages.send returned an error: {}", e).into()),
            _ => Ok(()),
        }
    }
}
