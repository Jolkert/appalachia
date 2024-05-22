use poise::{
	serenity_prelude::{futures::StreamExt, CreateEmbed},
	CreateReply,
};
use rand::prelude::SliceRandom;

use crate::{Context, Error, Reply};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn quote(ctx: Context<'_>) -> Result<(), Error>
{
	let guild = ctx.partial_guild().await.unwrap();

	if let Some(quote_channel_id) = ctx
		.data()
		.acquire_lock()
		.await
		.guild_data(ctx.guild_id().unwrap())
		.and_then(|dat| dat.quotes_channel())
		&& let Some(quote_channel) = guild.channels(ctx).await?.get(quote_channel_id)
	{
		ctx.defer().await?;
		let mut messages_iterator = quote_channel_id.messages_iter(ctx).boxed();

		let mut quotes = Vec::new();
		while let Some(message) = messages_iterator.next().await.transpose()?
		{
			if !message.mentions.is_empty()
			{
				quotes.push(message);
			}
		}

		if let Some(selected_quote) = {
			let mut rng = rand::thread_rng();
			quotes.choose(&mut rng)
		}
		{
			let quote_author = ctx
				.http()
				.get_member(
					guild.id,
					selected_quote
						.mentions
						.first()
						.expect("Selected quote does not mention anyone!")
						.id,
				)
				.await?;

			ctx.send(
				CreateReply::default().embed(
					CreateEmbed::new()
						.title("Here's something someone said")
						.url(selected_quote.link())
						.description(format!("# {}", selected_quote.content))
						.color(
							quote_author
								.user
								.accent_colour
								.unwrap_or(crate::DEFAULT_COLOR),
						)
						.thumbnail(quote_author.face()),
				),
			)
			.await?;
		}
		else
		{
			ctx.reply_error(format!("No quotes found in {}!", quote_channel.name))
				.await?;
		}
	}
	else
	{
		ctx.reply_error(String::from("This server has no quotes channel!"))
			.await?;
	}

	Ok(())
}
