use crate::vkapi::{http::get_json, VkApi};
use serde_derive::Deserialize;
use std::future::Future;

pub struct VkLongPoll<'a> {
    pub state: VkLongPollState,
    pub api: &'a VkApi,
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
    pub attachments: Vec<VkPhoto>,
    pub forwarded: Vec<Box<VkMessage>>,
}

#[derive(Debug, PartialEq)]
pub struct VkPhoto {
    pub max_size_url: String,
}

impl<'a> VkLongPoll<'a> {
    pub async fn poll<F, R>(&mut self, mut callback: F) -> crate::BotResult<()>
    where
        F: FnMut(&'a VkApi, VkMessage) -> R,
        R: Future<Output = crate::BotResult<()>>,
    {
        let mut resp: serde_json::Value = get_json(
            &self.api.client,
            &self.state.server,
            &[
                ("act", "a_check"),
                ("key", &self.state.key),
                ("ts", &self.state.ts),
                ("wait", "25"),
            ],
            None,
        )
        .await?;

        self.state.ts = match resp.get_mut("ts").map(|ts| ts.take()) {
            Some(serde_json::Value::String(ts)) => ts,
            _ => return Err(format!("Long poll response missing \"ts\": {:?}", resp).into()),
        };

        match resp.get_mut("updates").map(|u| u.take()) {
            Some(serde_json::Value::Array(updates)) => {
                for u in updates.into_iter().filter_map(try_parse_update) {
                    callback(self.api, u).await?;
                }
            }
            _ => return Err(format!("Long poll response missing \"updates\": {:?}", resp).into()),
        }

        Ok(())
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
        .max_by_key(|size| {
            size["height"].as_u64().unwrap_or(0) + size["width"].as_u64().unwrap_or(0)
        })?;

    match max_photo_size_url["url"].take() {
        serde_json::Value::String(max_size_url) => Some(VkPhoto { max_size_url }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_forwarded_attachment() {
        let update = json!({
            "event_id": "deadbeef",
            "group_id": 1,
            "object": {
                "client_info": {
                    "button_actions": ["text", "vkpay", "open_app", "location", "open_link"],
                    "inline_keyboard": true,
                    "keyboard": true,
                    "lang_id": 3
                },
                "message": {
                    "attachments": [],
                    "conversation_message_id": 2,
                    "date": 1580239358,
                    "from_id": 1000,
                    "id": 2,
                    "important": false,
                    "is_hidden": false,
                    "out": 0,
                    "peer_id": 1000,
                    "random_id": 0,
                    "fwd_messages": [{
                        "attachments": [{
                            "photo": {
                                "access_key": "$photo_key",
                                "album_id": -1,
                                "date": 1580239332,
                                "id": 100,
                                "owner_id": 1000,
                                "sizes": [
                                    { "height": 75, "type": "s", "url": "$small_url", "width": 75 },
                                    { "height": 130, "type": "m", "url": "$med_url", "width": 130 },
                                    { "height": 604, "type": "x", "url": "$x_url", "width": 604 },
                                    { "height": 807, "type": "y", "url": "$800_url", "width": 807 },
                                    { "height": 1080, "type": "z", "url": "$1080_url", "width": 1080 },
                                    { "height": 1903, "type": "w", "url": "$2560_url", "width": 1903 },
                                    { "height": 130, "type": "o", "url": "$o_crop_url", "width": 130 },
                                    { "height": 200, "type": "p", "url": "$p_crop_url", "width": 200 },
                                    { "height": 320, "type": "q", "url": "$q_crop_url", "width": 320 },
                                    { "height": 510, "type": "r", "url": "$r_crop_url", "width": 510 }
                                ],
                                "text": ""
                            },
                            "type": "photo"
                        },
                        {
                            "video": {
                                "ignored": "..."
                            },
                            "type": "video"
                        }],
                        "conversation_message_id": 1,
                        "date": 1580239336,
                        "from_id": 1000,
                        "id": 1,
                        "peer_id": 1000,
                        "text": "forwarded text"
                    }],
                    "text": "hey check this out"
                }
            },
            "type": "message_new"
        });
        let msg = try_parse_update(update);
        assert_eq!(
            msg,
            Some(VkMessage {
                text: "hey check this out".to_owned(),
                attachments: vec![],
                forwarded: vec![Box::new(VkMessage {
                    text: "forwarded text".to_owned(),
                    attachments: vec![VkPhoto {
                        max_size_url: "$2560_url".into()
                    }],
                    forwarded: vec![]
                })]
            })
        );
    }
}
