use poise::{
	serenity_prelude::{ChannelType, CreateEmbed, Member, Mentionable},
	CreateReply,
};
use rand::prelude::SliceRandom;

use crate::{command::ExpectGuildOnly, Context, Error, Reply};

/// Generate a random user from the current server
#[poise::command(prefix_command, slash_command, guild_only, rename = "randuser")]
pub async fn random_user(
	ctx: Context<'_>,
	#[description = "Whether to pull only from your current voice channel (Whole server by default)"]
	include: Option<ChannelPolicy>,
	#[description = "Whether or not you should be considered for being potentially pulled (True by default)"]
	include_self: Option<bool>,
	#[description = "Whether or not bots should be considered for being potentially pulled (False by default)"]
	#[flag]
	include_bots: bool,
) -> Result<(), Error>
{
	let channel_policy = include.unwrap_or_default();
	let include_self = include_self.unwrap_or(true);

	// we create this closure here because it makes it a lot more readable than the alternatives.
	// a.) adding a `!` to the front: requires a lot more parens that gets annoying (and it makes
	// clippy angry because you can distribute the `!` and flip the || <-> &&)
	// b.) just distributing the `!`: makes it entirely harder to decipher what the check is
	// actually doing (variable names help for the what but the *why* is a bit more cognitive load
	// than i'd like)
	// c.) isolating the expression to its own function: makes the number of arguments a
	// pain (closure capturing locals is less pain),
	// closure it is -morgan 2024-01-28
	// also, serenity, we *really* shouldve called that `is_bot` cmon
	let should_exclude = |member: &Member| {
		(!include_self && member.user.id == ctx.author().id) || (!include_bots && member.user.bot)
	};

	let guild_id = ctx.guild_id().expect_guild_only();
	let members = if channel_policy == ChannelPolicy::CurrentVoiceChannel
	{
		if let Some((_, channel)) = ctx
			.partial_guild()
			.await
			.expect_guild_only()
			.channels(ctx)
			.await?
			.into_iter()
			.find(|(_, channel)| {
				channel.kind == ChannelType::Voice
					&& channel.members(ctx).is_ok_and(|members| {
						members
							.iter()
							.any(|member| member.user.id == ctx.author().id)
					})
			})
		{
			channel.members(ctx)?
		}
		else
		{
			ctx.reply_error("I couldn't find you in any channels!")
				.await?;
			return Ok(());
		}
	}
	else
	{
		ctx.http()
			.get_guild_members(guild_id, None, None)
			.await?
			.into_iter()
			.filter(|member| !should_exclude(member))
			.collect::<Vec<_>>()
	};

	let generated = {
		// the rng has to go out of scope before the await call. im actually not entirely sure
		// what's going on here? but the borrow checker *really* doesnt like not declaring this
		// binding. ive tried a couple ways of limiting it to its own scope without this, but none
		// of them seem to work properly? kinda weird -morgan 2024-05-14
		let mut rng = rand::thread_rng();
		ctx.http().get_member(
			guild_id,
			members.choose(&mut rng).ok_or(NoMembersError)?.user.id,
		)
	}
	.await?;

	ctx.send(
		CreateReply::default().embed(
			CreateEmbed::new()
				.title("I choose...")
				.description(format!("# {}", generated.mention()))
				.color(generated.user.accent_colour.unwrap_or(crate::DEFAULT_COLOR))
				.thumbnail(generated.face()),
		),
	)
	.await?;

	Ok(())
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("No members found in guild!")]
pub struct NoMembersError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, poise::ChoiceParameter)]
enum ChannelPolicy
{
	#[default]
	#[name = "Whole Server"]
	EntireGuild,
	#[name = "Current Channel Only"]
	CurrentVoiceChannel,
}
