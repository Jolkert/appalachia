use std::{
	fmt::{Display, Write},
	time::Duration,
};

use poise::{
	serenity_prelude::{
		ButtonStyle, CreateActionRow, CreateAllowedMentions, CreateButton, CreateEmbed,
		CreateEmbedFooter, CreateMessage, Member, Mentionable, Message, UserId,
	},
	CreateReply,
};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, IntoStaticStr};

use crate::{
	data::{Outcome, Score},
	Context, Data, Error, Respond,
};

macro_rules! write_lb_line {
	($buffer:expr, $spacing:expr, $rank:expr, $user:expr, $elo:expr, $wins:expr, $losses:expr, $win_rate:expr) => {
		writeln!(
			$buffer,
			"{:<rank_wid$} │ {:<user_wid$} │ {:>elo_wid$} │ {:>win_wid$} │ {:>loss_wid$} │ {:>rate_wid$}",
			$rank,
			$user,
			$elo,
			$wins,
			$losses,
			$win_rate,
			rank_wid = $spacing.rank,
			user_wid = $spacing.name,
			elo_wid = $spacing.elo,
			win_wid = $spacing.wins,
			loss_wid = $spacing.losses,
			rate_wid = $spacing.winrate,
		)
	};
}

macro_rules! repeat {
	($n:expr, $code:block) => {
		for _ in 0..($n)
		{
			$code
		}
	};
}

// TODO: please rewrite all of this i beg you -morgan 2024-05-19

