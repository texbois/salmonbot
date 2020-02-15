use crate::behavior::Behavior;
use crate::img_match::ImageMatcher;
use crate::vkapi::{Client, VkApi, VkMessage, VkMessagesApi, VkOutboundMessage, VkPhotosApi};
use crate::{BotResult, MSG_DELAY};

const SUCCESS_IMG: &'static str = "tests/fixtures/test.jpg";
const SUCCESS_TEXT: &'static str =
    "Внутри сундука ты нашел это! Покажи сообщение в канцелярии, чтобы получить награду";
const FAIL_TEXT: &'static str = "Ничего не произошло";

pub struct ChestBehavior {
    matcher: ImageMatcher,
}

impl Behavior for ChestBehavior {
    fn new() -> Self {
        Self {
            matcher: ImageMatcher::new(),
        }
    }

    fn on_message<C: Client>(&mut self, vk: &VkApi<C>, msg: VkMessage) -> BotResult<()> {
        const HASH_WRENCH: [u8; 14] = [
            220, 149, 201, 150, 157, 70, 121, 74, 100, 98, 218, 101, 142, 77,
        ];

        let attachments = msg.all_attachments();
        for att in attachments {
            let image = vk.download_photo(att)?;
            let hash = self.matcher.hash(&image)?;
            if ImageMatcher::matches(HASH_WRENCH, hash) {
                let photo = vk.upload_message_photo(msg.from_id, SUCCESS_IMG)?;
                vk.send_with_delay(
                    VkOutboundMessage::media(msg.from_id, String::from(SUCCESS_TEXT), photo),
                    MSG_DELAY,
                );
                return Ok(());
            }
        }

        vk.send_with_delay(
            VkOutboundMessage::text(msg.from_id, String::from(FAIL_TEXT)),
            MSG_DELAY,
        );
        Ok(())
    }
}
