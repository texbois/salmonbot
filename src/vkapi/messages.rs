use crate::vkapi::{Client, VkApi};
use crate::BotResult;
use std::time::Duration;

#[derive(Debug)]
pub struct VkOutboundMessage {
    peer_id: i64,
    text: String,
    attachment: Option<String>,
}

impl VkOutboundMessage {
    pub fn text(peer_id: i64, text: String) -> Self {
        Self {
            peer_id,
            text,
            attachment: None,
        }
    }

    pub fn media(peer_id: i64, text: String, attachment: String) -> Self {
        Self {
            peer_id,
            text,
            attachment: Some(attachment),
        }
    }
}

pub trait VkMessagesApi {
    fn send_with_delay(&self, msg: VkOutboundMessage, delay: Duration);
    fn send(&self, msg: &VkOutboundMessage) -> BotResult<()>;
}

impl<C: Client> VkMessagesApi for VkApi<C> {
    fn send_with_delay(&self, msg: VkOutboundMessage, delay: Duration) {
        let client = self.clone();
        std::thread::spawn(move || {
            std::thread::sleep(delay);
            if let Err(e) = client.send(&msg) {
                eprintln!("Error when sending a delayed message {:?}: {}", msg, e);
            }
        });
    }

    fn send(&self, msg: &VkOutboundMessage) -> BotResult<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time_now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let random_id = time_now.as_millis().to_string();
        let opt_attachment = msg.attachment.as_ref().map(|s| s.as_str());
        let resp: serde_json::Value = self.call_api(
            "messages.send",
            &[
                ("peer_id", &msg.peer_id.to_string()),
                ("message", &msg.text),
                ("random_id", &random_id),
                ("attachment", opt_attachment.unwrap_or_default()),
            ],
            None,
        )?;
        match resp.get("error") {
            Some(e) => Err(format!("messages.send returned an error: {}", e).into()),
            _ => Ok(()),
        }
    }
}
