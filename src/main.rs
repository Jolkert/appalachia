// hex color literals are prefectly fine without spaces thank you very much -morgan 2024-01-15
#![allow(clippy::unreadable_literal)]
#![feature(let_chains)]

mod command;
mod data;
mod events;
mod respond;

pub use respond::*;

use data::{config::Config, Data, DataManager};
use poise::serenity_prelude::{ClientBuilder, Color, CreateAllowedMentions, GatewayIntents};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

const DEFAULT_COLOR: Color = Color::new(0xa9e5e5);
const ERROR_COLOR: Color = Color::new(0xe59a9a);

#[tokio::main]
async fn main()
{
	let dot_env_result = dotenv::dotenv();
	env_logger::init();

	if let Err(e) = dot_env_result
		&& !e.not_found()
	{
		log::error!("Error reading .env file {e}");
	}

	let config = Config::load("config.toml").unwrap_or_else(|err| {
		log::error!("Could not read config file! {err}");
		panic!("Could not read config file! {err}")
	});
	if config.token.is_empty()
	{
		log::error!("No token specified!");
		panic!("No token specified!");
	}
	let intents = GatewayIntents::all();

	let framework = poise::Framework::builder()
		.options(poise::FrameworkOptions {
			commands: command::vec(),
			prefix_options: poise::PrefixFrameworkOptions {
				prefix: Some(config.prefix),
				mention_as_prefix: true,
				ignore_bots: true,
				..Default::default()
			},
			pre_command: |ctx| {
				Box::pin(async move {
					log::info!(
						"{} ({}) running [{}]",
						ctx.author().name,
						ctx.author().id,
						ctx.invocation_string()
					);
				})
			},
			post_command: |ctx| {
				Box::pin(async move {
					ctx.data().acquire_lock().await.sync();
				})
			},
			event_handler: |ctx, event, framework, data| {
				Box::pin(events::handle(ctx, event, framework, data))
			},
			allowed_mentions: Some(CreateAllowedMentions::new().all_users(true)),
			..Default::default()
		})
		.setup(|ctx, _ready, framework| {
			Box::pin(async move {
				command::register(ctx, &framework.options().commands).await?;

				Ok(Data::new(
					config.status,
					DataManager::load_or_create_from_dir(config.data_directory),
				))
			})
		})
		.build();

	let client = ClientBuilder::new(config.token, intents)
		.framework(framework)
		.await;

	client
		.expect("Failed to create client!")
		.start()
		.await
		.expect("Failed to start connection!");
}
