# ⛰️ Appalachia
[<img alt="github" src="https://img.shields.io/badge/jolkert%2Fappalachia-babbf1?style=for-the-badge&logo=github&label=github&logoColor=D9E0EE&labelColor=292c3c" height=23>](https://github.com/jolkert/appalachia)
[<img alt="crates.io" src="https://img.shields.io/crates/v/appalachia?style=for-the-badge&logo=rust&logoColor=D9E0EE&labelColor=292c3c&color=ef9f76" height=23>](https://crates.io/crates/appalachia)
[<img alt="docs.rs" src="https://img.shields.io/badge/appalachia-e5c890?style=for-the-badge&logo=docs.rs&logoColor=D9E0EE&label=docs.rs&labelColor=292c3c" height=23>](https://docs.rs/appalachia/latest/appalachia)
[<img alt="Static Badge" src="https://img.shields.io/badge/jolkland-a4baeb?style=for-the-badge&logo=discord&logoColor=D9E0EE&label=discord&labelColor=292c3c" height=23>](https://discord.gg/G3pqGwydVd)

**Appalachia** is a discord bot (re)written in Rust

## Add to your Discord server
If you want to add appalachia to your server, all you need to do is
[click here](https://discord.com/oauth2/authorize?client_id=519292417816395779&permissions=8&scope=bot+applications.commands) and select 
the server you wish to add it to

## Commands
### Rock Paper Scissors `/rps`
- Using `/rps challenge` you can challenge other users in a server to a rock paper 
scissors match. If the opponent accpets, you play by interacting with buttons
on a message the bot will send in the channel the challenge was issued from. 
If you specify an integer in the `first_to` field, the game will continue 
until either player reaches the specified amount of wins.
- Using `/rps leaderboard` you can view the leaderboard for current server. The bot
keeps track of the wins and losses of each member of the server who has played rock
paper scissors at least once in the server. You can also specify a specific member
to view the scores of

### Dice Rolling `/roll`
Using `/roll` you can enter an expression in
[Dice Notation](https://en.wikipedia.org/wiki/Dice_notation), and Appalachia 
will roll the dice for you using the
[Saikoro](`https://github.com/jolkert/saikoro`) dice parser.

### Coin Flipping `/flip`
Using `/flip` you can simulate a coin toss.

### Random User Selection `/random user`
Using `/random user` you can prompt the bot to select a random user from the 
server you are in. You can also specify whether or not bots should be included
(`false` by default) and whether or not the user who ran the command should be
included (`true` by default)

### Random Quote Selection `/quote`
- Using `/quote` you can randomly select a quote (defined as a message which has
at least one user mention) from the server's designated *quotes channel.*
Optionally, you can specify a user pull only quotes which mention that user.
- Using `/quote_channel` you can set which channel is the server's specified
*quotes channel*

### Assigning Autoroles `/autorole`
Using `/autorole` you can set a role to be automatically added to all users
when they first join the server  

To use this command:
- You must have the "Manage Server" permission and
- The bot must have the "Manage Roles" permission

## Planned Features
- `/random user` filter for only including users in the current voice channel
- Custom welcome messages