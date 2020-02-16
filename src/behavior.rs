use crate::vkapi::{Client, VkApi, VkMessage};
use crate::BotResult;
use std::sync::Arc;

mod chest;
pub use chest::ChestBehavior;

pub trait Behavior<C: Client>: Send + Sync {
    fn process_on_own_thread(&self, vk: &VkApi<C>, msg: &VkMessage) -> BotResult<()>;
}

pub fn spawn_message_handler<C: Client>(
    handler: Arc<(Box<dyn Behavior<C>>, VkApi<C>)>,
    msg: VkMessage,
) {
    std::thread::spawn(move || {
        if let Err(e) = handler.0.process_on_own_thread(&handler.1, &msg) {
            eprintln!("Error when processing {:?}: {}", msg, e);
        }
    });
}
