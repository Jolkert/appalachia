use poise::serenity_prelude::RoleId;

use super::Leaderboard;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct GuildData
{
	#[serde(default)]
	autorole: Option<RoleId>,
	#[serde(default)]
	rps_leaderboard: Leaderboard,
}
impl GuildData
{
	pub fn autorole(&self) -> Option<&RoleId>
	{
		self.autorole.as_ref()
	}

	pub fn set_autorole(&mut self, role: impl Into<Option<RoleId>>)
	{
		self.autorole = role.into();
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
