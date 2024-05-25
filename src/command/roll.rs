use std::cmp::Ordering;

use palette::{rgb, FromColor, Oklch, Srgb};
use poise::{
	serenity_prelude::{Color, CreateAllowedMentions, CreateEmbed, CreateEmbedFooter, Mentionable},
	CreateReply,
};
use saikoro::{error::ParsingError, evaluation::DiceEvaluation};

use crate::{Context, Error};

/// Roll some dice!
#[poise::command(slash_command, prefix_command)]
pub async fn roll(
	ctx: Context<'_>,
	#[flag]
	#[description = "When true, only you will see the results"]
	hidden: bool,
	#[description = "The dice expression to be evaluated"]
	#[rest]
	dice: String,
) -> Result<(), Error>
{
	let roll_result = saikoro::evaluate(&dice);
	let mut embed = embed_from_roll(&ctx, &dice, &roll_result);
	if hidden && let poise::Context::Prefix(_) = ctx
	{
		embed = embed.footer(CreateEmbedFooter::new(
			"Note: hidden rolls dont't work with non-slash commands!",
		));
	}

	ctx.send(
		CreateReply::default()
			.embed(embed)
			.reply(true)
			.allowed_mentions(CreateAllowedMentions::new())
			.ephemeral(hidden || roll_result.is_err()),
	)
	.await?;
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
			.color(roll_color(roll))
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
		Err(err) => crate::error_embed(format!(
			"Trying to interpret `{input_string}` failed!\n*{}*",
			err.to_string().replace('*', r"\*")
		)),
	}
}

fn roll_color(roll: &DiceEvaluation) -> Color
{
	const LIGHTNESS: f32 = 0.742;
	const CHROMA: f32 = 0.104;

	const MIN_HUE: f64 = 15.49;
	// yes this makes the gradient non-uniform. i've done it because i wanted the average roll to be
	// yellow not green -morgan 2024-05-25
	const MID_HUE: f64 = 94.01;
	const MAX_HUE: f64 = 228.07;

	let norm_z = roll.mean_z_score_normalized();
	let hue = match norm_z.partial_cmp(&0.0)
	{
		Some(Ordering::Less) => lerp(MIN_HUE, MID_HUE, 1.0 + norm_z),
		Some(Ordering::Greater) => lerp(MID_HUE, MAX_HUE, norm_z),
		_ => MID_HUE,
	} as f32;

	Color::new(
		0xffffff
			& Srgb::from_color(Oklch::new(LIGHTNESS, CHROMA, hue))
				.into_format::<u8>()
				.into_u32::<rgb::channels::Argb>(),
	)
}

fn lerp(a: f64, b: f64, t: f64) -> f64
{
	a.mul_add(1.0 - t, b * t)
}
