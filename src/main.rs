// hex color literals are prefectly fine without spaces thank you very much -morgan 2024-01-15
#![allow(clippy::unreadable_literal)]
mod rps;

use std::{collections::HashMap, hash::Hash};

use poise::serenity_prelude::ActivityData;
use poise::{serenity_prelude as serenity, FrameworkContext};
use poise::{
	serenity_prelude::{
		ClientBuilder, Color, CreateAllowedMentions, CreateEmbed, FullEvent, GatewayIntents,
		GuildId, Mentionable, Ready,
	},
	CreateReply,
};
use saikoro::{error::ParsingError, evaluation::DiceEvaluation};

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
			commands: vec![roll(), flip(), rps::rps()],
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
			event_handler: |ctx, event, framework, data| {
				Box::pin(async move {
					if let FullEvent::Ready { data_about_bot } = event
					{
						on_ready(ctx, data_about_bot, framework, data);
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

#[allow(unused_variables)]
fn on_ready(
	ctx: &serenity::Context,
	ready: &Ready,
	framework: FrameworkContext<'_, Data, Error>,
	data: &Data,
)
{
	println!("API v{}", ready.version);
	println!("Loaded {} commands", framework.options.commands.len());

	ctx.set_activity(Some(ActivityData::custom("Trans rights!")));
	println!("{} online!", ready.user.name);
}

/// Enter a dice expression to roll
#[poise::command(slash_command, prefix_command)]
async fn roll(
	ctx: Context<'_>,
	#[description = "Dice expression"]
	#[rest]
	roll_str: String,
) -> Result<(), Error>
{
	let roll_result = saikoro::evaluate(&roll_str);
	let reply = CreateReply::default()
		.embed(embed_from_roll(&ctx, &roll_str, &roll_result))
		.reply(true)
		.allowed_mentions(CreateAllowedMentions::new());

	if roll_result.is_err()
	{
		ctx.defer_ephemeral().await?;
	}

	ctx.send(reply).await?;
	Ok(())
}

fn embed_from_roll(
	ctx: &Context<'_>,
	input_string: &str,
	roll: &Result<DiceEvaluation, ParsingError>,
) -> CreateEmbed
{
	match roll
	{
		Ok(roll) => CreateEmbed::new()
			.title("The dice have spoken")
			.description(format!(
				"# **{}**\n{} rolled `{input_string}`",
				roll.value,
				ctx.author().mention(),
			))
			.color(DEFAULT_COLOR)
			.fields(roll.roll_groups.iter().map(|group| {
				(
					format!("{}d{}", group.len(), group.faces),
					format!(
						"[{}]",
						group
							.iter()
							.map(ToString::to_string)
							.collect::<Vec<_>>()
							.join(", ")
					),
					true,
				)
			})),
		Err(err) => CreateEmbed::new()
			.title("Error parsing dice expression!")
			.description(format!(
				"Trying to interpret `{input_string}` failed!\n*{}*",
				err.to_string().replace('*', r"\*")
			))
			.color(ERROR_COLOR),
	}
}

#[poise::command(slash_command, prefix_command)]
async fn flip(ctx: Context<'_>) -> Result<(), Error>
{
	ctx.send(
		CreateReply::default()
			.embed(
				CreateEmbed::new()
					.title("Fortuna says:")
					.description(format!(
						"# {}",
						if rand::random::<bool>()
						{
							"Heads"
						}
						else
						{
							"Tails"
						}
					))
					.color(DEFAULT_COLOR),
			)
			.reply(true)
			.allowed_mentions(CreateAllowedMentions::new()),
	)
	.await?;
	Ok(())
}

trait InsertPair<K, V>
{
	fn insert_pair(&mut self, pair: (K, V)) -> Option<V>;
}
impl<K: Eq + Hash, V> InsertPair<K, V> for HashMap<K, V>
{
	fn insert_pair(&mut self, pair: (K, V)) -> Option<V>
	{
		self.insert(pair.0, pair.1)
	}
}
