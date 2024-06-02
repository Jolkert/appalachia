mod admin;
mod flip;
mod quote;
mod random_user;
mod roll;
mod rps;

use crate::{data::Data, Error};
use poise::{
	serenity_prelude::{self as serenity, GuildId},
	Command,
};

pub fn vec() -> Vec<Command<Data, Error>>
{
	vec![
		roll::roll(),
		flip::flip(),
		rps::rps(),
		random_user::random_user(),
		quote::quote(),
		admin::autorole(),
		admin::quote_channel(),
	]
}

#[cfg(debug_assertions)]
pub async fn register(
	ctx: &serenity::Context,
	commands: &[Command<Data, Error>],
) -> Result<(), Error>
{
	poise::builtins::register_in_guild(ctx, commands, GuildId::from(1094129348455436368)).await?;
	poise::builtins::register_in_guild(ctx, commands, GuildId::from(390334803972587530)).await?;
	log::info!("Registered commands in guilds");
	Ok(())
}

#[cfg(not(debug_assertions))]
pub async fn register(
	ctx: &serenity::Context,
	commands: &[Command<Data, Error>],
) -> Result<(), Error>
{
	poise::builtins::register_globally(ctx, commands).await?;
	log::info!("Registered commands globally");
	Ok(())
}

// im not really sure how i feel about this syntax? Maybe reconsider it at some point -morgan
// 2024-05-29
macro_rules! parent_command {
	(let $name:ident = $command_options:meta) => {
		#[allow(clippy::unused_async)]
		#[$command_options]
		pub async fn $name(_: crate::Context<'_>) -> Result<(), crate::Error>
		{
			Ok(())
		}
	};
}

pub(crate) use parent_command;

macro_rules! expect_guild_only_impl {
	($($t:ty),+) => {
		$(
			// honestly? i thought not using the lifetime parameter in the impl
			// would make rustc mad at me lol -morgan 2024-05-31
			impl<'a> ExpectGuildOnly for Option<$t>
			{
				type Output = $t;
				fn expect_guild_only(self) -> Self::Output
				{
					self.expect(&format!(
						"Guild-only command couldn't find {} in context!",
						type_name(stringify!($t))
					))
				}
			}
		)*
	};
}

pub trait ExpectGuildOnly
{
	type Output;
	fn expect_guild_only(self) -> Self::Output;
}

expect_guild_only_impl![
	poise::serenity_prelude::GuildId,
	poise::serenity_prelude::PartialGuild,
	poise::serenity_prelude::GuildChannel,
	poise::serenity_prelude::GuildRef<'a>
];

fn type_name(s: &str) -> &str
{
	let start = s.rfind("::").map(|idx| idx + 2).unwrap_or_default();
	let end = s.find('<').unwrap_or(s.len());

	&s[start..end]
}
