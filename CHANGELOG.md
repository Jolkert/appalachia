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