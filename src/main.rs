// hex color literals are prefectly fine without spaces thank you very much -morgan 2024-01-15
#![allow(clippy::unreadable_literal)]
#![feature(let_chains)]

mod command;

use poise::{
	serenity_prelude::{
		self as serenity, ActivityData, ClientBuilder, Color, ComponentInteraction,
		CreateAllowedMentions, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
		CreateInteractionResponseMessage, FullEvent, GatewayIntents, GuildId, Ready,
	},
	FrameworkContext,
};

struct Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

const DEFAULT_COLOR: Color = Color::new(0xa9e5e5);
const ERROR_COLOR: Color = Color::new(0xe59a9a);

#[tokio::main]
async fn main() -> Result<(), Error>
{
	let token = std::fs::read_to_string("token").expect("could not find token file!");
	let intents = GatewayIntents::all();

	let config = load_config()?;

	let framework = poise::Framework::builder()
		.options(poise::FrameworkOptions {
			commands: vec![
				command::roll(),
				command::flip(),
				command::rps(),
				command::random(),
			],
			prefix_options: poise::PrefixFrameworkOptions {
				prefix: Some(config.prefix),
				mention_as_prefix: true,
				ignore_bots: true,
				..Default::default()
			},
			pre_command: |ctx| {
				Box::pin(async move {
					println!(
						"{} ({}) running [{}]",
						ctx.author().name,
						ctx.author().id,
						ctx.command().name
					);
				})
			},
			event_handler: |ctx, event, framework, _data| {
				Box::pin(async move {
					match event
					{
						FullEvent::Ready { data_about_bot } =>
						{
							on_ready(ctx, data_about_bot, framework);
						}
						FullEvent::CacheReady { guilds } => on_cache_ready(guilds),
						_ => (),
					}

					Ok(())
				})
			},
			allowed_mentions: Some(CreateAllowedMentions::new().all_users(true)),
			..Default::default()
		})
		.setup(|ctx, _ready, framework| {
			Box::pin(async move {
				// TODO: register globally in final build. guild is faster for tests
				// -morgan 2024-01-15
				poise::builtins::register_in_guild(
					ctx,
					&framework.options().commands,
					GuildId::from(1094129348455436368),
				)
				.await?;
				Ok(Data)
			})
		})
		.build();

	let client = ClientBuilder::new(token, intents)
		.framework(framework)
		.await;

	client.unwrap().start().await.unwrap();
	Ok(())
}

fn load_config() -> Result<Config, Error>
{
	match std::fs::read_to_string("config.toml")
	{
		Ok(file_content) => toml::from_str(&file_content).map_err(Into::into),
		Err(err) if err.kind() == std::io::ErrorKind::NotFound =>
		{
			let config = Config::default();
			std::fs::write("config.toml", toml::to_string_pretty(&config)?)?;
			Ok(config)
		}
		Err(err) => Err(Box::new(err)),
	}
}

fn on_ready(ctx: &serenity::Context, ready: &Ready, framework: FrameworkContext<'_, Data, Error>)
{
	println!("Appalachia v{}", env!("CARGO_PKG_VERSION"));
	println!("Discord API v{}", ready.version);
	println!("Loaded {} commands", framework.options.commands.len());

	ctx.set_activity(Some(ActivityData::custom(
		"\u{1f3f3}\u{200d}\u{26a7} Trans rights!",
	)));
	println!("{} online!", ready.user.name);
}

fn on_cache_ready(guilds: &[GuildId])
{
	println!("Active in {} guilds", guilds.len());
}

async fn respond_to(
	interaction: ComponentInteraction,
	ctx: Context<'_>,
	embed: CreateEmbed,
) -> Result<(), Error>
{
	interaction
		.create_response(
			ctx,
			CreateInteractionResponse::Message(
				CreateInteractionResponseMessage::new()
					.embed(embed)
					.allowed_mentions(CreateAllowedMentions::new()),
			),
		)
		.await?;
	Ok(())
}

async fn respond_ephemeral(
	interaction: ComponentInteraction,
	ctx: Context<'_>,
	embed: CreateEmbed,
) -> Result<(), Error>
{
	interaction
		.create_response(
			ctx,
			CreateInteractionResponse::Message(
				CreateInteractionResponseMessage::new()
					.embed(embed)
					.allowed_mentions(CreateAllowedMentions::new())
					.ephemeral(true),
			),
		)
		.await?;
	Ok(())
}

fn error_embed(description: impl Into<String>) -> CreateEmbed
{
	CreateEmbed::new()
		.title("Error")
		.description(description)
		.color(ERROR_COLOR)
		.footer(
			CreateEmbedFooter::new("If you think this is a bug, contact my mama, Jolkert!")
				.icon_url("https://jolkert.dev/img/icon_small.png"),
		)
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct Config
{
	prefix: String,
}
impl Default for Config
{
	fn default() -> Self
	{
		Self {
			prefix: String::from("$"),
		}
	}
}
