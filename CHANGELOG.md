# 0.2.8
## Backend
- Updated all dependencies to their latest versions
    - `dirs`: 5.0.1 -> 6.0.0
    - `rand`: 0.8.5 -> 0.9.2 (this is the big one)
    - `strum(_macros)?`: 0.26.2 -> 0.27.2
    - `thiserror`: 1.0.60 -> 2.0.17
    - `toml`: 0.8.12 -> 0.9.7
- Removed dependency on `lazy_static` in favor of `std::sync::LazyLock`
## Repo cleanliness
- Removed `bacon.toml` file in favor of defining clippy lints directly in `Cargo.toml`
- Updated `README.md` to link to the codeberg repo instead of the github one as the main link

# 0.2.7
## Commands
- `/roll` now truncates the embed string if it would be > 1024 characters rather than allowing the malformed embed field message to be sent

# 0.2.6
## Commands
- `/random user` renamed to `/randuser`
- `/randuser` now allows for selecting only from the user's current voice channel

# 0.2.5
## Commands
- `/rps` now allows the user to challenge the bot
    - bot currently plays comepletely at random

# v0.2.3
## Commands
- `/roll` now color-codes rolls based on how good they were relative to the mean

# v0.2.2
## Commands
- Added quote fetching `/quote` and quote channel setting `/quote_channel`
    - `/quote` can pull a random message containing a user mention (optionally, mentioning a specfic user) from the designated quotes channel
    - `/quote_channel` can be used by admins to set which channel `/quote` will pull from in the server

# v0.2.1
## Bugfixes
- Various rock paper scissors leaderboard misalignments fixed up
- Now using `unidecode` to normalize all nicknames to ascii so alignment doesnt break. Some things might still break tho. no promises

# v0.2.0
## Commands
- Added rock paper scissors leaderboard `/rps leaderboard`

# v0.1.2
## Bugfixes
- Added descriptions for `/random user` and `/autorole` commands and their arguments

# v0.1.1
## Bugfixes
- No longer crashes when no `.env` file is present

## Misc
- Better error reporting

# v0.1.0
## Commands
- Added rock paper scissors `/rps`
- Added dice rolling `/roll`
- Added random user generation `/random user`
- Added autorole assignment `/autorole`

## Other
- Added ability to assign autoroles on user join
