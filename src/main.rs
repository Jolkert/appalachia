// hex color literals are prefectly fine without spaces thank you very much -morgan 2024-01-15
#![allow(clippy::unreadable_literal)]

use poise::{
	serenity_prelude::{
		ClientBuilder, CreateAllowedMentions, CreateEmbed, GatewayIntents, GuildId, Mentionable,
	},
	CreateReply,
};
use saikoro::{error::ParsingError, evaluation::DiceEvaluation};

struct Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main()
{
	let token = std::fs::read_to_string("token").expect("could not find token file!");
	let intents = GatewayIntents::all();

	let framework = poise::Framework::builder()
		.options(poise::FrameworkOptions {
			commands: vec![roll()],
			prefix_options: poise::PrefixFrameworkOptions {
				prefix: Some(String::from("$")),
				mention_as_prefix: true,
				ignore_bots: true,
				..Default::default()
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

#[poise::command(slash_command, prefix_command)]
async fn roll(
	ctx: Context<'_>,
	#[description = "Dice roll"]
	#[rest]
	roll_str: String,
) -> Result<(), Error>
{
	let reply = CreateReply::default()
		.embed(embed_from_roll(
			&ctx,
			&roll_str,
			saikoro::evaluate(&roll_str),
		))
		.reply(true)
		.allowed_mentions(CreateAllowedMentions::default().empty_roles().empty_users());

	ctx.send(reply).await?;
	Ok(())
}

fn embed_from_roll(
	ctx: &Context<'_>,
	input_string: &str,
	roll: Result<DiceEvaluation, ParsingError>,
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
			.color(0x9ae59a)
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
			.color(0xe59a9a),
	}
}
