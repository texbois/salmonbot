#![recursion_limit = "256"]

mod vkapi;
use vkapi::{VkApi, VkLongPoll};
mod behavior;
use behavior::{Behavior, ChestBehavior};
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
    let client = ureq::agent();
    let vk = VkApi::new(client, token)?;
    println!("Running {}", vk);

    let mut behavior = ChestBehavior::new();

    let mut lp = VkLongPoll::init(&vk)?;
    loop {
        lp.poll_once(|msg| {
            // TODO: better logging
            println!("Message: {:?}", msg);
            behavior.on_message(&vk, msg)
        })?;
    }
}
