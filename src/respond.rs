// i like it better >:( -morgan 2024-05-31
#![allow(async_fn_in_trait)]

use poise::{
	serenity_prelude::{
		ComponentInteraction, CreateAllowedMentions, CreateEmbed, CreateEmbedFooter,
		CreateInteractionResponse, CreateInteractionResponseMessage,
	},
	CreateReply,
};

use crate::{Context, Error};

pub trait Respond
{
	async fn respond(self, ctx: Context<'_>, embed: CreateEmbed) -> Result<(), Error>;
	async fn respond_ephemeral(self, ctx: Context<'_>, embed: CreateEmbed) -> Result<(), Error>;
}
impl Respond for ComponentInteraction
{
	async fn respond(self, ctx: Context<'_>, embed: CreateEmbed) -> Result<(), Error>
	{
		self.create_response(
			ctx,
			CreateInteractionResponse::Message(
				CreateInteractionResponseMessage::new()
					.embed(embed)
					.allowed_mentions(CreateAllowedMentions::new()),
			),
		)
		.await?;
		Ok(())
	}

	async fn respond_ephemeral(self, ctx: Context<'_>, embed: CreateEmbed) -> Result<(), Error>
	{
		self.create_response(
			ctx,
			CreateInteractionResponse::Message(
				CreateInteractionResponseMessage::new()
					.embed(embed)
					.allowed_mentions(CreateAllowedMentions::new())
					.ephemeral(true),
			),
		)
		.await?;
		Ok(())
	}
}

pub trait Reply
{
	// i know there's a reason i didn't make this impl Into<String> before because i remember being
	// sad that i couldn't do it. but now it compiles just fine? im really not sure what the issue
	// was before but it seems to work so im leaving it here cause its a lot better -morgan
	// 2024-05-31
	async fn reply_error(self, error_text: impl Into<String> + Send + Sync) -> Result<(), Error>;
}
impl Reply for Context<'_>
{
	async fn reply_error(self, error_text: impl Into<String> + Send + Sync) -> Result<(), Error>
	{
		self.send(
			CreateReply::default()
				.embed(crate::error_embed(error_text))
				.reply(true)
				.allowed_mentions(CreateAllowedMentions::new())
				.ephemeral(true),
		)
		.await?;

		Ok(())
	}
}

pub fn error_embed(description: impl Into<String>) -> CreateEmbed
{
	CreateEmbed::new()
		.title("Error")
		.description(description)
		.color(crate::ERROR_COLOR)
		.footer(
			CreateEmbedFooter::new("If you think this is a bug, contact my mama, Jolkert!")
				.icon_url("https://jolkert.dev/img/icon_small.png"),
		)
}
