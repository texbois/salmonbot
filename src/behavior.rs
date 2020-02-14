use crate::vkapi::{Client, VkApi, VkMessage};
use crate::BotResult;

mod chest;
pub use chest::ChestBehavior;

pub trait Behavior {
    fn new() -> Self;
    fn on_message<C: Client>(&mut self, vk: &VkApi<C>, msg: VkMessage) -> BotResult<()>;
}
