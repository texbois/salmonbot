use crate::vkapi::{Client, VkApi, VkMessage};
use crate::BotResult;

mod chest;
pub use chest::ChestBehavior;

pub trait Behavior<C: Client>: Send + Sync + std::fmt::Display {
    fn process_on_own_thread(&self, vk: &VkApi<C>, msg: &VkMessage) -> BotResult<()>;
}
