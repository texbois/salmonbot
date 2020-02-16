use crate::vkapi::{Client, VkApi, VkMessage};

mod chest;
pub use chest::ChestBehavior;
mod test;
pub use test::TestBehavior;

pub type ThreadResult<'e> = Result<(), Box<dyn std::error::Error + 'e>>;

pub trait Behavior<C: Client>: Send + Sync + std::fmt::Display {
    fn process_on_own_thread<'s>(&'s self, vk: &VkApi<C>, msg: &VkMessage) -> ThreadResult<'s>;
}
