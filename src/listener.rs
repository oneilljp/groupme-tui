use std::error::Error;
use std::sync::mpsc;

use serde_json::{json, Value};

use tungstenite::{client::AutoStream, connect, protocol::WebSocket, Message};

use url::Url;

use chrono::prelude::*;
use chrono::Duration;

use notify_rust::Notification;

#[cfg(target_os = "macos")]
static SOUND: &'static str = "Ping";

#[cfg(all(unix, not(target_os = "macos")))]
static SOUND: &str = "message-new-instant";

#[cfg(target_os = "windows")]
static SOUND: &'static str = "Mail";


/* Call on seperate thread to poll for new notifications, and then send a desktop notification
 * with the contents
 *
 * rx: mspc::Reciever<bool> - channel to send shutdown signal from the main thread
 * user_id: &str - User's ID
 * secret: API Key
 */
pub fn listener(rx: mpsc::Receiver<bool>, user_id: &str, secret: &str) {
    let mut id: u64 = 1;


    let (mut socket, _) = connect(Url::parse("wss://push.groupme.com/faye").unwrap())
        .expect("Couldn't connect socket");

    let mut client_id = subscribe(&mut socket, secret, user_id, &mut id);
    let mut last_hs = Local::now();

    loop {
        // Check for shutdown signal
        match rx.try_recv() {
            Ok(shutdown) => {
                if shutdown {
                    socket.close(None).unwrap();
                    return;
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // Unable to receive shutdown signal, returning to be safe
                return;
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }

        if last_hs.timestamp() + Duration::hours(1).num_seconds() < Local::now().timestamp() {
            client_id = subscribe(&mut socket, secret, user_id, &mut id);
            last_hs = Local::now();
        }

        poll(&mut socket, &client_id, &mut id).unwrap();
    }

}

/* Subscribe to push notifications using websockets
 *
 * socket: Websocket connected to wss://push.groupme.com/faye
 * secret: API key
 * user_id: User's id
 * id: Incrementing id for communicating with GroupMe's servers
 */
fn subscribe(
    socket: &mut WebSocket<AutoStream>,
    secret: &str,
    user_id: &str,
    id: &mut u64,
) -> String {
    // First do handshake
    let hs = json!(
    [{
        "channel": "/meta/handshake",
        "version": "1.0",
        "supportedConnectionTypes": ["websocket"],
        "id": id.to_string(),
    }]
    );

    socket
        .write_message(Message::Text(hs.to_string()))
        .expect("Error handshaking");
    *id += 1;

    let resp = socket.read_message().expect("Error receiving handshake");
    let resp_json: Value =
        serde_json::from_str(resp.to_text().unwrap()).expect("Error parsing json");


    let client_id = resp_json.as_array().unwrap()[0]["clientId"]
        .as_str()
        .unwrap();
    let timestamp = Local::now().timestamp().to_string();

    // Subscribe to user channel to recieve notifications when polling
    let sub_msg = json!(
    [{
        "channel": "/meta/subscribe",
        "clientId": client_id,
        "subscription": format!("/user/{}", user_id),
        "id": id.to_string(),
        "ext": {
            "access_token": secret,
            "timestamp": timestamp
        }
    }]
    );

    // Write subscription
    socket
        .write_message(Message::Text(sub_msg.to_string()))
        .expect("Error Subscribing");
    *id += 1;

    // Read confirmation message to empty queue
    socket
        .read_message()
        .expect("Error recieving sub confirmation");

    client_id.to_string()
}

/* Poll push notification server once
 * socket: &mut WebSocket<AutoStream> - Socket connected to wss:://push.groupme.com/faye
 * client_id: &str - Current polling signature, obtained from handshake
 * id: Incrementing id for communicating with GroupMe's servers
 */
fn poll(socket: &mut WebSocket<AutoStream>, client_id: &str, id: &mut u64) -> Result<(), Box<dyn Error>> {
    let poll_msg = json!(
        [{
            "channel": "/meta/connect",
            "clientId": client_id,
            "connectionType": "websocket",
            "id": id
        }]
        );
    socket.write_message(Message::Text(poll_msg.to_string())).expect("Error polling faye");
    *id += 1;

    let resp = socket.read_message().expect("Error reading message");

    // As far as I know, the only text responses that will be sent are notifications
    if resp.is_text() {
        let msg = resp.to_text().unwrap();
        let poll_json: Value = serde_json::from_str(&msg)?;
        if poll_json.as_array().unwrap().len() == 1 {
            return Ok(())
        }
        let poll_results = poll_json.as_array().unwrap()[1].as_object().unwrap();

        if !poll_results.contains_key("data") || !poll_results["data"].as_object().unwrap().contains_key("alert") {
            return Ok(())
        }
        
        let alert = poll_results["data"]["alert"].as_str().unwrap();
        Notification::new()
            .summary("GroupMe")
            .sound_name(SOUND)
            .icon("mail-unread")
            .body(&alert)
            .show()?;
    }
    Ok(())
}
