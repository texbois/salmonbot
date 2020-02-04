#[derive(Debug, PartialEq)]
pub struct VkMessage {
    pub text: String,
    pub from_id: i64,
    pub attachments: Vec<VkPhoto>,
    pub forwarded: Vec<VkMessage>,
    pub reply_to: Option<Box<VkMessage>>,
}

#[derive(Debug, PartialEq)]
pub struct VkPhoto(pub String);

impl VkMessage {
    pub fn all_attachments(&self) -> Vec<&VkPhoto> {
        let mut attachments = Vec::new();
        fn append_attachments<'a>(msg: &'a VkMessage, atts: &mut Vec<&'a VkPhoto>) {
            atts.extend(msg.attachments.iter());
            for fwd in msg.forwarded.iter() {
                append_attachments(fwd, atts);
            }
            if let Some(ref reply) = msg.reply_to {
                append_attachments(&reply, atts);
            }
        }
        append_attachments(self, &mut attachments);
        attachments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_attachments() {
        let msg = VkMessage {
            text: String::new(),
            from_id: 0,
            attachments: vec![VkPhoto("$outer".into())],
            forwarded: vec![VkMessage {
                text: String::new(),
                from_id: 1,
                attachments: vec![VkPhoto("$inner".into())],
                forwarded: vec![],
                reply_to: Some(Box::new(VkMessage {
                    text: String::new(),
                    from_id: 2,
                    attachments: vec![VkPhoto("$inner_reply".into())],
                    forwarded: vec![],
                    reply_to: None,
                })),
            }],
            reply_to: None,
        };
        assert_eq!(
            msg.all_attachments(),
            vec![
                &VkPhoto("$outer".into()),
                &VkPhoto("$inner".into()),
                &VkPhoto("$inner_reply".into())
            ]
        )
    }
}
