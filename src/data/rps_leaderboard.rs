use std::{cmp::Ordering, collections::HashMap, future::Future};

use poise::serenity_prelude::UserId;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Leaderboard
{
	map: HashMap<UserId, Score>,
}
impl Leaderboard
{
	pub fn score(&self, player: UserId) -> Option<&Score>
	{
		self.map.get(&player)
	}
	pub fn score_mut(&mut self, player: UserId) -> &mut Score
	{
		self.map.entry(player).or_default()
	}

	pub fn ordered_scores(&self, limit: Option<usize>) -> Vec<LeaderboardEntry<'_>>
	{
		let mut unranked_vec = self
			.map
			.iter()
			.take(limit.unwrap_or(usize::MAX))
			.map(|(id, score)| (*id, score))
			.collect::<Vec<_>>();

		// unstable sorting by id then stable sorting by score should ensure ordered by score then
		// by id -morgan 2024-05-19
		unranked_vec.sort_unstable_by_key(|item| item.0);
		unranked_vec.sort_by(|a, b| b.1.cmp(a.1));

		// oh god this is a nightmare -morgan 2024-05-20
		let mut ranked_vec = Vec::with_capacity(unranked_vec.len());
		let mut unranked_vec_iter = unranked_vec.into_iter().peekable();
		let mut rank = 1;
		while let Some((id, score)) = unranked_vec_iter.next()
		{
			ranked_vec.push(LeaderboardEntry::new(id, rank, score));
			if let Some((_, next_score)) = unranked_vec_iter.peek()
			{
				if &score != next_score
				{
					rank += 1;
				}
			}
		}

		ranked_vec
	}
}

pub struct LeaderboardEntry<'a, U = UserId>
{
	user: U,
	rank: u32,
	score: &'a Score,
}
impl<'a, U> LeaderboardEntry<'a, U>
{
	pub fn new(id: U, rank: u32, score: &'a Score) -> Self
	{
		Self {
			user: id,
			rank,
			score,
		}
	}

	pub fn user(&self) -> &U
	{
		&self.user
	}
	pub fn rank(&self) -> u32
	{
		self.rank
	}
	pub fn score(&self) -> &Score
	{
		self.score
	}

	pub fn map_user<T>(self, transform: impl Fn(U) -> T) -> LeaderboardEntry<'a, T>
	{
		LeaderboardEntry {
			user: transform(self.user),
			rank: self.rank,
			score: self.score,
		}
	}
}
impl<'a, U, E> LeaderboardEntry<'a, Result<U, E>>
{
	pub fn transpose(self) -> Result<LeaderboardEntry<'a, U>, E>
	{
		match self.user
		{
			Ok(inner) => Ok(LeaderboardEntry {
				user: inner,
				rank: self.rank,
				score: self.score,
			}),
			Err(error) => Err(error),
		}
	}
}
impl<'a, U: Future + Send + Sync> LeaderboardEntry<'a, U>
{
	pub async fn await_user(self) -> LeaderboardEntry<'a, U::Output>
	{
		LeaderboardEntry {
			user: self.user.await,
			rank: self.rank,
			score: self.score,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Score
{
	pub wins: u32,
	pub losses: u32,
	pub elo: i32,
}
impl Score
{
	pub const BASE_ELO: i32 = 1500;
	/// sets the smoothness of the ELO change curve; lower values exaggerate ELO difference more
	const ELO_SMOOTHING: f64 = 400.0;

	pub fn win_rate(&self) -> f64
	{
		(self.total_games() != 0)
			.then(|| f64::from(self.wins) / f64::from(self.total_games()))
			.unwrap_or_default()
	}

	fn total_games(&self) -> u32
	{
		self.wins + self.losses
	}

	pub fn increment_wins(&mut self)
	{
		self.wins += 1;
	}
	pub fn increment_losses(&mut self)
	{
		self.losses += 1;
	}

	pub fn update_elo(&mut self, opponent_elo: i32, outcome: Outcome) -> i32
	{
		self.elo += self.elo_change(opponent_elo, outcome);
		self.elo
	}

	pub fn elo_change(&self, opponent_elo: i32, outcome: Outcome) -> i32
	{
		let elo_difference = f64::from(opponent_elo - self.elo);
		let sensitivity = match self.elo
		{
			..=2100 => 40.0,
			2101..=2400 => 25.0,
			2401.. => 15.0,
		};

		let float_change = sensitivity
			* (outcome.value()
				- (1.0 + f64::powf(10.0, elo_difference / Self::ELO_SMOOTHING)).recip());

		float_change.floor() as i32
	}
}
impl Default for Score
{
	fn default() -> Self
	{
		Self {
			wins: 0,
			losses: 0,
			elo: Self::BASE_ELO,
		}
	}
}
impl PartialOrd for Score
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering>
	{
		Some(self.cmp(other))
	}
}
impl Ord for Score
{
	fn cmp(&self, other: &Self) -> Ordering
	{
		match self.elo.cmp(&other.elo)
		{
			Ordering::Equal => (),
			ord => return ord,
		}

		match self.wins.cmp(&other.wins)
		{
			Ordering::Equal => (),
			ord => return ord,
		}

		self.losses.cmp(&other.losses).reverse()
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Outcome
{
	#[default]
	Loss,
	Win,
}
impl Outcome
{
	pub fn value(self) -> f64
	{
		match self
		{
			Self::Loss => 0.0,
			Self::Win => 1.0,
		}
	}
}
impl std::ops::Not for Outcome
{
	type Output = Self;

	fn not(self) -> Self::Output
	{
		match self
		{
			Self::Loss => Self::Win,
			Self::Win => Self::Loss,
		}
	}
}
impl From<bool> for Outcome
{
	fn from(value: bool) -> Self
	{
		if value
		{
			Self::Win
		}
		else
		{
			Self::Loss
		}
	}
}
