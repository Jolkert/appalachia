use std::{collections::HashMap, hash::Hash};

use poise::{
	serenity_prelude::{CreateAllowedMentions, CreateEmbed, Mentionable},
	CreateReply,
};
use saikoro::{error::ParsingError, evaluation::DiceEvaluation};

use crate::{Context, Error};

/// Roll some dice!
#[poise::command(slash_command, prefix_command)]
pub async fn roll(
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
			.color(crate::DEFAULT_COLOR)
			.fields(roll.roll_groups.iter().map(|group| {
				(
					format!("{}d{}", group.len(), group.faces),
					format!(
						"[{}]",
						group
							.iter()
							.map(|roll| {
								let wrap = (roll.original_value >= group.faces)
									.then_some("**")
									.unwrap_or_default();
								format!("{wrap}{roll}{wrap}")
							})
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
			.color(crate::ERROR_COLOR),
	}
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
