use crate::vkapi::{Client, VkApi};
use serde_derive::Deserialize;

pub struct VkLongPoll<'a, C: Client> {
    pub state: VkLongPollState,
    pub api: &'a VkApi<C>,
}

#[derive(Debug, Deserialize)]
pub struct VkLongPollState {
    key: String,
    server: String,
    ts: String,
}

#[derive(Debug, PartialEq)]
pub struct VkMessage {
    pub text: String,
    pub from_id: i64,
    pub attachments: Vec<VkPhoto>,
    pub forwarded: Vec<Box<VkMessage>>,
}

#[derive(Debug, PartialEq)]
pub struct VkPhoto {
    pub max_size_url: String,
}

impl<'a, C: Client> VkLongPoll<'a, C> {
    pub fn init(api: &'a VkApi<C>) -> crate::BotResult<VkLongPoll<'a, C>> {
        let state = api.init_long_poll_state()?;
        Ok(Self { state, api })
    }

    pub fn poll_once<F>(&mut self, mut callback: F) -> crate::BotResult<()>
    where
        F: FnMut(VkMessage) -> crate::BotResult<()>,
    {
        let params = [
            ("act", "a_check"),
            ("key", &self.state.key),
            ("ts", &self.state.ts),
            ("wait", "25"),
        ];
        let mut resp: serde_json::Value =
            self.api
                .client
                .get_json(&self.state.server, &params, None)?;

        self.state.ts = match resp.get_mut("ts").map(|ts| ts.take()) {
            Some(serde_json::Value::String(ts)) => ts,
            _ => return Err(format!("Long poll response missing \"ts\": {:?}", resp).into()),
        };

        match resp.get_mut("updates").map(|u| u.take()) {
            Some(serde_json::Value::Array(updates)) => updates
                .into_iter()
                .filter_map(try_parse_update)
                .try_for_each(&mut callback),
            _ => Err(format!("Long poll response missing \"updates\": {:?}", resp).into()),
        }
    }
}

fn try_parse_update(mut update: serde_json::Value) -> Option<VkMessage> {
    update
        .get_mut("object")?
        .as_object_mut()?
        .remove("message")
        .and_then(try_parse_message)
}

fn try_parse_message(mut message: serde_json::Value) -> Option<VkMessage> {
    let text = match message.get_mut("text").map(|t| t.take()) {
        Some(serde_json::Value::String(s)) => s,
        _ => String::new(),
    };
    let from_id = message.get("from_id")?.as_i64()?;
    let attachments = match message.get_mut("attachments").map(|a| a.take()) {
        Some(serde_json::Value::Array(atts)) => atts
            .into_iter()
            .filter_map(try_extract_attachment)
            .collect(),
        _ => Vec::new(),
    };
    let forwarded = match message.get_mut("fwd_messages").map(|a| a.take()) {
        Some(serde_json::Value::Array(msgs)) => msgs
            .into_iter()
            .filter_map(try_parse_message)
            .map(Box::new)
            .collect(),
        _ => Vec::new(),
    };
    Some(VkMessage {
        text,
        from_id,
        attachments,
        forwarded,
    })
}

fn try_extract_attachment(mut attachment: serde_json::Value) -> Option<VkPhoto> {
    let mut max_photo_size_url = attachment
        .get_mut("photo")?
        .as_object_mut()?
        .remove("sizes")?
        .as_array_mut()?
        .drain(0..)
        .filter(|size| ["m", "x", "y", "z", "w"].contains(&size["type"].as_str().unwrap_or("")))
        .min_by_key(|size| size["width"].as_u64().unwrap_or(std::u64::MAX))?;

    match max_photo_size_url["url"].take() {
        serde_json::Value::String(max_size_url) => Some(VkPhoto { max_size_url }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poll_instantiation() {
        let vk = VkApi {
            client: crate::vkapi::http::TestClient::new("long_poll_init.json"),
            token: "token".into(),
            community_name: "sample_community".into(),
            community_id: "1001".into(),
        };
        let poll = VkLongPoll::init(&vk).unwrap();
        assert_eq!(poll.state.key, "long_poll_key");
        assert_eq!(poll.state.server, "https://long_poll_server");
        assert_eq!(poll.state.ts, "100");
    }

    #[test]
    fn test_parse_forwarded_attachment() {
        let vk = VkApi {
            client: crate::vkapi::http::TestClient::new("long_poll_fwd_attachment.json"),
            token: "token".into(),
            community_name: "sample_community".into(),
            community_id: "1001".into(),
        };
        let mut msg: Option<VkMessage> = None;
        VkLongPoll {
            api: &vk,
            state: VkLongPollState {
                key: "long_poll_key".into(),
                server: "https://long_poll_server".into(),
                ts: "100".into(),
            },
        }
        .poll_once(|m| {
            msg = Some(m);
            Ok(())
        })
        .unwrap();
        assert_eq!(
            msg,
            Some(VkMessage {
                text: "hey check this out".to_owned(),
                from_id: 1010,
                attachments: vec![],
                forwarded: vec![Box::new(VkMessage {
                    text: "forwarded text".to_owned(),
                    from_id: 1020,
                    attachments: vec![VkPhoto {
                        max_size_url: "$med_url".into()
                    }],
                    forwarded: vec![]
                })]
            })
        );
    }
}
