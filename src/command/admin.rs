use poise::{
	serenity_prelude::{Mentionable, Role},
	CreateReply,
};

use crate::{data::GuildData, Context, Error};

#[allow(clippy::unused_async)]
#[poise::command(
	prefix_command,
	slash_command,
	guild_only,
	required_permissions = "MANAGE_GUILD",
	subcommands("set", "clear", "check")
)]
pub async fn autorole(_: Context<'_>) -> Result<(), Error>
{
	Ok(())
}

#[poise::command(
	prefix_command,
	slash_command,
	guild_only,
	required_permissions = "MANAGE_GUILD"
)]
pub async fn set(ctx: Context<'_>, role: Role) -> Result<(), Error>
{
	ctx.data()
		.acquire_lock()
		.guild_data_mut(ctx.guild_id().unwrap())
		.set_autorole(role.id);

	ctx.send(
		CreateReply::default()
			.content(format!("Autorole changed to: {}", role.mention()))
			.ephemeral(true),
	)
	.await?;

	Ok(())
}

#[poise::command(
	prefix_command,
	slash_command,
	guild_only,
	required_permissions = "MANAGE_GUILD"
)]
pub async fn clear(ctx: Context<'_>) -> Result<(), Error>
{
	ctx.data()
		.acquire_lock()
		.guild_data_mut(ctx.guild_id().unwrap())
		.set_autorole(None);

	ctx.send(
		CreateReply::default()
			.content(format!(
				"Removed autorole from {}",
				ctx.guild().unwrap().name
			))
			.ephemeral(true),
	)
	.await?;

	Ok(())
}

#[poise::command(
	prefix_command,
	slash_command,
	guild_only,
	required_permissions = "MANAGE_GUILD"
)]
pub async fn check(ctx: Context<'_>) -> Result<(), Error>
{
	let (guild_id, guild_name) = {
		let guild = ctx.guild().unwrap();
		(guild.id, guild.name.clone())
	};

	let role_id = ctx
		.data()
		.acquire_lock()
		.guild_data(guild_id)
		.and_then(GuildData::autorole)
		.copied();

	if let Some(role_id) = role_id
	{
		ctx.send(
			CreateReply::default()
				.content(format!("Autorole is: {}", role_id.mention()))
				.ephemeral(true),
		)
		.await?;
	}
	else
	{
		ctx.send(
			CreateReply::default()
				.content(format!("{guild_name} has no autorole set"))
				.ephemeral(true),
		)
		.await?;
	}

	Ok(())
}
