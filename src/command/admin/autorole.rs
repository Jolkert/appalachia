use poise::{
	serenity_prelude::{Mentionable, Role},
	CreateReply,
};

use crate::{data::GuildData, Context, Error};

#[allow(clippy::unused_async)]
/// Modify the automatically assigned role in the server
#[poise::command(
	prefix_command,
	slash_command,
	guild_only,
	required_permissions = "MANAGE_GUILD",
	required_bot_permissions = "MANAGE_ROLES",
	subcommands("set", "clear", "check")
)]
pub async fn autorole(_: Context<'_>) -> Result<(), Error>
{
	Ok(())
}

/// Change which role is automatically assigned
#[poise::command(
	prefix_command,
	slash_command,
	guild_only,
	required_permissions = "MANAGE_GUILD",
	required_bot_permissions = "MANAGE_ROLES"
)]
pub async fn set(
	ctx: Context<'_>,
	#[description = "The new role to be automatically assigned"] role: Role,
) -> Result<(), Error>
{
	ctx.data()
		.acquire_lock()
		.await
		.guild_data_mut(ctx.guild_id().unwrap())
		.set_autorole(Some(role.id));

	ctx.send(
		CreateReply::default()
			.content(format!("Autorole changed to: {}", role.mention()))
			.ephemeral(true),
	)
	.await?;

	Ok(())
}

/// Clear the current automatically assigned role
#[poise::command(
	prefix_command,
	slash_command,
	guild_only,
	required_permissions = "MANAGE_GUILD",
	required_bot_permissions = "MANAGE_ROLES"
)]
pub async fn clear(ctx: Context<'_>) -> Result<(), Error>
{
	ctx.data()
		.acquire_lock()
		.await
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

/// Show which role is currently being automatically assigned to new members
#[poise::command(
	prefix_command,
	slash_command,
	guild_only,
	required_permissions = "MANAGE_GUILD ",
	required_bot_permissions = "MANAGE_ROLES"
)]
pub async fn check(ctx: Context<'_>) -> Result<(), Error>
{
	let (guild_id, guild_name) = {
		let guild = ctx.guild().unwrap();
		(guild.id, guild.name.clone())
	};

	if let Some(role_id) = ctx
		.data()
		.acquire_lock()
		.await
		.guild_data(guild_id)
		.and_then(GuildData::autorole)
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
