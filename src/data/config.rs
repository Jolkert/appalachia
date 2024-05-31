use std::path::{Path, PathBuf};

use crate::Error;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config
{
	#[serde(default)]
	pub token: String,

	#[serde(default = "Config::default_prefix")]
	pub prefix: String,

	#[serde(default = "Config::default_data_dir")]
	pub data_directory: PathBuf,

	#[serde(default)]
	pub status: Option<String>,
}
impl Default for Config
{
	fn default() -> Self
	{
		Self {
			token: String::default(),
			prefix: Self::default_prefix(),
			data_directory: Self::default_data_dir(),
			status: None,
		}
	}
}
impl Config
{
	pub fn load(path: impl AsRef<Path>) -> Result<Self, Error>
	{
		let config_file_path = Self::root_dir().join(path);

		match std::fs::read_to_string(&config_file_path)
		{
			Ok(file_content) => toml::from_str(&file_content).map_err(Into::into),
			Err(err) if err.kind() == std::io::ErrorKind::NotFound =>
			{
				let config = Self::default();
				std::fs::write(config_file_path, toml::to_string_pretty(&config)?)?;
				Ok(config)
			}
			Err(err) => Err(Box::new(err)),
		}
	}

	fn default_prefix() -> String
	{
		String::from("~")
	}
	fn default_data_dir() -> PathBuf
	{
		Self::root_dir().join("data/")
	}

	#[cfg(debug_assertions)]
	fn root_dir() -> PathBuf
	{
		PathBuf::from("./")
	}

	#[cfg(not(debug_assertions))]
	fn root_dir() -> PathBuf
	{
		let dir = dirs::config_dir().map_or_else(
			|| PathBuf::from("./"),
			|config_dir| config_dir.join("appalachia/"),
		);
		std::fs::create_dir_all(&dir).expect("Failed to create config directory!");
		dir
	}
}
