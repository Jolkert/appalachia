use std::fmt::Write;

use poise::{
	serenity_prelude::{
		CreateAllowedMentions, CreateEmbed, Member, Mentionable, PartialGuild, UserId,
	},
	CreateReply,
};

use crate::{
	command::ExpectGuildOnly,
	data::{GuildData, LeaderboardEntry, Score},
	Context, Error, Reply,
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

	($buffer:expr, $spacing:expr, $entry:expr) => {
		write_lb_line!(
			$buffer,
			$spacing,
			($entry).rank(),
			($entry).user(),
			($entry).score().elo,
			($entry).score().wins,
			($entry).score().losses,
			format!("{:.4}", ($entry).score().win_rate()).trim_start_matches('0')
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

/// View the Rock, Paper, Scissors leaderboard for this server
#[poise::command(aliases("lb"), slash_command, prefix_command, guild_only)]
pub async fn leaderboard(
	ctx: Context<'_>,
	#[description = "Specify a user to see their specific score"] user: Option<Member>,
) -> Result<(), Error>
{
	let guild = ctx.partial_guild().await.expect_guild_only();

	if let Some(target_member) = user
	{
		user_score(ctx, &guild, target_member).await?;
	}
	else if let Some(sorted_leaderboard) = ctx
		.data()
		.acquire_lock()
		.await
		.guild_data(guild.id)
		.map(|dat| dat.leaderboard().ordered_scores(Some(15)))
		&& !sorted_leaderboard.is_empty()
	{
		full_leaderboard(ctx, &guild, sorted_leaderboard).await?;
	}
	else
	{
		ctx.reply_error("No leaderboard exists for this server!")
			.await?;
	}

	Ok(())
}

// TODO: i still dont like this code very much but thats for a later time i think -morgan 2024-05-27
// it's a litte better now i think. im still not a big fan of the `StringLengths` struct. either in
// name or in purpose. but i think it'll do for now -morgan 2024-05-28
async fn full_leaderboard(
	ctx: Context<'_>,
	guild: &PartialGuild,
	sorted_leaderboard: Vec<LeaderboardEntry<'_>>,
) -> Result<(), Error>
{
	// so for some reason vscode inlay hints are really confused as to what the type of `entry` is
	// despite rustc being perfectly able to deduce it? kinda weird -morgan 2024-05-28
	let scores = futures::future::try_join_all(sorted_leaderboard.into_iter().map(
		|entry: LeaderboardEntry<UserId>| async {
			entry
				.map_user(|id| async move {
					guild
						.member(ctx.http(), id)
						.await
						.map(|member| unidecode::unidecode(member.display_name()))
				})
				.await_user()
				.await
				.transpose()
		},
	))
	.await?;

	let string_lengths = get_max_lengths(&scores).cap_name_at(32);

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
	for entry in scores
	{
		if !top_3_line_drawn && entry.rank() > 3
		{
			leaderboard_string.push_str(&string_lengths.draw_line('┄', '┼'));
			top_3_line_drawn = true;
		}

		let _ = write_lb_line!(leaderboard_string, string_lengths, entry);
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

	Ok(())
}

fn get_max_lengths(leaderboard_entries: &[LeaderboardEntry<'_, String>]) -> StringLengths
{
	let mut lengths = StringLengths::default();
	for entry in leaderboard_entries
	{
		lengths.set_name(entry.user());
		lengths.set_rank(entry.rank());
		lengths.set_elo(entry.score().elo);
		lengths.set_losses(entry.score().losses);
		lengths.set_wins(entry.score().wins);
		lengths.set_winrate(entry.score().win_rate());
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
		let new = unidecode::unidecode(name).len();
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
		let new = format!("{winrate:.4}").trim_start_matches('0').len();
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
				repeat!(times + 1 + usize::from((1..=4).contains(&i)), {
					line_string.push(horizontal);
				});
				if i < 5
				{
					line_string.push(vertical);
				}
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

async fn user_score(
	ctx: Context<'_>,
	guild: &PartialGuild,
	target_member: Member,
) -> Result<(), Error>
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
				.embed(create_user_score_embed(
					&target_member,
					guild,
					score,
					guild_data,
				))
				.reply(true)
				.allowed_mentions(CreateAllowedMentions::new())
				.ephemeral(false),
		)
		.await?;
	}
	else
	{
		ctx.reply_error(format!(
			"{} has no rock paper scissors scores!",
			target_member.mention()
		))
		.await?;
	};

	Ok(())
}

fn create_user_score_embed(
	target_member: &Member,
	guild: &PartialGuild,
	score: &Score,
	guild_data: &GuildData,
) -> CreateEmbed
{
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
					.position(|entry| target_member.user.id == *entry.user())
					.unwrap_or_else(|| panic!(
						"Could not find user {}({}) in {}({})",
						target_member.user.name, target_member.user.id, guild.name, guild.id
					)) + 1,
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
		.thumbnail(target_member.face())
}
