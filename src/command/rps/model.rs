use std::{cmp::Ordering, fmt::Display};

use poise::serenity_prelude::{CreateButton, CreateEmbed, Member, Mentionable, UserId};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, IntoStaticStr};

pub type RoundOutcome = Game<Selection>;

#[derive(Debug, Clone)]
pub struct Game<S = Option<Selection>>
{
	players: ChallengerOpponentPair<Player<S>>,
	first_to: u32,
	round_count: u32,
}
impl<S> Game<S>
{
	pub fn challenger(&self) -> &Player<S>
	{
		&self.players.challenger
	}
	pub fn opponent(&self) -> &Player<S>
	{
		&self.players.opponent
	}

	pub fn side_of(&self, id: UserId) -> Option<Side>
	{
		if id == self.challenger().id()
		{
			Some(Side::Challenger)
		}
		else if id == self.opponent().id()
		{
			Some(Side::Opponent)
		}
		else
		{
			None
		}
	}

	pub fn current_winner(&self) -> Option<Side>
	{
		match self.challenger().score.cmp(&self.opponent().score)
		{
			Ordering::Greater => Some(Side::Challenger),
			Ordering::Less => Some(Side::Opponent),
			Ordering::Equal => None,
		}
	}
}
impl<S: Copy> Game<S>
{
	pub fn try_delcare_match(&self) -> Option<MatchOutcome>
	{
		self.current_winner()
			.is_some_and(|winner| self[winner].score >= self.first_to)
			.then(|| MatchOutcome::from_game(self))
	}
}
impl Game
{
	pub fn start(challenger: UserId, opponent: UserId, first_to: u32) -> Self
	{
		Self {
			players: ChallengerOpponentPair::generate(challenger, opponent, Player::new),
			// challenger: Player::new(challenger),
			// opponent: Player::new(opponent),
			first_to,
			round_count: 1,
		}
	}

	pub fn try_delcare_round(&mut self) -> Option<RoundOutcome>
	{
		self.players
			.as_ref()
			.map_ref(|player| player.selection)
			.zipped()
			.map(|(challenger_sel, opponent_sel)| {
				let mut outcome = Game {
					players: self.players.clone().gen_map(
						challenger_sel,
						opponent_sel,
						Player::with_selection,
					),
					first_to: self.first_to,
					round_count: self.round_count,
				};

				self.round_count += 1;
				self.players.challenger.selection = None;
				self.players.opponent.selection = None;
				if let Some(winner) = outcome.winner()
				{
					self[winner].increment_score();
					outcome[winner].increment_score();
				}

				outcome
			})
	}
}
impl RoundOutcome
{
	pub fn winner(&self) -> Option<Side>
	{
		match self.players.map_ref(|player| *player.selection()).tuple()
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
		}
	}

	pub fn winner_embed(&self, members: &ChallengerOpponentPair<Member>) -> CreateEmbed
	{
		self.winner()
			.map_or_else(
				|| {
					CreateEmbed::new()
						.description("# It's a tie!")
						.color(crate::DEFAULT_COLOR)
				},
				|winning_side| {
					let winner = &members[winning_side];
					CreateEmbed::new()
						.description(format!("# {} wins!", winner.mention(),))
						.color(winner.user.accent_colour.unwrap_or(crate::DEFAULT_COLOR))
						.thumbnail(winner.face())
				},
			)
			.title(format!("Round {}", self.round_count))
			.field(
				"Selections",
				format!(
					"{} chose {}\n{} chose {}",
					members.challenger.mention(),
					self.challenger().selection,
					members.opponent.mention(),
					self.opponent().selection,
				),
				false,
			)
			.field(
				"Score",
				format!("{}-{}", self.challenger().score, self.opponent().score),
				true,
			)
	}
}

impl<S> std::ops::Index<Side> for Game<S>
{
	type Output = Player<S>;

	fn index(&self, index: Side) -> &Self::Output
	{
		match index
		{
			Side::Challenger => &self.players.challenger,
			Side::Opponent => &self.players.opponent,
		}
	}
}
impl<S> std::ops::IndexMut<Side> for Game<S>
{
	fn index_mut(&mut self, index: Side) -> &mut Self::Output
	{
		match index
		{
			Side::Challenger => &mut self.players.challenger,
			Side::Opponent => &mut self.players.opponent,
		}
	}
}

pub struct MatchOutcome
{
	pub players: ChallengerOpponentPair<Player<()>>,
}
impl MatchOutcome
{
	fn from_game<S: Copy>(game: &Game<S>) -> Self
	{
		Self {
			players: game.players.map_ref(|player| player.clone().map_to(())),
		}
	}

	pub fn challenger(&self) -> &Player<()>
	{
		&self.players.challenger
	}
	pub fn opponent(&self) -> &Player<()>
	{
		&self.players.opponent
	}

	pub fn winning_side(&self) -> Side
	{
		if self.challenger().score > self.opponent().score
		{
			Side::Challenger
		}
		else
		{
			Side::Opponent
		}
	}
	pub fn losing_side(&self) -> Side
	{
		!self.winning_side()
	}

