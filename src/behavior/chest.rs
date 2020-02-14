use crate::behavior::Behavior;
use crate::img_match::ImageMatcher;
use crate::vkapi::{Client, VkApi, VkMessage, VkPhotosApi};
use crate::BotResult;

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

        let reply_attachment = vk.upload_message_photo(msg.from_id, "tests/fixtures/test.jpg")?;

        let attachments = msg.all_attachments();
        for att in attachments {
            let image = vk.fetch_photo(att)?;
            let hash = self.matcher.hash(&image)?;
            if ImageMatcher::matches(HASH_WRENCH, hash) {
                return vk.send_message(msg.from_id, ">", Some(&reply_attachment));
            }
        }

        vk.send_message(msg.from_id, "!", Some(&reply_attachment))
    }
}
