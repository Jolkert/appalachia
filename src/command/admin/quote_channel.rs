use poise::{
	serenity_prelude::{GuildChannel, Mentionable},
	CreateReply,
};

use crate::{data::GuildData, Context, Error};

#[allow(clippy::unused_async)]
/// Modify the channel that quotes are pulled from
#[poise::command(
	prefix_command,
	slash_command,
	guild_only,
	required_permissions = "MANAGE_GUILD",
	required_bot_permissions = "MANAGE_CHANNELS",
	subcommands("set", "clear", "check")
)]
pub async fn quote_channel(_: Context<'_>) -> Result<(), Error>
{
	Ok(())
}

/// Change which channel quotes are pulled from
#[poise::command(
	prefix_command,
	slash_command,
	guild_only,
	required_permissions = "MANAGE_GUILD",
	required_bot_permissions = "MANAGE_CHANNELS"
)]
pub async fn set(
	ctx: Context<'_>,
	#[description = "The channel to pull quotes from"] channel: GuildChannel,
) -> Result<(), Error>
{
	ctx.data()
		.acquire_lock()
		.await
		.guild_data_mut(ctx.guild_id().unwrap())
		.set_quotes_channel(Some(channel.id));

	ctx.send(
		CreateReply::default()
			.content(format!("Quotes channel changed to: {}", channel.mention()))
			.ephemeral(true),
	)
	.await?;

	Ok(())
}

/// Clear the channel that quotes are pulled from
#[poise::command(
	prefix_command,
	slash_command,
	guild_only,
	required_permissions = "MANAGE_GUILD",
	required_bot_permissions = "MANAGE_CHANNELS"
)]
pub async fn clear(ctx: Context<'_>) -> Result<(), Error>
{
	ctx.data()
		.acquire_lock()
		.await
		.guild_data_mut(ctx.guild_id().unwrap())
		.set_quotes_channel(None);

	ctx.send(
		CreateReply::default()
			.content(format!(
				"Removed quotes channel from {}",
				ctx.guild().unwrap().name
			))
			.ephemeral(true),
	)
	.await?;

	Ok(())
}

/// Show which channel quotes are currently being pulled from
#[poise::command(
	prefix_command,
	slash_command,
	guild_only,
	required_permissions = "MANAGE_GUILD",
	required_bot_permissions = "MANAGE_CHANNELS"
)]
pub async fn check(ctx: Context<'_>) -> Result<(), Error>
{
	let guild = ctx.partial_guild().await.unwrap();

	if let Some(channel_id) = ctx
		.data()
		.acquire_lock()
		.await
		.guild_data(guild.id)
		.and_then(GuildData::quotes_channel)
	{
		ctx.send(
			CreateReply::default()
				.content(format!("Quotes channel is: {}", channel_id.mention()))
				.ephemeral(true),
		)
		.await?;
	}
	else
	{
		ctx.send(
			CreateReply::default()
				.content(format!("{} has no quotes channel set", guild.name))
				.ephemeral(true),
		)
		.await?;
	}

	Ok(())
}
