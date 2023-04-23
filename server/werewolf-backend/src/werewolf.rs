use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
};

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use warp::ws::Message;

use crate::Game;

pub enum Action {
    Attach(usize, usize),
    Kill(usize),
    Heal(usize),
    Poison(usize),
    Elect(usize),
    Vote(usize),
}

pub trait IntoAction {
    fn into_action(self) -> Result<Action, Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WerewolfSettings {
    num_players: usize,
    num_werewolfs: usize,
    heals: usize,
    poisons: usize,
    mayor_votes: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Villager,
    Werewolf,
    Amor,
    Witch { heals: usize, poisons: usize },
    Mayor { votes: usize },
    Spectator,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum State {
    #[default]
    Pending,
    Lovers,
    Night,
    Bewitch,
    Election,
    Day,
}

impl Into<std::string::String> for State {
    fn into(self) -> std::string::String {
        match self {
            State::Pending => "pending".to_string(),
            State::Lovers => "lovers".to_string(),
            State::Night => "night".to_string(),
            State::Bewitch => "bewitch".to_string(),
            State::Election => "election".to_string(),
            State::Day => "day".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Player {
    pub role: Role,
    pub channel: mpsc::UnboundedSender<Result<Message, warp::Error>>,
    pub lover: Option<usize>,
    pub is_mayor: bool,
}

#[derive(Debug, Default)]
pub struct WerewolfGame {
    pub roles: Vec<Role>,
    pub state: State,
    pub players: HashMap<usize, Player>,
}

impl WerewolfGame {
    pub fn new(settings: WerewolfSettings) -> Self {
        let mut roles = vec![Role::Werewolf; settings.num_werewolfs];
        roles.push(Role::Amor);
        roles.push(Role::Witch {
            heals: settings.heals,
            poisons: settings.poisons,
        });
        roles.append(
            vec![Role::Villager; settings.num_players - settings.num_werewolfs - 2].as_mut(),
        );
        roles.shuffle(&mut rand::thread_rng());
        Self {
            roles,
            ..Default::default()
        }
    }
}

pub fn perform_action(action: Action, player: &Player, game: &Game) {
    match action {
        Action::Attach(id1, id2) if player.role == Role::Amor => {}
        Action::Kill(id) if player.role == Role::Werewolf => {}
        Action::Heal(id) => match player.role {
            Role::Witch { heals, poisons } => (),
            _ => (),
        },
        Action::Poison(id) => match player.role {
            Role::Witch { heals, poisons } => (),
            _ => (),
        },
        Action::Elect(id) => {}
        Action::Vote(id) => {}
        Action::Attach(_, _) | Action::Kill(_) => (),
    }
}