#[allow(clippy::unused_async)]
#[poise::command(
	prefix_command,
	slash_command,
	guild_only,
	subcommands("challenge", "leaderboard")
)]
pub async fn rps(_: Context<'_>) -> Result<(), Error>
{
	Ok(())
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
		ctx.send(
			CreateReply::default()
				.embed(crate::error_embed("You can't challenge yourself!"))
				.reply(true)
				.allowed_mentions(CreateAllowedMentions::new())
				.ephemeral(true),
		)
		.await?;
		return Ok(());
	}

	let first_to = first_to.unwrap_or(1);
	let reply = CreateReply::default()
		.content(opponent.mention().to_string())
		.embed(
			CreateEmbed::new()
				.title("Rock Paper Scissors")
				.description(format!(
					"{} challenges {} to a{} Rock, Paper, Scissors match!\n{}, do you accept?",
					ctx.author().mention(),
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
		.allowed_mentions(CreateAllowedMentions::new());

	let message = ctx.send(reply).await?.into_message().await?;
	if await_accept(ctx, &message, &opponent).await?
	{
		let game = process_selections(ctx, &opponent, first_to).await?;

		let mut data_lock = ctx.data().acquire_lock().await;
		let leaderboard = data_lock
			.guild_data_mut(ctx.guild_id().unwrap())
			.leaderboard_mut();

		leaderboard.score_mut(game.winner().id()).increment_wins();
		leaderboard.score_mut(game.loser().id()).increment_losses();
	}

	Ok(())
}

/// View the Rock, Paper, Scissors leaderboard for this server
#[allow(clippy::too_many_lines)] // this hurts -morgan 2024-05-20
#[poise::command(aliases("lb"), slash_command, prefix_command, guild_only)]
pub async fn leaderboard(
	ctx: Context<'_>,
	#[description = "Specify a user to see their specific score"] user: Option<Member>,
) -> Result<(), Error>
{
	let guild = ctx.partial_guild().await.unwrap();

	if let Some(target_member) = user
	{
		let target_member = ctx
			.http()
			.get_member(guild.id, target_member.user.id)
			.await?;

		if let Some(guild_data) = ctx.data().acquire_lock().await.guild_data(guild.id)
			&& let Some(score) = guild_data.leaderboard().score(target_member.user.id)
		{
			ctx.send(
				CreateReply::default()
					.embed(
						CreateEmbed::new()
							.title("Rock Paper Scissors Stats")
							.description(format!("# Stats for {}", target_member.mention()))
							.field(
								"Rank",
								format!(
									"#{} in {}",
									guild_data
										.leaderboard()
										.ordered_scores(None)
										.iter()
										.position(|(_, _, sc)| sc == &score)
										.unwrap() + 1,
									guild.name
								),
								true,
							)
							.field("Rating", score.elo.to_string(), false)
							.field("Wins", score.wins.to_string(), true)
							.field("Losses", score.losses.to_string(), true)
							.field(
								"Win Rate",
								format!("{:.2}%", score.win_rate() * 100.0),
								true,
							)
							.color(
								target_member
									.user
									.accent_colour
									.unwrap_or(crate::DEFAULT_COLOR),
							)
							.thumbnail(target_member.face()),
					)
					.reply(true)
					.allowed_mentions(CreateAllowedMentions::new())
					.ephemeral(false),
			)
			.await?;
		}
		else
		{
			ctx.send(
				CreateReply::default()
					.embed(crate::error_embed(format!(
						"{} has no rock paper scissors scores!",
						target_member.mention()
					)))
					.reply(true)
					.allowed_mentions(CreateAllowedMentions::new())
					.ephemeral(true),
			)
			.await?;
		}
	}
	else if let Some(id_scores) = ctx
		.data()
		.acquire_lock()
		.await
		.guild_data(guild.id)
		.map(|dat| dat.leaderboard().ordered_scores(Some(15)))
		&& !id_scores.is_empty()
	{
		// no scoped threads :( i dont wanna install crossbeam just for this -morgan 2024-05-20
		let mut scores = Vec::with_capacity(id_scores.len());
		for (id, rank, score) in id_scores
		{
			scores.push((guild.member(ctx.http(), id).await?, rank, score));
		}

		let string_lengths = get_max_lengths(&scores).cap_name_at(25);

		let mut leaderboard_string = String::from("```");

		let _ = write_lb_line!(
			leaderboard_string,
			string_lengths,
			"#",
			"USER",
			"ELO",
			"W",
			"L",
			"WRATE"
		);

		leaderboard_string.push_str(&string_lengths.draw_line('═', '╪'));

		let mut top_3_line_drawn = false;
		for (member, rank, score) in scores
		{
			if !top_3_line_drawn && rank > 3
			{
				leaderboard_string.push_str(&string_lengths.draw_line('┄', '┼'));
				top_3_line_drawn = true;
			}

			let _ = write_lb_line!(
				leaderboard_string,
				string_lengths,
				rank,
				member.display_name(),
				score.elo,
				score.wins,
				score.losses,
				format!("{:.4}", score.win_rate()).trim_start_matches('0')
			);
		}

		leaderboard_string.push_str("```");

		let mut embed = CreateEmbed::new()
			.title("Rock Paper Scissors Leaderboard")
			.description(leaderboard_string)
			.color(crate::DEFAULT_COLOR);

		if let Some(guild_icon) = guild.icon_url()
		{
			embed = embed.thumbnail(guild_icon);
		}

		ctx.send(
			CreateReply::default()
				.embed(embed)
				.reply(true)
				.allowed_mentions(CreateAllowedMentions::new()),
		)
		.await?;
	}
	else
	{
		ctx.send(
			CreateReply::default()
				.embed(crate::error_embed("No leaderboard exists for this server!"))
				.reply(true)
				.allowed_mentions(CreateAllowedMentions::new())
				.ephemeral(true),
		)
		.await?;
	}

	Ok(())
}

fn get_max_lengths(scores: &[(Member, u32, &Score)]) -> StringLengths
{
	let mut lengths = StringLengths::default();
	for (member, rank, score) in scores
	{
		lengths.set_name(member.display_name());
		lengths.set_rank(*rank);
		lengths.set_elo(score.elo);
		lengths.set_losses(score.losses);
		lengths.set_wins(score.wins);
		lengths.set_winrate(score.win_rate());
	}

	lengths
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct StringLengths
{
	pub rank: usize,
	pub name: usize,
	pub elo: usize,
	pub wins: usize,
	pub losses: usize,
	pub winrate: usize,
}
#[allow(clippy::cast_sign_loss)]
impl StringLengths
{
	pub fn set_rank(&mut self, rank: u32)
	{
		let new = (rank / 10) as usize + 1;
		if new > self.rank
		{
			self.rank = new;
		}
	}

	pub fn cap_name_at(mut self, max_len: usize) -> Self
	{
		self.name = usize::min(self.name, max_len);
		self
	}
	pub fn set_name(&mut self, name: &str)
	{
		let new = name.len();
		if new > self.name
		{
			self.name = new;
		}
	}

	pub fn set_elo(&mut self, elo: i32)
	{
		let new = f64::from(elo.abs()).log10().floor() as usize + 1;
		if new > self.elo
		{
			self.elo = new;
		}
	}

	pub fn set_wins(&mut self, wins: u32)
	{
		let new = f64::from(wins).log10().floor() as usize + 1;
		if new > self.wins
		{
			self.wins = new;
		}
	}
	pub fn set_losses(&mut self, losses: u32)
	{
		let new = f64::from(losses).log10().floor() as usize + 1;
		if new > self.losses
		{
			self.losses = new;
		}
	}

	pub fn set_winrate(&mut self, winrate: f64)
	{
		// one for each digit, one for the decimal point, and rounded to 4 places;
		// in theory, this number should never exceed 6 but just in case -morgan 2024-05-20
		let new = (winrate.abs().log10().floor() as usize) + 5;
		if new > self.winrate
		{
			self.winrate = new;
		}
	}

	pub fn draw_line(&self, horizontal: char, vertical: char) -> String
	{
		// ive done my absolute best to not make this the worst thing ever -morgan 2024-05-20
		// ok i think its a bit better now? not by a ton but stil -morgan 2024-05-21
		self.spacings_iterator()
			.enumerate()
			.fold(String::new(), |mut line_string, (i, times)| {
				repeat!(times + 1 + usize::from((1..=5).contains(&i)), {
					line_string.push(horizontal);
				});
				line_string.push(vertical);
				line_string
			}) + "\n"
	}

	fn spacings_iterator(&self) -> impl Iterator<Item = usize>
	{
		[
			self.rank,
			self.name,
			self.elo,
			self.wins,
			self.losses,
			self.winrate,
		]
		.into_iter()
	}
}

async fn await_accept(
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
			continue;
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

async fn process_selections(
	ctx: Context<'_>,
	opponent: &Member,
	first_to: u32,
) -> Result<Game, Error>
{
	let channel = ctx.guild_channel().await.unwrap();
	let message_template = CreateMessage::new()
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
			Selection::map(Selection::button).collect(),
		)]);

	// we fetch member through http instead of just passing the reference from the commands so we
	// can use the accent color later.  2024-01-18
	let guild_id = ctx.guild_id().unwrap();
	let mut game = Game::new(
		Player::new(ctx.http().get_member(guild_id, ctx.author().id).await?),
		Player::new(ctx.http().get_member(guild_id, opponent.user.id).await?),
		first_to,
	);

	while game.highest_score() < game.first_to
	{
		let selection_message = channel.send_message(ctx, message_template.clone()).await?;
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

		if let Some(winner) = winning_side
		{
			game[winner].increment_score();
		}

		channel
			.send_message(
				ctx,
				CreateMessage::new().embed(
					game.winner_embed(winning_side, selections, ctx.data())
						.await,
				),
			)
			.await?;

		game.next_round();
	}

	Ok(game)
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
			interaction
				.respond_ephemeral(
					ctx,
					crate::error_embed("Only the person who was challenged is allowed to respond!"),
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

		if let (Some(challenger), Some(opponent)) = game.selections()
		{
			break Ok((challenger, opponent));
		}
	}
}

struct Game
{
	challenger: Player,
	opponent: Player,
	first_to: u32,
	round_count: u32,
}
impl Game
{
	fn new(challenger: Player, opponent: Player, first_to: u32) -> Self
	{
		Self {
			challenger,
			opponent,
			first_to,
			round_count: 1,
		}
	}

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
		if id == self.challenger.id()
		{
			Some(Side::Challenger)
		}
		else if id == self.opponent.id()
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

	fn highest_score(&self) -> u32
	{
		u32::max(self.challenger.score, self.opponent.score)
	}

	fn has_winner(&self) -> bool
	{
		self.highest_score() >= self.first_to
	}

	fn next_round(&mut self)
	{
		self.challenger.selection = None;
		self.opponent.selection = None;
		self.round_count += 1;
	}

	fn winner(&self) -> &Player
	{
		if self.highest_score() == self.challenger.score
		{
			&self.challenger
		}
		else
		{
			&self.opponent
		}
	}
	fn loser(&self) -> &Player
	{
		if self.highest_score() == self.challenger.score
		{
			&self.opponent
		}
		else
		{
			&self.challenger
		}
	}

	#[allow(clippy::too_many_lines)] // i agree but i really cant deal with that rn -morgan 2024-05-19
	async fn winner_embed(
		&self,
		winning_side: Option<Side>,
		selections: (Selection, Selection),
		data: &Data,
	) -> CreateEmbed
	{
		let is_declared = self.has_winner();
		let title = is_declared
			.then(|| String::from("Game, set, and match!"))
			.unwrap_or_else(|| format!("Round {}", self.round_count));

		let (mut embed, rating_changes) = if let Some(winner) =
			winning_side.map(|side| self.get_player(side))
		{
			let rating_changes = if is_declared
			{
				// this *really* shouldn't be happening in here -morgan 2024-05-19
				let mut data_lock = data.acquire_lock().await;
				let leaderboard = data_lock
					.guild_data_mut(winner.member().guild_id)
					.leaderboard_mut();

				let (old_challenger_elo, old_opponent_elo) = (
					leaderboard
						.score(self.challenger.id())
						.map_or(Score::BASE_ELO, |score| score.elo),
					leaderboard
						.score(self.opponent.id())
						.map_or(Score::BASE_ELO, |score| score.elo),
				);

				let challenger_outcome = Outcome::from(self.challenger.id() == winner.id());
				let new_challenger_elo = leaderboard
					.score_mut(self.challenger.id())
					.update_elo(old_opponent_elo, challenger_outcome);

				let new_opponent_elo = leaderboard
					.score_mut(self.opponent.id())
					.update_elo(old_challenger_elo, !challenger_outcome);

				let (challenger_diff, opponent_diff) = (
					new_challenger_elo - old_challenger_elo,
					new_opponent_elo - old_opponent_elo,
				);

				let mut rating_str = String::new();
				let _ = writeln!(
					rating_str,
					"\n{} {old_challenger_elo} → {new_challenger_elo} ({:+})",
					self.challenger.member().mention(),
					challenger_diff
				);
				let _ = writeln!(
					rating_str,
					"{} {old_opponent_elo} → {new_opponent_elo} ({:+})",
					self.opponent.member().mention(),
					opponent_diff
				);
				rating_str
			}
			else
			{
				String::default()
			};

			(
				CreateEmbed::new()
					.description(format!(
						"# {} wins{}!",
						winner.member.mention(),
						(is_declared && self.first_to > 1)
							.then_some(" the set")
							.unwrap_or_default(),
					))
					.color(
						winner
							.member
							.user
							.accent_colour
							.unwrap_or(crate::DEFAULT_COLOR),
					)
					.thumbnail(winner.member.face()),
				rating_changes,
			)
		}
		else
		{
			(
				CreateEmbed::new()
					.description("# It's a tie!")
					.color(crate::DEFAULT_COLOR),
				String::default(),
			)
		};

		embed = embed
			.title(title)
			.field(
				"Selections",
				format!(
					"{} chose {}\n{} chose {}",
					self.challenger.member.mention(),
					selections.0,
					self.opponent.member.mention(),
					selections.1,
				),
				false,
			)
			.field(
				"Score",
				format!("{}-{}", self.challenger.score, self.opponent.score),
				true,
			);

		if is_declared
		{
			embed = embed.field("Rating", rating_changes, true);
		}

		embed
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
	member: Member,
	selection: Option<Selection>,
	score: u32,
}
impl Player
{
	fn new(member: Member) -> Self
	{
		Self {
			member,
			selection: None,
			score: 0,
		}
	}

	fn id(&self) -> UserId
	{
		self.member.user.id
	}

	fn select(&mut self, selection: Selection)
	{
		self.selection = Some(selection);
	}

	fn has_selected(&self) -> bool
	{
		self.selection.is_some()
	}

	fn increment_score(&mut self)
	{
		self.score += 1;
	}

	fn member(&self) -> &Member
	{
		&self.member
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side
{
	Challenger,
	Opponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr, EnumIter)]
enum Selection
{
	Rock,
	Paper,
	Scissors,
}
impl Selection
{
	fn map<T>(f: impl FnMut(Self) -> T) -> impl Iterator<Item = T>
	{
		Self::iter().map(f)
	}

	fn emoji(self) -> char
	{
		match self
		{
			Self::Rock => '\u{270a}',
			Self::Paper => '\u{1f590}',
			Self::Scissors => '\u{270c}',
		}
	}

	fn as_str(self) -> &'static str
	{
		self.into()
	}

	fn button(self) -> CreateButton
	{
		CreateButton::new(self.as_str().to_lowercase())
			.label(self.as_str())
			.emoji(self.emoji())
	}
}
impl Display for Selection
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		write!(f, "{} {}", (*self).as_str(), (*self).emoji())
	}
}
