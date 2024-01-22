// hex color literals are prefectly fine without spaces thank you very much -morgan 2024-01-15
#![allow(clippy::unreadable_literal)]
mod command;

use poise::{
	serenity_prelude::{
		self as serenity, ActivityData, ClientBuilder, Color, ComponentInteraction,
		CreateAllowedMentions, CreateEmbed, CreateInteractionResponse,
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
async fn main()
{
	let token = std::fs::read_to_string("token").expect("could not find token file!");
	let intents = GatewayIntents::all();

	let framework = poise::Framework::builder()
		.options(poise::FrameworkOptions {
			commands: vec![command::roll(), command::flip(), command::rps()],
			prefix_options: poise::PrefixFrameworkOptions {
				prefix: Some(String::from("$")),
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
}

fn on_ready(ctx: &serenity::Context, ready: &Ready, framework: FrameworkContext<'_, Data, Error>)
{
	println!("Appalachia v{}", env!("CARGO_PKG_VERSION"));
	println!("Discord API v{}", ready.version);
	println!("Loaded {} commands", framework.options.commands.len());

	ctx.set_activity(Some(ActivityData::custom("Trans rights!")));
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
