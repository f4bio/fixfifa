# \#FixFifa

## global data

### fetch from

`https://iam.f4b.io/fixfifa/globals.json`  
or:  
`https://s3.eu-central-1.amazonaws.com/fixfifa/globals.json`  

#### example

[docs/globals.example.json](./docs/globals.example.json)

```bash
1418D7EAE
extern init call
got event: EVENT_PREGAME_GROUP_FOR_PLAYER_CHANGED / _GI=0;USER_NAME=Konry11:Daev2105
got event: EVENT_PREGAME_SESSION_ATTRIBUTE_CHANGED / Q=116:Daev2105
got event: EVENT_PREGAME_GROUP_FOR_PLAYER_CHANGED / _GI=-1;USER_NAME=Konry11:Daev2105
got event: EVENT_PREGAME_GROUP_FOR_PLAYER_CHANGED / _GI=1;USER_NAME=Lurienne:Daev2105
got event: EVENT_PREGAME_SESSION_ATTRIBUTE_CHANGED / Q=115:Daev2105
got event: EVENT_PREGAME_PLAYER_ATTRIBUTE_CHANGED / VPRO=12|80|6.700000|1;USER_NAME=Konry11:Konry11
got event: EVENT_PREGAME_PLAYER_ATTRIBUTE_CHANGED / VPRO=16|90|8|0;USER_NAME=Lurienne:Lurienne
got event: EVENT_PREGAME_SESSION_ATTRIBUTE_CHANGED / Q=114:Daev2105
got event: EVENT_PREGAME_SESSION_ATTRIBUTE_CHANGED / Q=113:Daev2105
got event: EVENT_PREGAME_SESSION_ATTRIBUTE_CHANGED / Q=112:Daev2105
got event: EVENT_PREGAME_GROUP_FOR_PLAYER_CHANGED / _GI=1;USER_NAME=Konry11:Daev2105
got event: EVENT_PREGAME_SESSION_ATTRIBUTE_CHANGED / Q=111:Daev2105
got event: EVENT_PREGAME_SESSION_ATTRIBUTE_CHANGED / Q=110:Daev2105
```

## build

`$ cargo build --all`
