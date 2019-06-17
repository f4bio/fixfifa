use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Player {
    id: i32,
    name: String,
}

impl Player {
    pub fn new() -> Self {
        Player { id: -1, name: "".to_string() }
    }
}
