use poise::{
	serenity_prelude::{
		futures::{StreamExt, TryStreamExt},
		CreateEmbed, Member, Mentionable,
	},
	CreateReply,
};
use rand::prelude::SliceRandom;

use crate::{command::ExpectGuildOnly, Context, Error, Reply};

/// Pull a random quote from the server's set quotes channel
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn quote(
	ctx: Context<'_>,
	#[description = "The user to find a quote from"] user: Option<Member>,
) -> Result<(), Error>
{
	let guild = ctx.partial_guild().await.expect_guild_only();

	if let Some(quote_channel_id) = ctx
		.data()
		.acquire_lock()
		.await
		.guild_data(ctx.guild_id().expect_guild_only())
		.and_then(|dat| dat.quotes_channel())
		&& let Some(quote_channel) = guild.channels(ctx).await?.get(quote_channel_id)
	{
		ctx.defer().await?;

		let quotes = quote_channel_id
			.messages_iter(ctx)
			.boxed()
			.try_filter(|msg| {
				std::future::ready(
					!msg.mentions.is_empty()
						&& (user.is_none()
							|| user
								.as_ref()
								.is_some_and(|usr| msg.mentions_user(&usr.user))),
				)
			})
			.try_collect::<Vec<_>>()
			.await?;

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
			ctx.reply_error(format!("No quotes found in {}!", quote_channel.mention()))
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
