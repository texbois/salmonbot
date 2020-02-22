#![recursion_limit = "256"]

mod vkapi;
use vkapi::{Client, VkApi, VkLongPoll, VkMessage};
mod behavior;
use behavior::*;
mod img_match;
mod storage;

use std::{env, error::Error, sync::Arc, time::Duration};

pub const MSG_DELAY_FAIL: Duration = Duration::from_millis(4800);
pub const MSG_DELAY_SUCCESS: Duration = Duration::from_millis(400);

pub type BotResult<T> = Result<T, Box<dyn Error>>;

const REDIS_URL: &str = "redis://127.0.0.1/";

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
    let args: Vec<String> = env::args().collect();
    let token = env::var("COMMUNITY_TOKEN")
        .expect("Provide a valid API token via the COMMUNITY_TOKEN environment variable");

    println!("Booting up...");
    if let Err(err) = make_bot(args, token).and_then(run_bot) {
        eprintln!("Error: {}", err);
    }
}

fn make_bot(args: Vec<String>, token: String) -> BotResult<Arc<Bot<ureq::Agent>>> {
    let storage = storage::Storage::new(REDIS_URL)?;
    let vk = VkApi::new(ureq::agent(), token)?;
    let behavior: Box<dyn Behavior<ureq::Agent>> = match args.get(1).map(|a| a.as_str()) {
        Some("chest") => Box::new(ChestBehavior::new(storage)),
        Some("gates") => Box::new(GatesBehavior::new(storage)),
        Some("stats") => Box::new(StatsBehavior::new(storage, admin_ids())),
        Some("stone") => Box::new(StoneBehavior::new(storage, admin_ids())),
        Some("test") => Box::new(TestBehavior::new()),
        _ => {
            return Err(format!(
                r#"No behavior specified.
Usage: {} behavior
    where `behavior` is one of the challenges (`chest`, ...)
    or `test` to reply with hashes of received images."#,
                args[0]
            )
            .into())
        }
    };
    Ok(Arc::new(Bot { vk, behavior }))
}

fn admin_ids() -> Vec<i64> {
    let admin_ids = env::var("SALMON_ADMIN_IDS")
        .unwrap_or_default()
        .split(',')
        .filter_map(|id| i64::from_str_radix(id, 10).ok())
        .collect::<Vec<i64>>();
    if admin_ids.is_empty() {
        println!("Warning: no admin users specified (SALMON_ADMIN_IDS is empty)")
    } else {
        println!("Admin users: {:?}", admin_ids)
    }
    admin_ids
}

fn run_bot(bot: Arc<Bot<ureq::Agent>>) -> BotResult<()> {
    println!("{}", bot);

    let mut lp = VkLongPoll::init(&bot.vk)?;
    loop {
        lp.poll_once(|msg| spawn_message_handler(bot.clone(), msg))?;
    }
}

fn spawn_message_handler<C: Client>(bot: Arc<Bot<C>>, msg: VkMessage) {
    std::thread::spawn(move || {
        if let Err(e) = bot.behavior.process_on_own_thread(&bot.vk, &msg) {
            eprintln!("Error when processing {:?}: {}", msg, e);
            eprintln!("Initiating hard shutdown, how do you like THAT Elon Musk?");
            std::process::exit(1);
        }
    });
}
