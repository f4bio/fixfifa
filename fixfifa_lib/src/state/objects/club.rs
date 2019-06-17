use crate::state::objects::player::Player;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::string::ToString;

#[derive(Deserialize, Serialize)]
pub struct Club {
    id: i32,
    name: String,
    abbr: String,
    players: Vec<Player>,
    captain: Player,
}

impl Display for Club {
    fn fmt(&self, c: &mut Formatter) -> fmt::Result {
        write!(c, "Club: id={}, name={}, abbr={}", self.id, self.name, self.abbr)
    }
}

impl Club {
    pub fn new() -> Self {
        Club {
            id: -1,
            name: "".to_string(),
            abbr: "".to_string(),
            players: vec![],
            captain: Player::new(),
        }
    }
}
