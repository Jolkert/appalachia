pub mod admin;
mod flip;
mod quote;
mod random_user;
mod roll;
mod rps;

pub use flip::flip;
pub use quote::quote;
pub use random_user::random;
pub use roll::roll;
pub use rps::rps;

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
