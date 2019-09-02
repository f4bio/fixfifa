# \#FixFifa

FIFA (19) fixes

## Motivation

### Rant (aka "Prologue")

EA just doesn't give a fuck!
Each year, they release a new expensive game (which, at most, should be considered a DLC), patch it few times and release fucking FUT cards.
Couple of months into the game, they stop updating/patching all together (except FUT ofc).

Even worst than their update/release politics, their processing of cheaters/hacker/bots/younameit...
Wanna know how hard it is to cheat in pro clubs?  
See: https://www.elitepvpers.com/forum/fifa/4533276-free-pro-clubs-trainer-fifa-19-a.html  
Public forum, includes instructions and free (no more Bitcoin transfers to some shady russian YouTubers, right?) and
btw: Props to `marcopro007` for sharing his hack open and publicly; nice work and nice attitude, bro! 

Furthermore, there is no acceptable/easy way to report cheating players (One has to remember their nick, and search them in origin? Also, how do I search clubs in origin? Oo)  
But, even if reported and proven guilty (via videos and what not), there will be no punishments from EA... - don\'t even bother

Other then to "speak with your wallet" (which doesn't work as it seems -  at all), there is no way to convince EA to ever change...

So, we took matters in our own hands...

tl;dr: **FUCK YOU, EA!** and let us introduce \#FixFifa 

## Introduction

As stated, this is a (hopefully growing) compilation of hacks to add some desperately needed features to FIFA 

### How does it work?

(Not entirely coincidentally,) we're using the same techniques to identify cheaters as they use to cheat in the first place...  
So, I guess this violates EAs ToS? right? maybe?

## Hacks

### Blacklist

All players of a team have to keep this this tool running in the background while playing.
Whenever matched against a known cheating team, the plug gets pulled (you get disconnected).
You and your team get sent back into lobby and the game won't count.
I've started a (master-)list of ~5 teams [hosted publicly](https://s3.eu-central-1.amazonaws.com/fixfifa/globals.json).

#### Key points:

* Runs on the good guys' (i.e. not cheaters) PCs only
* There don't _has_ to be large amount of users for this tool; works for a single player just as good
* But, the more players use this tool, the better the swamp can be drained
* No need to be (absolutely) sure you've identified cheaters, everyone can be blocked
* Game will be left before kicking off (if matched against cheating teams)

#### Shame list

`https://iam.f4b.io/fixfifa/globals.json`  
or:  
`https://s3.eu-central-1.amazonaws.com/fixfifa/globals.json`  

### Disable Alt-Tab disconnect

...

### Skip Launcher

...

## Technical Details

### Hooking / Hacking / Memory manipulation
* event to listen to `EVENT_PREGAME_GROUP_ATTRIBUTE_CHANGED`
* in-memory
  ```
  _G_I_0=29227
  _G_O_0=AMK
  _G_N_0="FC AMKAR PERM"
  _G_Z_0=99020102
  _G_T_0=5
  _G_J_0=7664
  _G_C_0=hop1q
  _G_I_1=25681
  _G_O_1=LEV
  _G_N_1=glhfc
  _G_Z_1=99070109
  _G_T_1=1853
  _G_J_1=7509
  _G_C_1=19AntiCampeR89
  Q=29
  ```
  
  ```
  _G_I_0=24154
  _G_O_0=FEK
  _G_N_0="Fecking Pirates"
  _G_Z_0=99071012
  _G_T_0=1370
  _G_J_0=7512
  _G_C_0=Hubiektyw
  _G_I_1=25681
  _G_O_1=LEV
  _G_N_1=glhfc
  _G_Z_1=99070109
  _G_T_1=1853
  _G_J_1=7509
  _G_C_1=19AntiCampeR89
  Q=-1
  ```
  
  ```
  _G_I_0=24249
  _G_O_0=BAZ
  _G_N_0=Bazinga
  _G_Z_0=112092
  _G_T_0=112092
  _G_J_0=1836515328
  _G_C_0=Ecc0Problems
  _G_I_1=822
  _G_O_1=WOB
  _G_N_1=Hulkfc
  _G_Z_1=99081024
  _G_T_1=110062
  _G_J_1=7621
  _G_C_1=teeee2011
  Q=-1
  ```
  
  ```
  _G_I_0=29322
  _G_O_0=AS
  _G_N_0="Atletico Squad"
  _G_Z_0=99090101
  _G_T_0=315
  _G_J_0=7623
  _G_C_0=kolyaf228
  _G_I_1=25681
  _G_O_1=LEV
  _G_N_1=glhfc
  _G_Z_1=99070109
  _G_T_1=1853
  _G_J_1=7509
  _G_C_1=19AntiCampeR89
  Q=-1
  ```
* \*.json

  ```json
  {
    "bad_clubs": [
      {"id": 29227, "name": "Atletico Squad", "abbr": "AS"},
      {"id": 24154, "name": "Fecking Pirates", "abbr": "FEK"},
      {"id": 24249, "name": "Bazinga", "abbr": "BAZ"},
      {"id": 29322, "name": "Atletico Squad", "abbr": "AS"}
    ]
  }
  ```
* ...

## Lookout

### Todo / Roadmap

Keep in mind: We both have full time jobs, so there is a big maybe behind all following points!

* local blacklists
* notify users when and why plug was pulled
* figure out how users can find out cheaters ids ("Report Team/Players" button)
* some sort of voting/review system for the master list
* some way to get of the master list (e.g. if falsely added)
* expand to other game modes
* maybe implement some server-client model
  * don't rely on lists
  * master (i.e. team captain) tells everybody: "get outta here"
* check out different ways to disconnect

## Contribute

### Donate
* ...

### Pull Request
* ...

## Build

`$ cargo build --all`
