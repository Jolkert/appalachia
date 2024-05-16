// hex color literals are prefectly fine without spaces thank you very much -morgan 2024-01-15
#![allow(clippy::unreadable_literal)]
#![feature(let_chains)]

mod command;
mod data;

use std::{
	path::PathBuf,
	sync::{Arc, Mutex, MutexGuard},
};

use data::{DataManager, GuildData};
use poise::{
	serenity_prelude::{
		self as serenity, ActivityData, CacheHttp, ClientBuilder, Color, ComponentInteraction,
		CreateAllowedMentions, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
		CreateInteractionResponseMessage, FullEvent, GatewayIntents, GuildId, Member, Ready,
	},
	FrameworkContext,
};

struct Data
{
	status: Option<String>,
	data_manager: Arc<Mutex<DataManager>>,
}
impl Data
{
	pub fn acquire_lock(&self) -> MutexGuard<DataManager>
	{
		self.data_manager
			.lock()
			.expect("Unable to acquire lock on data!")
	}
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

const DEFAULT_COLOR: Color = Color::new(0xa9e5e5);
const ERROR_COLOR: Color = Color::new(0xe59a9a);

#[tokio::main]
async fn main() -> Result<(), Error>
{
	dotenv::dotenv()?;
	env_logger::init();

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
				command::autorole(),
			],
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
						ctx.command().name
					);
				})
			},
			post_command: |ctx| {
				Box::pin(async move {
					ctx.data().acquire_lock().sync();
				})
			},
			event_handler: |ctx, event, framework, data| {
				Box::pin(async move {
					match event
					{
						FullEvent::Ready { data_about_bot } =>
						{
							on_ready(ctx, data_about_bot, framework);
						}
						FullEvent::CacheReady { guilds } => on_cache_ready(guilds),
						FullEvent::GuildMemberAddition { new_member } =>
						{
							add_autorole(ctx, new_member, data).await?;
						}
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
					GuildId::from(390334803972587530),
				)
				.await?;
				Ok(Data {
					status: config.status,
					data_manager: Arc::new(Mutex::new(DataManager::load_or_create_from_dir(
						config.data_directory,
					))),
				})
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
	log::info!("Appalachia v{}", env!("CARGO_PKG_VERSION"));
	log::info!("Discord API v{}", ready.version);
	log::info!("Loaded {} commands", framework.options.commands.len());

	ctx.set_activity(
		framework
			.user_data
			.status
			.as_ref()
			.map(ActivityData::custom),
	);
	log::info!("{} online!", ready.user.name);
}

fn on_cache_ready(guilds: &[GuildId])
{
	log::info!("Active in {} guilds", guilds.len());
}

async fn add_autorole(ctx: &serenity::Context, member: &Member, data: &Data) -> Result<(), Error>
{
	let role_id = {
		data.acquire_lock()
			.guild_data(member.guild_id)
			.and_then(GuildData::autorole)
			.copied()
	};

	if let Some(role_id) = role_id
	{
		member.add_role(ctx.http(), role_id).await?;
	}

	Ok(())
}

trait Respond
{
	async fn respond(self, ctx: Context<'_>, embed: CreateEmbed) -> Result<(), Error>;
	async fn respond_ephemeral(self, ctx: Context<'_>, embed: CreateEmbed) -> Result<(), Error>;
}
impl Respond for ComponentInteraction
{
	async fn respond(self, ctx: Context<'_>, embed: CreateEmbed) -> Result<(), Error>
	{
		self.create_response(
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

	async fn respond_ephemeral(self, ctx: Context<'_>, embed: CreateEmbed) -> Result<(), Error>
	{
		self.create_response(
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
	#[serde(default = "Config::default_prefix")]
	prefix: String,

	#[serde(default = "Config::default_data_dir")]
	data_directory: PathBuf,

	#[serde(default)]
	status: Option<String>,
}
impl Default for Config
{
	fn default() -> Self
	{
		Self {
			prefix: Self::default_prefix(),
			data_directory: Self::default_data_dir(),
			status: None,
		}
	}
}
impl Config
{
	fn default_prefix() -> String
	{
		String::from("~")
	}
	fn default_data_dir() -> PathBuf
	{
		dirs::home_dir().map_or_else(
			|| PathBuf::from("./data"),
			|home_dir| home_dir.join(PathBuf::from(".appalachia/data")),
		)
	}
}
