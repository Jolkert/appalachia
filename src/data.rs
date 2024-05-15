use std::{collections::HashMap, path::PathBuf};

use poise::serenity_prelude::{GuildId, RoleId};

#[derive(Debug)]
pub struct DataManager
{
	dir_path: PathBuf,
	unsynced: bool,
	guild_data: HashMap<GuildId, GuildData>,
}
impl DataManager
{
	pub fn new(dir_path: PathBuf) -> Self
	{
		Self {
			dir_path,
			unsynced: false,
			guild_data: HashMap::default(),
		}
	}
	pub fn load_from_dir(dir_path: PathBuf) -> Result<Self, DataLoadError>
	{
		let guild_data_path = dir_path.join("guild_data.toml");

		Ok(Self {
			dir_path,
			unsynced: false,
			guild_data: toml::from_str(&std::fs::read_to_string(guild_data_path)?)?,
		})
	}

	pub fn load_or_create_from_dir(dir_path: PathBuf) -> Self
	{
		let _ = std::fs::create_dir_all(&dir_path);
		Self::load_from_dir(dir_path.clone()).unwrap_or_else(|_| Self::new(dir_path))
	}

	pub fn guild_data(&self, guild_id: GuildId) -> Option<&GuildData>
	{
		self.guild_data.get(&guild_id)
	}
	pub fn guild_data_mut(&mut self, guild_id: GuildId) -> &mut GuildData
	{
		if self.unsynced
		{
			self.sync();
		}
		self.unsynced = true;

		self.guild_data.entry(guild_id).or_default()
	}
	pub fn sync(&mut self)
	{
		std::fs::write(
			self.dir_path.join("guild_data.toml"),
			toml::to_string_pretty(&self.guild_data)
				.unwrap_or_else(|err| panic!("Unable to serialize toml data! {err}")),
		)
		.expect("Unable to write to guild data file!");
		self.unsynced = false;
	}
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct GuildData
{
	#[serde(default)]
	autorole: Option<RoleId>,
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
}

#[derive(Debug, thiserror::Error)]
pub enum DataLoadError
{
	#[error("Could not load data file!")]
	IoError(#[from] std::io::Error),

	#[error("Could not parse data from file!")]
	TomlError(#[from] toml::de::Error),
}
