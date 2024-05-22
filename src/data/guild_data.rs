use poise::serenity_prelude::{ChannelId, RoleId};

use super::Leaderboard;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct GuildData
{
	#[serde(default)]
	autorole: Option<RoleId>,
	#[serde(default)]
	quotes_channel: Option<ChannelId>,
	#[serde(default)]
	rps_leaderboard: Leaderboard,
}
impl GuildData
{
	pub fn autorole(&self) -> Option<&RoleId>
	{
		self.autorole.as_ref()
	}
	pub fn set_autorole(&mut self, role: Option<RoleId>)
	{
		self.autorole = role;
	}

	pub fn quotes_channel(&self) -> Option<&ChannelId>
	{
		self.quotes_channel.as_ref()
	}
	pub fn set_quotes_channel(&mut self, quotes_channel: Option<ChannelId>)
	{
		self.quotes_channel = quotes_channel;
	}

	pub fn leaderboard(&self) -> &Leaderboard
	{
		&self.rps_leaderboard
	}
	pub fn leaderboard_mut(&mut self) -> &mut Leaderboard
	{
		&mut self.rps_leaderboard
	}
}
