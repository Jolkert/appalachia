// hex color literals are prefectly fine without spaces thank you very much -morgan 2024-01-15
#![allow(clippy::unreadable_literal)]
#![feature(let_chains, iter_intersperse)]

mod command;
mod data;

use std::{path::PathBuf, sync::Arc};

use data::{DataManager, GuildData};
use poise::{
	serenity_prelude::{
		self as serenity,
		futures::lock::{Mutex, MutexGuard},
		ActivityData, CacheHttp, ClientBuilder, Color, ComponentInteraction, CreateAllowedMentions,
		CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
		CreateInteractionResponseMessage, FullEvent, GatewayIntents, GuildId, Member, Ready,
	},
	Command, CreateReply, FrameworkContext,
};

struct Data
{
	status: Option<String>,
	data_manager: Arc<Mutex<DataManager>>,
}
impl Data
{
	pub async fn acquire_lock(&self) -> MutexGuard<DataManager>
	{
		self.data_manager.lock().await
	}
}
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

	let config = load_config().unwrap_or_else(|err| {
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
			commands: vec![
				command::roll(),
				command::flip(),
				command::rps(),
				command::random(),
				command::quote(),
				command::admin::autorole(),
				command::admin::quote_channel(),
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
				register_commands(ctx, &framework.options().commands).await?;

				Ok(Data {
					status: config.status,
					data_manager: Arc::new(Mutex::new(DataManager::load_or_create_from_dir(
						config.data_directory,
					))),
				})
			})
		})
		.build();

	let client = ClientBuilder::new(config.token, intents)
		.framework(framework)
		.await;

	client.unwrap().start().await.unwrap();
}

#[cfg(debug_assertions)]
async fn register_commands(
	ctx: &serenity::Context,
	commands: &[Command<Data, Error>],
) -> Result<(), Error>
{
	poise::builtins::register_in_guild(ctx, commands, GuildId::from(1094129348455436368)).await?;
	poise::builtins::register_in_guild(ctx, commands, GuildId::from(390334803972587530)).await?;
	log::info!("Registered commands in guilds");
	Ok(())
}

#[cfg(not(debug_assertions))]
async fn register_commands(
	ctx: &serenity::Context,
	commands: &[Command<Data, Error>],
) -> Result<(), Error>
{
	poise::builtins::register_globally(ctx, commands).await?;
	log::info!("Registered commands globally");
	Ok(())
}

fn load_config() -> Result<Config, Error>
{
	let config_file_path = config_root().join("config.toml");

	match std::fs::read_to_string(&config_file_path)
	{
		Ok(file_content) => toml::from_str(&file_content).map_err(Into::into),
		Err(err) if err.kind() == std::io::ErrorKind::NotFound =>
		{
			let config = Config::default();
			std::fs::write(config_file_path, toml::to_string_pretty(&config)?)?;
			Ok(config)
		}
		Err(err) => Err(Box::new(err)),
	}
}

#[cfg(debug_assertions)]
fn config_root() -> PathBuf
{
	PathBuf::from("./")
}

#[cfg(not(debug_assertions))]
fn config_root() -> PathBuf
{
	let dir = dirs::config_dir().map_or_else(
		|| PathBuf::from("./"),
		|config_dir| config_dir.join("appalachia/"),
	);
	std::fs::create_dir_all(&dir).expect("Failed to create config directory!");
	dir
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
			.await
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

trait Reply
{
	async fn reply_error(self, error_text: String) -> Result<(), Error>;
}
impl Reply for Context<'_>
{
	async fn reply_error(self, error_text: String) -> Result<(), Error>
	{
		self.send(
			CreateReply::default()
				.embed(crate::error_embed(error_text))
				.reply(true)
				.allowed_mentions(CreateAllowedMentions::new())
				.ephemeral(true),
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
	#[serde(default)]
	token: String,

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
			token: String::default(),
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
		config_root().join("data/")
	}
}
