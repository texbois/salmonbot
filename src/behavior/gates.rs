use crate::behavior::{Behavior, ThreadResult};
use crate::storage::Storage;
use crate::vkapi::{Client, VkApi, VkMessage, VkMessagesApi};
use crate::{MSG_DELAY_FAIL, MSG_DELAY_SUCCESS};

const SUCCESS_TEXT: &str = "Ворота открылись, и ты можешь идти дальше: vk.com/forestofwisdom";
const FAIL_TEXT: &str = "Ничего не произошло";
const ANSWER: &str = "679823154";

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
            Ok(())
        } else if msg.text.contains(ANSWER) {
            std::thread::sleep(MSG_DELAY_SUCCESS);
            vk.send(msg.from_id, SUCCESS_TEXT, None)?;
            self.storage.set_add(STORAGE_COMPL_SET, msg.from_id)
        } else {
            std::thread::sleep(MSG_DELAY_FAIL);
            vk.send(msg.from_id, FAIL_TEXT, None)
        }
    }
}
