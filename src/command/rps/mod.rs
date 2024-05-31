mod leaderboard;
mod model;

use std::{fmt::Write, time::Duration};

use lazy_static::lazy_static;
use model::{ChallengerOpponentPair, Game, MatchOutcome, RoundOutcome, Selection, Side};
use poise::{
	serenity_prelude::{
		ButtonStyle, CreateActionRow, CreateAllowedMentions, CreateButton, CreateEmbed,
		CreateEmbedFooter, CreateMessage, GuildChannel, Member, Mentionable, Message, User,
	},
	CreateReply,
};
use rand::Rng;

use crate::{
	command::parent_command,
	data::{Leaderboard, Outcome, Score},
	Context, Error, Reply, Respond,
};

use super::ExpectGuildOnly;

parent_command! {
	let rps = poise::command(
		prefix_command,
		slash_command,
		guild_only,
		subcommands("challenge", "leaderboard::leaderboard")
	)
}

/// Challenge another user to a game of Rock, Paper, Scissors
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn challenge(
	ctx: Context<'_>,
	#[description = "Player to challenge"] opponent: Member,
	#[description = "The amount of games needed to win (default: 1)"] first_to: Option<u32>,
) -> Result<(), Error>
{
	if ctx.author().id == opponent.user.id
	{
		ctx.reply_error("You can't challenge yourself!").await?;
		Ok(())
	}
	else if opponent.user.id == ctx.framework().bot_id
	{
		start_bot_match(ctx, first_to.unwrap_or(1)).await
	}
	else if opponent.user.bot
	{
		ctx.reply_error("You can't challenge a bot!").await?;
		Ok(())
	}
	else
	{
		start_challenge(ctx, opponent, first_to.unwrap_or(1)).await
	}
}

