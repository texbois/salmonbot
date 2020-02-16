#![recursion_limit = "256"]

mod vkapi;
use vkapi::{VkApi, VkLongPoll};
mod behavior;
use behavior::{Behavior, ChestBehavior, spawn_message_handler};
mod img_match;

pub const MSG_DELAY: std::time::Duration = std::time::Duration::from_millis(4800);

pub type BotResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() {
    let token = std::env::var("COMMUNITY_TOKEN")
        .expect("Provide a valid API token via the COMMUNITY_TOKEN environment variable");

    match run_bot(token) {
        Ok(_) => (),
        Err(err) => eprintln!("Error: {}", err),
    }
}

fn run_bot(token: String) -> BotResult<()> {
    use std::sync::Arc;

    let http_client = ureq::agent();
    let vk = VkApi::new(http_client, token)?;

    let behavior: Box<dyn Behavior<ureq::Agent>> = Box::new(ChestBehavior::new());
    let handler = Arc::new((behavior, vk));

    println!("Running {}", handler.1);

    let mut lp = VkLongPoll::init(&handler.1)?;
    loop {
        lp.poll_once(|msg| {
            // TODO: better logging
            println!("Message: {:?}", msg);
            spawn_message_handler(handler.clone(), msg);
            Ok(())
        })?;
    }
}
