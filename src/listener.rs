use std::error::Error;
use std::sync::mpsc;

use ureq;

use serde_json::Value;

use chrono::prelude::*;
use chrono::Duration;

use notify_rust::Notification;

pub fn listener(rx: mpsc::Receiver<bool>, user_id: &str, secret: &str) {
    let mut req_id = 1;

    // Complete Handshake
    let mut client_id = handshake(&req_id.to_string()).unwrap();
    req_id += 1;
    while client_id == "" {
        client_id = handshake(&req_id.to_string()).unwrap();
        req_id += 1;
    }

    let mut last_handshake = Local::now();

    // Subscribe for new notifications, may need to redo
    user_subscribe(&client_id, user_id, &req_id.to_string(), secret).unwrap();
    req_id += 1;

    loop {
        // Check for shutdown signal from main thread
        match rx.try_recv() {
            Ok(shutdown) => {
                if shutdown {
                    return;
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // Unable to receive shutdown signal, returning to be safe
                return;
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }

        // Redo handshake and subscription after an hour
        if last_handshake.timestamp() + Duration::hours(1).num_seconds() < Local::now().timestamp()
        {
            client_id = handshake(&req_id.to_string()).unwrap();
            req_id += 1;

            user_subscribe(&client_id, user_id, &req_id.to_string(), secret).unwrap();
            req_id += 1;

            last_handshake = Local::now();
        }
        poll(&client_id, &req_id.to_string()).unwrap();
        req_id += 1;
    }
}

fn handshake(id: &str) -> Result<String, Box<dyn Error>> {
    let url = "https://push.groupme.com/faye";
    let body = ureq::post(&url)
        .send_json(ureq::json!([{
            "channel": "/meta/handshake",
            "version": "1.0",
            "supportedConnectionTypes": ["long-polling"],
            "id": id
        }]))?
        .into_string()?;

    let resp: Value = serde_json::from_str(&body)?;

    let hs_result = &resp.as_array().unwrap()[0];

    if hs_result["successful"].as_bool().unwrap() {
        return Ok(hs_result["clientId"].as_str().unwrap().to_string());
    }

    Ok("".to_string())
}

fn user_subscribe(
    client_id: &str,
    user_id: &str,
    id: &str,
    secret: &str,
) -> Result<bool, Box<dyn Error>> {
    let url = "https://push.groupme.com/faye";
    let timestamp = Local::now().timestamp().to_string();
    let body = ureq::post(&url)
        .send_json(ureq::json!([{
            "channel": "/meta/subscribe",
            "client_id": client_id,
            "subscription": format!("/user/{}", user_id),
            "id": id,
            "ext": {
                "access_token": secret,
                "timestamp": timestamp
            }
        }]))?
        .into_string()?;

    let resp: Value = serde_json::from_str(&body)?;

    let hs_result = &resp.as_array().unwrap()[0];

    Ok(hs_result["successful"].as_bool().unwrap())
}

fn poll(client_id: &str, req_id: &str) -> Result<(), Box<dyn Error>> {
    let url = "https://push.groupme.com/faye";
    let body = ureq::post(&url)
        .send_json(ureq::json!([{
            "channel": "/meta/connect",
            "clientId": client_id,
            "connectionType": "long-polling",
            "id": req_id
        }]))?
        .into_string()?;

    // Need to check for Success but we'll ignore for now

    let resp: Value = serde_json::from_str(&body)?;

    let result_arr = resp.as_array().unwrap();
    let results = result_arr[1].as_object().unwrap();

    // Nothing to report
    if results["data"].as_object().unwrap().contains_key("ping") {
        return Ok(());
    } else {
        let alert = results["alert"].as_str().unwrap();
        Notification::new()
            .summary("RustMe Notification")
            .body(&alert)
            .show()?;
    }
    Ok(())
}
