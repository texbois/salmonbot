use crate::behavior::Behavior;
use crate::img_match::ImageMatcher;
use crate::vkapi::{Client, VkApi, VkMessage, VkMessagesApi, VkPhotosApi};
use crate::{BotResult, MSG_DELAY};

const SUCCESS_IMG: &'static str = "tests/fixtures/test.jpg";
const SUCCESS_TEXT: &'static str =
    "Внутри сундука ты нашел это! Покажи сообщение в канцелярии, чтобы получить награду";
const FAIL_TEXT: &'static str = "Ничего не произошло";

pub struct ChestBehavior {
    matcher: ImageMatcher,
}

impl ChestBehavior {
    pub fn new() -> Self {
        Self {
            matcher: ImageMatcher::new(),
        }
    }
}

impl<C: Client> Behavior<C> for ChestBehavior {
    fn process_on_own_thread(&self, vk: &VkApi<C>, msg: &VkMessage) -> BotResult<()> {
        const HASH_WRENCH: [u8; 14] = [
            220, 149, 201, 150, 157, 70, 121, 74, 100, 98, 218, 101, 142, 77,
        ];

        let attachments = msg.all_attachments();
        for att in attachments {
            let image = vk.download_photo(att)?;
            let hash = self.matcher.hash(&image)?;
            if ImageMatcher::matches(HASH_WRENCH, hash) {
                let photo = vk.upload_message_photo(msg.from_id, SUCCESS_IMG)?;

                std::thread::sleep(MSG_DELAY);
                return vk.send(msg.from_id, SUCCESS_TEXT, Some(&photo));
            }
        }

        std::thread::sleep(MSG_DELAY);
        vk.send(msg.from_id, FAIL_TEXT, None)
    }
}
