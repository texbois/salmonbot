use crate::behavior::{Behavior, ThreadResult};
use crate::vkapi::{Client, VkApi, VkMessage, VkMessagesApi};
use crate::storage::Storage;
use crate::MSG_DELAY;

const SUCCESS_TEXT: &str = "RIGHT.";
const FAIL_TEXT: &str = "WRONG.";
const ANSWER: &str = "123456789";
pub const STORAGE_COMPL_SET: &str = "gates_completed_by";

pub struct GatesBehavior {
    storage: Storage,
}

impl GatesBehavior {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

impl std::fmt::Display for GatesBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Gates")
    }
}

impl<C: Client> Behavior<C> for GatesBehavior {
    fn process_on_own_thread<'s>(&'s self, vk: &VkApi<C>, msg: &VkMessage) -> ThreadResult<'s> {
        if self.storage.set_contains(STORAGE_COMPL_SET, msg.from_id)? {
            return Ok(());
        }

        if msg.text.contains(ANSWER) {
            self.storage.set_add(STORAGE_COMPL_SET, msg.from_id)?;

            std::thread::sleep(MSG_DELAY);
            vk.send(msg.from_id, SUCCESS_TEXT, None)?;
        }
        else {
            std::thread::sleep(MSG_DELAY);
            vk.send(msg.from_id, FAIL_TEXT, None)?;
        }
        Ok(())
    }
}
