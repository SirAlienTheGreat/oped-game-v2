use futures_util::{SinkExt, StreamExt};
use log::*;
use std::{net::SocketAddr, time::Duration};
use async_channel::{Receiver, Sender, unbounded};
use tokio::net::{TcpListener, TcpStream};
//use tokio::sync::{mpsc,broadcast};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tungstenite::{Message, Result};

mod get_anime;

async fn accept_connection(peer: SocketAddr, stream: TcpStream, data_sender:Sender<GameInfo>, data_receiver:Receiver<GameInfo>, id:i32, anime_list:Vec<String>) {
    if let Err(e) = handle_connection(peer, stream, data_sender, data_receiver, id, anime_list).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {:?}", err),
        }
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream, data_sender:Sender<GameInfo>, data_receiver:Receiver<GameInfo>, id:i32, anime_list:Vec<String>) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    println!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut interval = tokio::time::interval(Duration::from_millis(1000));

    // Echo incoming WebSocket messages and send a message periodically every second.

    let mut client_game_info = GameInfo {
        players:vec![],
        correct_answer:"".to_string(),
        current_video:"".to_string(),
        showing_video:false,
    };

    let mut player:Player = Player {
        name:"Anonymous".to_string(),
        score:0,
        guess:"...".to_string(),
        id,
    };

    // Add the player to the GameInfo structure
    let mut game_info = data_receiver.recv().await.expect("Couldn't receive GameInfo structure to add new player");
    
    println!("adding new player to GameInfo structure");

    game_info.players.push(player.clone());
    data_sender.send(game_info).await.expect("Couldn't send GameInfo structure to add new player");

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {

                // If a message was received
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        if msg.is_text() ||msg.is_binary() {
                            // The prefix is the command - e.g the "VID" in VID saekanoS2-ED1.webm
                            if msg.to_string().len() >= 3 {
                                // Store the player info to see if it changes
                                let player_buffer = player.clone();

                                let prefix = msg.to_string()[0..3].to_string();

                                println!("Prefix is {}", prefix);

                                match prefix.as_str() {
                                    "FIN" => {
                                        let (current_video, current_answer) = get_anime::get_new_theme_recursive(&anime_list).await;

                                        let mut newgamestate = data_receiver.recv().await.unwrap();
                                        // set all guesses to ...
                                        for i in 0..newgamestate.players.len() {
                                        if newgamestate.players[i].guess != "|-|".to_string(){
                                            newgamestate.players[i].guess = "...".to_string();
                                        }

                                        }
                                        newgamestate.showing_video = false;

                                        newgamestate.current_video = current_video.clone();
                                        newgamestate.correct_answer = current_answer;

                                        data_sender.send(newgamestate.clone()).await.unwrap();

                                        ws_sender.send(tungstenite::Message::Text(format!("VID {}", current_video))).await.unwrap();
                                        if player.guess != "|-|".to_string(){
                                            player.guess = "...".to_string();
                                        }
                                    },
                                    "ANS" => player.guess = msg.to_string()[4..].to_string(),
                                    "USR" => player.name = msg.to_string()[4..].to_string(),
                                    "LST" => {
                                        let newgamestate = data_receiver.recv().await.expect("couldn't get GameInfo");
                                        data_sender.send(newgamestate.clone()).await.expect("couldn't return GameInfo");
                                        
                                        let mut players:Vec<String> = vec![];
                                        for player in &newgamestate.players {
                                            players.push(player.name.clone());
                                            players.push(player.score.to_string());
                                        }
                                        ws_sender.send(Message::Text(format!("USR {:?}", players.join("||||||")))).await.expect("couldn't send all players");
                                    },
                                        _ => ws_sender.send(tungstenite::Message::Text(format!("LOG Bad message: '{:?}'{}",msg,player.id))).await.unwrap(),
                                }
                                // If the player info has changed, send any changes
                                if player != player_buffer{

                                    let mut newgamestate = data_receiver.recv().await.expect("couldn't get GameInfo");

                                    //find index of the player's id
                                    for i in 0..(newgamestate.players.len()) {
                                        if newgamestate.players[i].id == id {
                                            // if the id is found
                                            newgamestate.players[i] = player.clone();
                                            println!("global GameInfo will be updated with guess {}", player.guess);
                                            break
                                        }
                                    }


                                    data_sender.send(newgamestate).await.expect("couldn't send changes to player");
                                }

                            } else {
                                ws_sender.send(tungstenite::Message::Text(format!("LOG Bad message (too short): '{:?}'",msg))).await.unwrap();
                            }

                        } else if msg.is_close() {
                            // Delete users when they disconnect
                            let mut newgamestate = data_receiver.recv().await.expect("couldn't get GameInfo");

                            //find index of the player's id
                            for i in 0..(newgamestate.players.len()) {
                                if newgamestate.players[i].id == id {
                                    // if the id is found
                                    newgamestate.players.remove(i);
                                    println!("removing player {}", player.name);
                                    break
                                }
                            }
                            data_sender.send(newgamestate).await.expect("couldn't send changes to data_sender after removing user");


                            break;
                        }
                    }
                    None => break,
                }
            }
            _ = interval.tick() => {
                //ws_sender.send(Message::Text("tick".to_owned())).await?;
                // Retrieves  returns data from thread
                // This acts as a read-only mode after answers are checked for
                let mut newgamestate = data_receiver.recv().await.expect("couldn't get GameInfo");
                
                //change the client's stored answer to the new answer 
                //find index of the player's id
                /*for i in 0..(newgamestate.players.len()) {
                    if newgamestate.players[i].id == id {
                        // if the id is found
                        newgamestate.players[i].guess = player.guess.clone();
                    }
                }*/
                
                //if the theme hasnt been revealed yet,
                if !newgamestate.showing_video {
                    // Check if all answers have been submitted
                    println!("checking answers");
                    let mut all_answers_are_in = true;
                    for i in &newgamestate.players {
                        if i.guess == "..." {
                            all_answers_are_in = false;
                            println!("{} needs to guess {}",i.name, i.guess);
                        }
                    }

                    // if its not showing video, send information about answers and tell it to show video
                    if all_answers_are_in {
                        let mut players_and_answers = vec![];
                        for i in &newgamestate.players {
                            players_and_answers.push(i.name.clone());
                            players_and_answers.push(i.guess.clone());
                        }
                        ws_sender.send(Message::Text(format!("ANS {}", newgamestate.correct_answer))).await.unwrap();
                        ws_sender.send(Message::Text(format!("OAN {}", players_and_answers.join("||||||")))).await.unwrap();

                        newgamestate.showing_video = true;

                        // TODO: check if answers are correct
                    }
                }

                data_sender.send(newgamestate.clone()).await.expect("couldn't return GameInfo");
                // if any GameInfo has changes, send the client an updated version of only what they need
                if newgamestate != client_game_info{
                    if newgamestate.current_video != client_game_info.current_video {
                        ws_sender.send(Message::Text(format!("VID {}", newgamestate.current_video))).await.expect("couldn't send current_video");
                        client_game_info.current_video = newgamestate.current_video;
                    }

                    // if the client doesn't know that they are now showing the video
                    if newgamestate.showing_video != client_game_info.showing_video {
                        ws_sender.send(Message::Text(format!("DSP {}", newgamestate.showing_video.to_string()))).await.expect("couldn't showing_video information");
                        let mut players_and_answers = vec![];
                        for i in &newgamestate.players {
                            players_and_answers.push(i.name.clone());
                            players_and_answers.push(i.guess.to_string());
                        }
                        if newgamestate.showing_video{
                            ws_sender.send(Message::Text(format!("ANS {}", newgamestate.correct_answer))).await.unwrap();
                            ws_sender.send(Message::Text(format!("OAN {}", players_and_answers.join("||||||")))).await.unwrap();
                        }
                        client_game_info.showing_video = newgamestate.showing_video
                    }

                    
                    
                    // if the players have changed, send all the new players
                    if &newgamestate.players != &client_game_info.players {
                        let mut players:Vec<String> = vec![];
                        for player in &newgamestate.players {
                            players.push(player.name.clone());
                            players.push(player.score.to_string());
                        }
                        ws_sender.send(Message::Text(format!("USR {:?}", players.join("||||||")))).await.expect("couldn't send all players with tick");
                        client_game_info.players = newgamestate.players;
                    }

                    
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let list = get_anime::get_all_anime();

    let addr = "0.0.0.0:9002";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    println!("Listening on: {}", addr);

    let (status_sender, status_reciever) = unbounded();// bounded(10);
    status_sender.send(GameInfo {
        players:vec![],
        correct_answer:"".to_string(),
        current_video:"".to_string(),
        showing_video:false,
    }).await.expect("Couldn't initialize GameInfo sender");

    let mut current_id = 0;

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        println!("Peer address: {}", peer);

        tokio::spawn(accept_connection(peer, stream, status_sender.clone(),
                        status_reciever.clone(), current_id,list.clone()));
        current_id += 1;
    }
}

#[derive(Clone, PartialEq)]
struct GameInfo {
    players:Vec<Player>,
    correct_answer:String,
    current_video:String,
    showing_video:bool,
}

#[derive(Clone, PartialEq)]
struct Player {
    name:String,
    score:i32,
    guess:String,
    id:i32,
}