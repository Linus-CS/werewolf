use access::{check_access, check_master};
use futures::StreamExt;
use rejection::handle_reject;
use std::{
    io::{Error, ErrorKind},
    sync::Arc,
};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    hyper::{Method, StatusCode},
    ws::{Message, WebSocket},
    Filter, Rejection,
};
use werewolf::{perform_action, Action, IntoAction, Player, State, WerewolfGame, WerewolfSettings};

mod access;
mod rejection;
mod werewolf;

impl IntoAction for Message {
    fn into_action(self) -> Result<Action, Error> {
        let msg = self.to_str().unwrap();
        let mut msg = msg.split_whitespace();
        let action = msg.next().unwrap();
        let target = msg.next().unwrap().parse::<usize>().unwrap();
        match action {
            "attach" => {
                let lover = msg.next().unwrap().parse::<usize>().unwrap();
                Ok(Action::Attach(target, lover))
            }
            "kill" => Ok(Action::Kill(target)),
            "elect" => Ok(Action::Elect(target)),
            "vote" => Ok(Action::Vote(target)),
            "heal" => Ok(Action::Heal(target)),
            "poison" => Ok(Action::Poison(target)),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Invalid action")),
        }
    }
}

type Game = Arc<RwLock<Option<WerewolfGame>>>;
static NEXT_USERID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);

async fn can_join(game: Game) -> Result<Game, Rejection> {
    if game.read().await.is_some()
        && game.read().await.as_ref().unwrap().state == State::Pending
        && game.read().await.as_ref().unwrap().roles.len() > 0
    {
        Ok(game)
    } else {
        Err(warp::reject::not_found())
    }
}

async fn join_game(ws: WebSocket, game: Game) {
    let my_id = NEXT_USERID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    println!("Welcome User {}", my_id);

    let (player_tx, mut player_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();

    let rx = UnboundedReceiverStream::new(rx);
    tokio::spawn(rx.forward(player_tx));

    // Select role depending on settings.
    let player = Player {
        role: game.write().await.as_mut().unwrap().roles.pop().unwrap(),
        channel: tx,
        lover: None,
        is_mayor: false,
    };

    game.write()
        .await
        .as_mut()
        .unwrap()
        .players
        .insert(my_id, player);

    while let Some(result) = player_rx.next().await {
        process_msg(
            result.expect("Failed to fetch message"),
            &game
                .read()
                .await
                .as_ref()
                .unwrap()
                .players
                .get(&my_id)
                .unwrap(),
            &game,
        )
        .await;
    }

    disconnect(my_id, &game).await;
}

async fn process_msg(msg: Message, player: &Player, game: &Game) {
    let Ok(action) = msg.into_action() else { return };
    perform_action(action, player, game);
}

async fn disconnect(my_id: usize, game: &Game) {
    println!("Good bye user {}", my_id);

    game.write().await.as_mut().unwrap().players.remove(&my_id);
}

async fn create_game(
    settings: WerewolfSettings,
    game: Game,
) -> Result<impl warp::Reply, Rejection> {
    if game.read().await.is_some() {
        return Ok(warp::reply::with_status(
            "Game already exists!",
            StatusCode::CONFLICT,
        ));
    }
    game.write().await.replace(WerewolfGame::new(settings));
    Ok(warp::reply::with_status("Created!", StatusCode::OK))
}

#[tokio::main]
async fn main() {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(vec![
            "Access-Control-Allow-Origin",
            "Access-Control-Allow-Headers",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "Origin",
            "Accept",
            "X-Requested-With",
            "Content-Type",
        ]);

    let game: Game = Game::default();
    let game = warp::any().map(move || game.clone());

    let ping = warp::get()
        .and(warp::path("ping"))
        .and(warp::path::end())
        .map(|| "pong");

    let join = warp::path("join")
        .and(warp::path::end())
        .and(game.clone())
        .and_then(can_join)
        .and(warp::ws())
        .map(|game, ws: warp::ws::Ws| ws.on_upgrade(move |socket| join_game(socket, game)));

    let create = warp::path("create")
        .and(warp::path::end())
        .and(warp::body::json::<WerewolfSettings>())
        .and(game.clone())
        .and_then(create_game)
        .with(warp::wrap_fn(check_master));

    let routes = ping.or(join).or(create);

    let routes = routes
        .with(cors)
        .with(warp::wrap_fn(check_access))
        .recover(handle_reject);
    let address = [127, 0, 0, 1];
    let port = 8080;

    println!("Server started on port {address:?}:{port}");
    warp::serve(routes).run((address, port)).await;
}
