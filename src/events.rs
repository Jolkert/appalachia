use poise::{
	serenity_prelude::{
		self as serenity, ActivityData, CacheHttp, FullEvent, GuildId, Member, Ready,
	},
	FrameworkContext,
};

use crate::{
	data::{Data, GuildData},
	Error,
};

pub async fn handle(
	ctx: &serenity::prelude::Context,
	event: &FullEvent,
	framework: FrameworkContext<'_, Data, Error>,
	data: &Data,
) -> Result<(), Error>
{
	match event
	{
		FullEvent::Ready { data_about_bot } =>
		{
			on_ready(ctx, data_about_bot, framework);
		}
		FullEvent::CacheReady { guilds } => on_cache_ready(guilds),
		FullEvent::GuildMemberAddition { new_member } =>
		{
			add_autorole(ctx, new_member, data).await?;
		}
		_ => (),
	}

	Ok(())
}

fn on_ready(ctx: &serenity::Context, ready: &Ready, framework: FrameworkContext<'_, Data, Error>)
{
	log::info!("Appalachia v{}", env!("CARGO_PKG_VERSION"));
	log::info!("Discord API v{}", ready.version);
	log::info!("Loaded {} commands", framework.options.commands.len());

	ctx.set_activity(framework.user_data.status().map(ActivityData::custom));
	log::info!("{} online!", ready.user.name);
}

fn on_cache_ready(guilds: &[GuildId])
{
	log::info!("Active in {} guilds", guilds.len());
}

async fn add_autorole(ctx: &serenity::Context, member: &Member, data: &Data) -> Result<(), Error>
{
	let role_id = {
		data.acquire_lock()
			.await
			.guild_data(member.guild_id)
			.and_then(GuildData::autorole)
			.copied()
	};

	if let Some(role_id) = role_id
	{
		member.add_role(ctx.http(), role_id).await?;
	}

	Ok(())
}
