#![recursion_limit="256"]

mod vkapi;

pub type BotResult<T> = Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> BotResult<()> {
    let token = std::env::var("COMMUNITY_TOKEN")
        .expect("Provide a valid API token via the COMMUNITY_TOKEN environment variable");

    let vk = vkapi::VkApi::new(token).await?;
    vk.init_long_poll().await?.poll(process_message).await?;

    println!("Vk client: {:?}", vk);

    Ok(())
}

async fn process_message(msg: vkapi::VkMessage) -> BotResult<()> {
    println!("Message: {:?}", msg);
    Ok(())
}
