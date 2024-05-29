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

// im not really sure how i feel about this syntax? Maybe reconsider it at some point -morgan 2024-05-29
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
