use std::{fmt::Display, time::Duration};

use poise::{
	serenity_prelude::{
		ButtonStyle, ComponentInteraction, CreateActionRow, CreateAllowedMentions, CreateButton,
		CreateEmbed, CreateInteractionResponse, CreateInteractionResponseFollowup,
		CreateInteractionResponseMessage, CreateMessage, Mentionable, Message, User, UserId,
	},
	CreateReply,
};

use crate::{Context, Error};

/// Challenge another user to a game of Rock, Paper, Scissors
#[poise::command(slash_command, prefix_command)]
pub async fn rps(
	ctx: Context<'_>,
	#[description = "Player to challenge"] opponent: User,
	#[description = "The amount of games needed to win (defaults to 1)"] first_to: Option<u32>,
) -> Result<(), Error>
{
	let first_to = first_to.unwrap_or(1);

	let reply = CreateReply::default()
		.content(opponent.mention().to_string())
		.embed(
			CreateEmbed::new()
				.title("Rock Paper Scissors")
				.description(format!(
					"{} challenges {} to a Rock, Paper, Scissors match!\n{}, do you accept?",
					ctx.author().mention(),
					opponent.mention(),
					opponent.mention()
				))
				.color(crate::DEFAULT_COLOR),
		)
		.components(vec![CreateActionRow::Buttons(vec![
			CreateButton::new("rps-accept")
				.emoji('\u{1f44d}')
				.label("Accept")
				.style(ButtonStyle::Success),
			CreateButton::new("rps-decline")
				.emoji('\u{1f44e}')
				.label("Decline")
				.style(ButtonStyle::Danger),
		])])
		.reply(true)
		.allowed_mentions(CreateAllowedMentions::new());

	let message = ctx.send(reply).await?.into_message().await?;
	if await_accept(ctx, &message, &opponent).await?
	{
		process_selections(ctx, &opponent).await?;
	}

	Ok(())
}

async fn await_accept(
	ctx: Context<'_>,
	challenge_message: &Message,
	opponent: &User,
) -> Result<bool, Error>
{
	let accepted = loop
	{
		let Some(interaction) = challenge_message
			.await_component_interaction(ctx)
			.timeout(Duration::from_secs(3600))
			.await
		else
		{
			continue;
		};

		if interaction.user.id != opponent.id
		{
			respond_ephemeral(
				interaction,
				ctx,
				CreateEmbed::new()
					.title("Error")
					.description("Only the challenged user may accept or decline!")
					.color(crate::ERROR_COLOR),
			)
			.await?;

			continue;
		}

		match interaction.data.custom_id.as_str()
		{
			"rps-accept" =>
			{
				respond_to(
					interaction,
					ctx,
					CreateEmbed::new()
						.title("Challenge accepted!")
						.description(format!("{} accepts the challenge!", opponent.mention()))
						.color(crate::DEFAULT_COLOR),
				)
				.await?;

				break true;
			}
			"rps-deny" =>
			{
				respond_to(
					interaction,
					ctx,
					CreateEmbed::new()
						.title("Challenge declined!")
						.description(format!(
							"{} does not accept the challenge",
							opponent.mention()
						))
						.color(crate::DEFAULT_COLOR),
				)
				.await?;

				break false;
			}
			_ => continue,
		}
	};
	Ok(accepted)
}

