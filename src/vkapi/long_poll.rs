use crate::vkapi::{Client, VkApi, VkMessage, VkPhoto};
use serde_derive::Deserialize;
use serde_json::Value as JsonValue;

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

fn get_long_poll_state<C: Client>(vk: &VkApi<C>) -> crate::BotResult<VkLongPollState> {
    vk.call_api(
        "groups.getLongPollServer",
        &[("group_id", &vk.community_id)],
        Some("response"),
    )
}

impl<'a, C: Client> VkLongPoll<'a, C> {
    pub fn init(api: &'a VkApi<C>) -> crate::BotResult<VkLongPoll<'a, C>> {
        let state = get_long_poll_state(api)?;
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
        let mut resp: JsonValue = self
            .api
            .client
            .get_json(&self.state.server, &params, None)?;

        if let Some(err) = resp.get("failed").and_then(|e| e.as_u64()) {
            if err < 4 {
                self.state = get_long_poll_state(self.api)?;
                Ok(())
            } else {
                Err(format!("Long poll request failed: {:?}", resp).into())
            }
        } else {
            self.state.ts = match resp.get_mut("ts").map(|ts| ts.take()) {
                Some(JsonValue::String(ts)) => ts,
                _ => return Err(format!("Long poll response missing \"ts\": {:?}", resp).into()),
            };
            if let Some(JsonValue::Array(updates)) = resp.get_mut("updates") {
                updates
                    .iter_mut()
                    .filter_map(try_parse_update)
                    .try_for_each(&mut callback)
            } else {
                Err(format!("Long poll response missing \"updates\": {:?}", resp).into())
            }
        }
    }
}

fn try_parse_update(update: &mut JsonValue) -> Option<VkMessage> {
    try_parse_message(update.get_mut("object")?.get_mut("message")?)
}

fn try_parse_message(message: &mut JsonValue) -> Option<VkMessage> {
    let text = match message.get_mut("text").map(|t| t.take()) {
        Some(JsonValue::String(s)) => s,
        _ => String::new(),
    };
    let from_id = message.get("from_id")?.as_i64()?;
    let attachments = message
        .get_mut("attachments")
        .and_then(|a| a.as_array_mut())
        .map(|atts| atts.iter_mut().filter_map(try_extract_attachment).collect())
        .unwrap_or_default();
    let forwarded = message
        .get_mut("fwd_messages")
        .and_then(|a| a.as_array_mut())
        .map(|atts| atts.iter_mut().filter_map(try_parse_message).collect())
        .unwrap_or_default();
    let reply_to = message
        .get_mut("reply_message")
        .and_then(try_parse_message)
        .map(Box::new);

    Some(VkMessage {
        text,
        from_id,
        attachments,
        forwarded,
        reply_to,
    })
}

fn try_extract_attachment(attachment: &mut JsonValue) -> Option<VkPhoto> {
    let photo_obj = if let Some(doc) = attachment.get_mut("doc") {
        doc.get_mut("preview")?
    } else {
        attachment
    };

    let mut opt_size_obj = photo_obj
        .get_mut("photo")?
        .get_mut("sizes")?
        .as_array_mut()?
        .drain(..)
        .filter(|size| ["m", "x", "y", "z", "w"].contains(&size["type"].as_str().unwrap_or("")))
        .min_by_key(|size| size["width"].as_u64().unwrap_or(std::u64::MAX))?;

    if let Some(JsonValue::String(url)) = opt_size_obj.get_mut("url").map(|u| u.take()) {
        Some(VkPhoto(url))
    } else if let Some(JsonValue::String(url)) = opt_size_obj.get_mut("src").map(|u| u.take()) {
        Some(VkPhoto(url))
    } else {
        None
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
    fn test_error_response() {
        let vk = VkApi {
            client: crate::vkapi::http::TestClient::new("long_poll_failed.json"),
            token: "token".into(),
            community_name: "sample_community".into(),
            community_id: "1001".into(),
        };
        let mut lp = VkLongPoll {
            api: &vk,
            state: VkLongPollState {
                key: "long_poll_key".into(),
                server: "https://long_poll_server".into(),
                ts: "100".into(),
            },
        };
        lp.poll_once(|_| Ok(())).unwrap();
        assert_eq!(lp.state.key, "new_long_poll_key");
        assert_eq!(lp.state.server, "https://new_long_poll_server");
        assert_eq!(lp.state.ts, "101");
    }

    #[test]
    fn test_parse_reply_document() {
        let vk = VkApi {
            client: crate::vkapi::http::TestClient::new("long_poll_reply_document.json"),
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
                text: "but they are!".to_owned(),
                from_id: 1010,
                attachments: vec![],
                forwarded: vec![],
                reply_to: Some(Box::new(VkMessage {
                    text: "uh, docs aren't photos...".into(),
                    from_id: 1000,
                    attachments: vec![VkPhoto("$med_url".into())],
                    forwarded: vec![],
                    reply_to: None
                }))
            })
        );
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
                text: "hey check this out".into(),
                from_id: 1010,
                attachments: vec![],
                forwarded: vec![VkMessage {
                    text: "forwarded text".into(),
                    from_id: 1020,
                    attachments: vec![VkPhoto("$med_url".into())],
                    forwarded: vec![],
                    reply_to: None
                }],
                reply_to: None
            })
        );
    }
}