async fn start_challenge(ctx: Context<'_>, opponent: Member, first_to: u32) -> Result<(), Error>
{
	let challenge_message =
		send_challenge_message(ctx, ctx.author(), &opponent.user, first_to).await?;

	if await_challenge_accept(ctx, &challenge_message, &opponent).await?
	{
		let guild_id = ctx.guild_id().expect_guild_only();
		// we fetch member through http instead of just passing the reference from the commands
		// so we can use the accent color later. -morgan 2024-01-18
		let members = ChallengerOpponentPair::new(
			ctx.http().get_member(guild_id, ctx.author().id).await?,
			ctx.http().get_member(guild_id, opponent.user.id).await?,
		);

		let mut game = Game::start(
			members.challenger.user.id,
			members.opponent.user.id,
			first_to,
		);

		let channel = ctx.guild_channel().await.expect_guild_only();
		// if none, selections timed out -morgan 2024-05-27
		if let Some(match_outcome) = start_game(ctx, &mut game, &members, &channel, false).await?
		{
			let rating_changes = update_leaderboard(
				ctx.data()
					.acquire_lock()
					.await
					.guild_data_mut(ctx.guild_id().expect_guild_only())
					.leaderboard_mut(),
				&match_outcome,
			);

			channel
				.send_message(
					ctx,
					CreateMessage::new().embed(create_match_embed(
						ctx,
						&match_outcome,
						&members,
						Some(rating_changes),
					)),
				)
				.await?;
		}
	}

	Ok(())
}

fn create_match_embed(
	ctx: Context<'_>,
	match_outcome: &MatchOutcome,
	members: &ChallengerOpponentPair<Member>,
	rating_changes: Option<ChallengerOpponentPair<(i32, i32)>>,
) -> CreateEmbed
{
	let winner = &members[match_outcome.winning_side()];

	let mut embed = CreateEmbed::new()
		.title("Game, Set, and Match")
		.description(format!("# {} wins!", winner.mention()))
		.field(
			"Score",
			format!(
				"{} - {}",
				match_outcome.challenger().score(),
				match_outcome.opponent().score()
			),
			true,
		)
		.color(
			winner
				.user
				.accent_colour
				.unwrap_or_else(|| winner.colour(ctx).unwrap_or(crate::DEFAULT_COLOR)),
		)
		.thumbnail(winner.face());

	if let Some(rating_changes) = rating_changes
	{
		embed = embed.field(
			"Ratings",
			{
				let mut ratings_string = String::new();
				rating_changes.for_each(|(old_elo, new_elo)| {
					let _ = writeln!(
						ratings_string,
						"{old_elo} → {new_elo} ({:+})",
						new_elo - old_elo
					);
				});
				ratings_string
			},
			true,
		);
	}

	embed
}

fn update_leaderboard(
	leaderboard: &mut Leaderboard,
	match_outcome: &MatchOutcome,
) -> ChallengerOpponentPair<(i32, i32)>
{
	let old_ratings = match_outcome.players.map_ref(|player| {
		leaderboard
			.score(player.id())
			.map_or(Score::BASE_ELO, |score| score.elo)
	});

	let new_ratings =
		match_outcome
			.players
			.as_ref()
			.zip(old_ratings.flip())
			.map(|(player, opponent_elo)| {
				leaderboard.score_mut(player.id()).update_elo(
					opponent_elo,
					Outcome::from(match_outcome.winner().id() == player.id()),
				)
			});

	leaderboard
		.score_mut(match_outcome.winner().id())
		.increment_wins();
	leaderboard
		.score_mut(match_outcome.loser().id())
		.increment_losses();

	old_ratings.zip(new_ratings)
}

async fn await_challenge_accept(
	ctx: Context<'_>,
	challenge_message: &Message,
	opponent: &Member,
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
			log::info!(
				"Rps challenge {}({}) v {}({}) timed out",
				ctx.author().name,
				ctx.author().id,
				opponent.user.name,
				opponent.user.id
			);
			break false;
		};

		if interaction.user.id != opponent.user.id
		{
			interaction
				.respond_ephemeral(
					ctx,
					crate::error_embed("Only the challenged user may accept or decline!"),
				)
				.await?;

			continue;
		}

		match interaction.data.custom_id.as_str()
		{
			"rps-accept" =>
			{
				interaction
					.respond(
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
				interaction
					.respond(
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

async fn send_challenge_message(
	ctx: Context<'_>,
	challenger: &User,
	opponent: &User,
	first_to: u32,
) -> Result<Message, Error>
{
	ctx.send(CreateReply::default()
		.content(opponent.mention().to_string())
		.embed(
			CreateEmbed::new()
				.title("Rock Paper Scissors")
				.description(format!(
					"{} challenges {} to a{} Rock, Paper, Scissors match!\n{}, do you accept?",
					challenger.mention(),
					opponent.mention(),
					(first_to > 1)
						.then(|| format!(" **first-to {first_to}**"))
						.unwrap_or_default(),
					opponent.mention()
				))
				.color(crate::DEFAULT_COLOR)
				.footer(CreateEmbedFooter::new(
					"\u{2757} Interctions will only be valid within an hour of this message being sent",
				)),
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
		.allowed_mentions(CreateAllowedMentions::new()))
	.await?
	.into_message()
	.await
	.map_err(Into::into)
}

async fn start_game(
	ctx: Context<'_>,
	game: &mut Game<Option<Selection>>,
	members: &ChallengerOpponentPair<Member>,
	channel: &GuildChannel,
	is_bot_match: bool,
) -> Result<Option<MatchOutcome>, Error>
{
	let match_outcome = loop
	{
		if is_bot_match
		{
			game[Side::Opponent].select(rand::thread_rng().gen());
		}
		let selection_message = channel
			.send_message(ctx, SELECTION_MESSAGE_TEMPLATE.clone())
			.await?;
		let Some(round_outcome) = await_selections(ctx, &selection_message, game).await?
		else
		{
			break None;
		};

		channel
			.send_message(
				ctx,
				CreateMessage::new().embed(round_outcome.winner_embed(members)),
			)
			.await?;

		if let Some(match_outcome) = round_outcome.try_delcare_match()
		{
			break Some(match_outcome);
		}
	};

	Ok(match_outcome)
}

async fn await_selections(
	ctx: Context<'_>,
	selection_message: &Message,
	game: &mut Game<Option<Selection>>,
) -> Result<Option<RoundOutcome>, Error>
{
	let outcome = loop
	{
		let Some(interaction) = selection_message
			.await_component_interaction(ctx)
			.timeout(Duration::from_secs(3600))
			.await
		else
		{
			log::info!(
				"Rps game {}({}) v {}({}) timed out",
				ctx.http().get_user(game[Side::Challenger].id()).await?.name,
				game[Side::Challenger].id(),
				ctx.http().get_user(game[Side::Opponent].id()).await?.name,
				game[Side::Opponent].id(),
			);
			break None;
		};

		let Some(side) = game.side_of(interaction.user.id)
		else
		{
			interaction
				.respond_ephemeral(
					ctx,
					crate::error_embed("Only the players in the game are allowed to respond!"),
				)
				.await?;
			continue;
		};

		if game[side].has_selected()
		{
			interaction
				.respond_ephemeral(ctx, crate::error_embed("You have already selected!"))
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

		interaction
			.respond_ephemeral(
				ctx,
				CreateEmbed::new()
					.title("Selection made!")
					.description(format!("You have selected {selection}"))
					.color(crate::DEFAULT_COLOR),
			)
			.await?;

		if let Some(round_outcome) = game.try_delcare_round()
		{
			break Some(round_outcome);
		}
	};

	Ok(outcome)
}

async fn start_bot_match(ctx: Context<'_>, first_to: u32) -> Result<(), Error>
{
	let guild_id = ctx.guild_id().expect_guild_only();

	let mut game = Game::start(ctx.author().id, ctx.framework().bot_id, first_to);
	let members = ChallengerOpponentPair::new(
		ctx.http()
			.get_member(guild_id, game.challenger().id())
			.await?,
		ctx.http()
			.get_member(guild_id, game.opponent().id())
			.await?,
	);
	let channel = ctx.guild_channel().await.expect_guild_only();

	ctx.send(
		CreateReply::default()
			.embed(
				CreateEmbed::new()
					.title("Challenge accepted!")
					.description(format!(
						"I accept {}'s challenge!",
						members.challenger.mention()
					))
					.color(crate::DEFAULT_COLOR),
			)
			.allowed_mentions(CreateAllowedMentions::new())
			.reply(true),
	)
	.await?;

	// a little wet but thats ok i for now i think -morgan 2024-05-30
	if let Some(match_outcome) = start_game(ctx, &mut game, &members, &channel, true).await?
	{
		channel
			.send_message(
				ctx,
				CreateMessage::new().embed(create_match_embed(ctx, &match_outcome, &members, None)),
			)
			.await?;
	}

	Ok(())
}

lazy_static! {
	static ref SELECTION_MESSAGE_TEMPLATE: CreateMessage = CreateMessage::new()
		.embed(
			CreateEmbed::new()
				.title("Make your selection!")
				.description("Pick rock, paper, or, scissors")
				.color(crate::DEFAULT_COLOR)
				.footer(CreateEmbedFooter::new(
					"\u{2757} Interctions will only be valid within an hour of this message being sent",
				)),
		)
		.components(vec![CreateActionRow::Buttons(
			Selection::map_all(Selection::button).collect(),
		)]);
}
