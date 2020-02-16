#![recursion_limit = "256"]

mod vkapi;
use vkapi::{Client, VkApi, VkLongPoll, VkMessage};
mod behavior;
use behavior::{Behavior, ChestBehavior};
mod img_match;

use std::{error::Error, sync::Arc, time::Duration};

pub const MSG_DELAY: Duration = Duration::from_millis(4800);

pub type BotResult<T> = Result<T, Box<dyn Error>>;

struct Bot<C: Client> {
    behavior: Box<dyn Behavior<C>>,
    vk: VkApi<C>,
}

impl<C: Client> std::fmt::Display for Bot<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Behavior: {}, Vk: {}", self.behavior, self.vk)
    }
}

fn main() {
    let token = std::env::var("COMMUNITY_TOKEN")
        .expect("Provide a valid API token via the COMMUNITY_TOKEN environment variable");

    match run_bot(token) {
        Ok(_) => (),
        Err(err) => eprintln!("Error: {}", err),
    }
}

fn run_bot(token: String) -> BotResult<()> {
    println!("Booting up...");

    let bot = Arc::new(Bot {
        vk: VkApi::new(ureq::agent(), token)?,
        behavior: Box::new(ChestBehavior::new()),
    });

    println!("{}", bot);

    let mut lp = VkLongPoll::init(&bot.vk)?;
    loop {
        lp.poll_once(|msg| {
            // TODO: better logging
            println!("Message: {:?}", msg);
            spawn_message_handler(bot.clone(), msg);
            Ok(())
        })?;
    }
}

fn spawn_message_handler<C: Client>(bot: Arc<Bot<C>>, msg: VkMessage) {
    std::thread::spawn(move || {
        if let Err(e) = bot.behavior.process_on_own_thread(&bot.vk, &msg) {
            eprintln!("Error when processing {:?}: {}", msg, e);
        }
    });
}
