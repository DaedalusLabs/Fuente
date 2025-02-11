mod bot;
mod manager;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let vec_strings = include_str!("../relays.txt")
        .trim()
        .lines()
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();
    let relay_pool = nostro2::relays::NostrRelayPool::new(vec_strings).await?;
    let keys = nostro2::keypair::NostrKeypair::new(&std::env::var("FUENTE_PRIV_KEY")?)?;
    let ratings: crate::manager::LiveRatingMap =
        std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));
    let rating_bot = crate::bot::RatingBot {
        keys,
        broadcaster: relay_pool.broadcaster.clone(),
        ratings,
    };
    rating_bot.listen_to_relays(relay_pool).await?;
    Err(anyhow::anyhow!("Bot ended"))
}
