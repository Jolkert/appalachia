use poise::serenity_prelude::Member;

use crate::{Context, Error};

/// View the Rock, Paper, Scissors leaderboard for this server
#[allow(clippy::too_many_lines)] // this hurts -morgan 2024-05-20
#[poise::command(aliases("lb"), slash_command, prefix_command, guild_only)]
pub async fn leaderboard(
	ctx: Context<'_>,
	#[description = "Specify a user to see their specific score"] user: Option<Member>,
) -> Result<(), Error>
{
	todo!()
}
