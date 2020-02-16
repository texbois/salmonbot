use crate::behavior::{Behavior, ThreadResult};
use crate::img_match::ImageMatcher;
use crate::vkapi::{Client, VkApi, VkMessage, VkMessagesApi, VkPhotosApi};

pub struct TestBehavior {
    matcher: ImageMatcher,
}

impl TestBehavior {
    pub fn new() -> Self {
        let matcher = ImageMatcher::new();
        Self { matcher }
    }
}

impl std::fmt::Display for TestBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hash")
    }
}

impl<C: Client> Behavior<C> for TestBehavior {
    fn process_on_own_thread<'s>(&'s self, vk: &VkApi<C>, msg: &VkMessage) -> ThreadResult<'s> {
        let attachments = msg.all_attachments();
        if attachments.is_empty() {
            vk.send(msg.from_id, "No images received", None)?;
        }
        for att in attachments {
            let image = vk.download_photo(att)?;
            let hash = self.matcher.hash(&image)?;
            vk.send(msg.from_id, &format!("Hash: {:?}", hash.as_bytes()), None)?;
        }
        Ok(())
    }
}
