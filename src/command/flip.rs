use poise::{
	serenity_prelude::{CreateAllowedMentions, CreateEmbed},
	CreateReply,
};

use crate::{Context, Error};

/// Flip a coin!
#[poise::command(slash_command, prefix_command)]
pub async fn flip(ctx: Context<'_>) -> Result<(), Error>
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
					.color(crate::DEFAULT_COLOR),
			)
			.reply(true)
			.allowed_mentions(CreateAllowedMentions::new()),
	)
	.await?;
	Ok(())
}