async fn process_selections(ctx: Context<'_>, opponent: &User) -> Result<(), Error>
{
	// send selection message
	let rps_buttons = vec![
		CreateButton::new("rock").label("rock").emoji('\u{270a}'),
		CreateButton::new("paper").label("paper").emoji('\u{1f590}'),
		CreateButton::new("scissors")
			.label("scissors")
			.emoji('\u{270c}'),
	];

	let channel = ctx.guild_channel().await.unwrap();
	let selection_message = channel
		.send_message(
			ctx,
			CreateMessage::new()
				.embed(
					CreateEmbed::new()
						.title("Make your selection!")
						.description("Pick rock, paper, or, scissors")
						.color(crate::DEFAULT_COLOR),
				)
				.components(vec![CreateActionRow::Buttons(rps_buttons)]),
		)
		.await?;

	// start game
	// we fetch user through http instead of just passing the reference from the commands so we can
	// use the accent color later. Side effect of that is that apparently Http::get_user returns an
	// owned User? Thats weird -morgan 2024-01-18
	let mut game = Game {
		challenger: Player::new(ctx.http().get_user(ctx.author().id).await?),
		opponent: Player::new(ctx.http().get_user(opponent.id).await?),
	};

	let selections = await_selections(ctx, &selection_message, &mut game).await?;
	let winning_side = match selections
	{
		(Selection::Rock, Selection::Scissors)
		| (Selection::Paper, Selection::Rock)
		| (Selection::Scissors, Selection::Paper) => Some(Side::Challenger),

		(Selection::Rock, Selection::Paper)
		| (Selection::Paper, Selection::Scissors)
		| (Selection::Scissors, Selection::Rock) => Some(Side::Opponent),

		(Selection::Rock, Selection::Rock)
		| (Selection::Paper, Selection::Paper)
		| (Selection::Scissors, Selection::Scissors) => None,
	};

	let results_embed = if let Some(winner) = winning_side.map(|it| game.get_player(it))
	{
		let mut embed = CreateEmbed::new()
			.title("Rock Paper Scissors")
			.description(format!(
				"# {} wins!\n{} chose {}\n{} chose {}",
				winner.user.mention(),
				game.challenger.user.mention(),
				selections.0,
				game.opponent.user.mention(),
				selections.1
			))
			.color(winner.user.accent_colour.unwrap_or(crate::DEFAULT_COLOR));
		if let Some(avatar) = winner.user.avatar_url()
		{
			embed = embed.thumbnail(avatar);
		}

		embed
	}
	else
	{
		CreateEmbed::new()
			.title("Rock Paper Scissors")
			.description(format!(
				"# It's a tie!\n{} chose {}\n{} chose {}",
				game.challenger.user.mention(),
				selections.0,
				game.opponent.user.mention(),
				selections.1
			))
			.color(crate::DEFAULT_COLOR)
	};

	channel
		.send_message(ctx, CreateMessage::new().embed(results_embed))
		.await?;

	Ok(())
}

async fn await_selections(
	ctx: Context<'_>,
	selection_message: &Message,
	game: &mut Game,
) -> Result<(Selection, Selection), Error>
{
	loop
	{
		let Some(interaction) = selection_message
			.await_component_interaction(ctx)
			.timeout(Duration::from_secs(3600))
			.await
		else
		{
			continue;
		};

		let Some(side) = game.side_by_id(interaction.user.id)
		else
		{
			interaction.defer_ephemeral(&ctx).await?;
			interaction
				.create_followup(
					&ctx,
					CreateInteractionResponseFollowup::new()
						.content("Only the person who was challenged is allowed to respond!"),
				)
				.await?;
			continue;
		};

		if game[side].has_selected()
		{
			respond_ephemeral(
				interaction,
				ctx,
				CreateEmbed::new()
					.title("Error")
					.description("You have already selected!")
					.color(crate::ERROR_COLOR),
			)
			.await?;
			continue;
		}

		let selection = match interaction.data.custom_id.as_str()
		{
			"rock" => Selection::Rock,
			"paper" => Selection::Paper,
			"scissors" => Selection::Scissors,
			_ => continue, // this should never happen? -morgan 2024-01-18
		};
		game[side].select(selection);

		respond_ephemeral(
			interaction,
			ctx,
			CreateEmbed::new()
				.title("Selection made!")
				.description(format!("You have selected {selection}")),
		)
		.await?;

		if let (Some(challenger), Some(opponent)) = game.selections()
		{
			break Ok((challenger, opponent));
		}
	}
}

async fn respond_to(
	interaction: ComponentInteraction,
	ctx: Context<'_>,
	embed: CreateEmbed,
) -> Result<(), Error>
{
	interaction
		.create_response(
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

async fn respond_ephemeral(
	interaction: ComponentInteraction,
	ctx: Context<'_>,
	embed: CreateEmbed,
) -> Result<(), Error>
{
	interaction.defer_ephemeral(ctx).await?;
	interaction
		.create_followup(
			ctx,
			CreateInteractionResponseFollowup::new()
				.embed(embed)
				.allowed_mentions(CreateAllowedMentions::new()),
		)
		.await?;

	Ok(())
}

struct Game
{
	challenger: Player,
	opponent: Player,
}
impl Game
{
	fn get_player(&self, side: Side) -> &Player
	{
		match side
		{
			Side::Challenger => &self.challenger,
			Side::Opponent => &self.opponent,
		}
	}

	fn side_by_id(&self, id: UserId) -> Option<Side>
	{
		if id == self.challenger.user.id
		{
			Some(Side::Challenger)
		}
		else if id == self.opponent.user.id
		{
			Some(Side::Opponent)
		}
		else
		{
			None
		}
	}
	fn selections(&self) -> (Option<Selection>, Option<Selection>)
	{
		(self.challenger.selection, self.opponent.selection)
	}
}
impl std::ops::Index<Side> for Game
{
	type Output = Player;

	fn index(&self, index: Side) -> &Self::Output
	{
		match index
		{
			Side::Challenger => &self.challenger,
			Side::Opponent => &self.opponent,
		}
	}
}
impl std::ops::IndexMut<Side> for Game
{
	fn index_mut(&mut self, index: Side) -> &mut Self::Output
	{
		match index
		{
			Side::Challenger => &mut self.challenger,
			Side::Opponent => &mut self.opponent,
		}
	}
}

#[derive(Debug)]
struct Player
{
	user: User,
	selection: Option<Selection>,
}
impl Player
{
	fn new(user: User) -> Self
	{
		Self {
			user,
			selection: None,
		}
	}

	fn select(&mut self, selection: Selection)
	{
		self.selection = Some(selection);
	}

	fn has_selected(&self) -> bool
	{
		self.selection.is_some()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side
{
	Challenger,
	Opponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Selection
{
	Rock,
	Paper,
	Scissors,
}
impl Selection
{
	fn as_str(self) -> &'static str
	{
		match self
		{
			Self::Rock => "Rock \u{270a}",
			Self::Paper => "Paper \u{1f590}",
			Self::Scissors => "Scissors \u{270c}",
		}
	}
}
impl Display for Selection
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		write!(f, "{}", (*self).as_str())
	}
}
