use crate::behavior::{Behavior, ThreadResult};
use crate::img_match::ImageMatcher;
use crate::storage::Storage;
use crate::vkapi::{Client, VkApi, VkMessage, VkMessagesApi, VkPhotosApi};
use crate::MSG_DELAY_SUCCESS;
use crate::MSG_DELAY_FAIL;

const SUCCESS_IMG: (&[u8], &str) = (include_bytes!("../../static/chest_success.jpg"), "jpg");
const SUCCESS_TEXT: &str =
    "Внутри сундука ты нашел это! Покажи сообщение в канцелярии, чтобы получить награду";
const FAIL_TEXT: &str = "Ничего не произошло";
const HASH_WRENCH: [u8; 18] = [
    220, 171, 38, 54, 217, 211, 81, 60, 164, 202, 200, 137, 211, 93, 76, 99, 38, 148,
];

pub const STORAGE_COMPL_SET: &str = "chest_completed_by";

pub struct ChestBehavior {
    matcher: ImageMatcher,
    storage: Storage,
}

impl ChestBehavior {
    pub fn new(storage: Storage) -> Self {
        let matcher = ImageMatcher::new();
        Self { matcher, storage }
    }
}

impl std::fmt::Display for ChestBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chest")
    }
}

impl<C: Client> Behavior<C> for ChestBehavior {
    fn process_on_own_thread<'s>(&'s self, vk: &VkApi<C>, msg: &VkMessage) -> ThreadResult<'s> {
        if self.storage.set_contains(STORAGE_COMPL_SET, msg.from_id)? {
            return Ok(());
        }

        for att in msg.all_attachments() {
            let image = vk.download_photo(att)?;
            let hash = self.matcher.hash(&image)?;
            if ImageMatcher::matches(&HASH_WRENCH, &hash) {
                let photo = vk.upload_message_photo(msg.from_id, SUCCESS_IMG)?;
                let completed_cnt = self.storage.set_add(STORAGE_COMPL_SET, msg.from_id)?;

                println!(
                    "Chest challenge completed by {} (total completions: {})",
                    msg.from_id, completed_cnt
                );

                std::thread::sleep(MSG_DELAY_SUCCESS);
                return vk.send(msg.from_id, SUCCESS_TEXT, Some(&photo));
            }
        }

        std::thread::sleep(MSG_DELAY_FAIL);
        vk.send(msg.from_id, FAIL_TEXT, None)
    }
}