	pub fn winner(&self) -> &Player<()>
	{
		&self.players[self.winning_side()]
	}
	pub fn loser(&self) -> &Player<()>
	{
		&self.players[self.losing_side()]
	}
}

#[derive(Debug, Clone, Copy)]
pub struct ChallengerOpponentPair<T>
{
	pub challenger: T,
	pub opponent: T,
}
impl<T> ChallengerOpponentPair<T>
{
	pub fn new(challenger: T, opponent: T) -> Self
	{
		Self {
			challenger,
			opponent,
		}
	}

	pub fn tuple(self) -> (T, T)
	{
		(self.challenger, self.opponent)
	}

	pub fn generate<U>(challenger_arg: U, opponent_arg: U, generator: impl Fn(U) -> T) -> Self
	{
		Self::new(generator(challenger_arg), generator(opponent_arg))
	}

	pub fn map<U>(self, mut transform: impl FnMut(T) -> U) -> ChallengerOpponentPair<U>
	{
		ChallengerOpponentPair::new(transform(self.challenger), transform(self.opponent))
	}

	pub fn map_ref<U>(&self, transform: impl Fn(&T) -> U) -> ChallengerOpponentPair<U>
	{
		ChallengerOpponentPair::new(transform(&self.challenger), transform(&self.opponent))
	}

	pub fn gen_map<U, V>(
		self,
		challenger_arg: V,
		opponent_arg: V,
		mut generator: impl FnMut(T, V) -> U,
	) -> ChallengerOpponentPair<U>
	{
		ChallengerOpponentPair::new(
			generator(self.challenger, challenger_arg),
			generator(self.opponent, opponent_arg),
		)
	}

	pub fn for_each(self, mut func: impl FnMut(T))
	{
		func(self.challenger);
		func(self.opponent);
	}

	pub fn zip<U>(self, other: ChallengerOpponentPair<U>) -> ChallengerOpponentPair<(T, U)>
	{
		ChallengerOpponentPair::new(
			(self.challenger, other.challenger),
			(self.opponent, other.opponent),
		)
	}

	pub fn as_ref(&self) -> ChallengerOpponentPair<&T>
	{
		ChallengerOpponentPair::new(&self.challenger, &self.opponent)
	}

	pub fn flip(self) -> Self
	{
		Self::new(self.opponent, self.challenger)
	}
}
impl<T> ChallengerOpponentPair<Option<T>>
{
	pub fn zipped(self) -> Option<(T, T)>
	{
		self.challenger.zip(self.opponent)
	}
}
impl<T> std::ops::Index<Side> for ChallengerOpponentPair<T>
{
	type Output = T;

	fn index(&self, index: Side) -> &Self::Output
	{
		match index
		{
			Side::Challenger => &self.challenger,
			Side::Opponent => &self.opponent,
		}
	}
}
impl<T, U> std::ops::Sub<ChallengerOpponentPair<U>> for ChallengerOpponentPair<T>
where
	T: std::ops::Sub<U>,
{
	type Output = ChallengerOpponentPair<T::Output>;

	fn sub(self, rhs: ChallengerOpponentPair<U>) -> Self::Output
	{
		ChallengerOpponentPair::new(
			self.challenger - rhs.challenger,
			self.opponent - rhs.opponent,
		)
	}
}

#[derive(Debug, Clone)]
pub struct Player<S>
{
	id: UserId,
	selection: S,
	score: u32,
}
impl<S> Player<S>
{
	pub fn id(&self) -> UserId
	{
		self.id
	}
	pub fn selection(&self) -> &S
	{
		&self.selection
	}
	pub fn score(&self) -> u32
	{
		self.score
	}

	pub fn increment_score(&mut self)
	{
		self.score += 1;
	}

	pub fn map_to<T>(self, new_selection: T) -> Player<T>
	{
		Player {
			selection: new_selection,
			id: self.id,
			score: self.score,
		}
	}
}
impl<S> Player<Option<S>>
{
	pub fn new(id: UserId) -> Self
	{
		Self {
			id,
			selection: None,
			score: 0,
		}
	}

	pub fn select(&mut self, selection: S)
	{
		self.selection = Some(selection);
	}
	pub fn has_selected(&self) -> bool
	{
		self.selection.is_some()
	}
	pub fn with_selection(self, selection: S) -> Player<S>
	{
		Player {
			id: self.id,
			selection,
			score: self.score,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side
{
	Challenger,
	Opponent,
}
impl std::ops::Not for Side
{
	type Output = Self;

	fn not(self) -> Self::Output
	{
		match self
		{
			Self::Challenger => Self::Opponent,
			Self::Opponent => Self::Challenger,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr, EnumIter)]
pub enum Selection
{
	Rock,
	Paper,
	Scissors,
}
impl Selection
{
	pub fn map_all<T>(f: impl FnMut(Self) -> T) -> impl Iterator<Item = T>
	{
		Self::iter().map(f)
	}

	pub fn emoji(self) -> char
	{
		match self
		{
			Self::Rock => '\u{270a}',
			Self::Paper => '\u{1f590}',
			Self::Scissors => '\u{270c}',
		}
	}

	pub fn as_str(self) -> &'static str
	{
		self.into()
	}

	pub fn button(self) -> CreateButton
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
