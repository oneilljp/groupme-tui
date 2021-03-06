use crate::app::App;
use crate::lists::{DirectMessage, GroupInfo, MsgInfo};

use serde_json::Value;

use ureq;

use std::error::Error;

use textwrap::{fill, Options};

use tui::style::{Color, Style};
use tui::text::Text;

use chrono::prelude::*;

// Naive way to get rid of quotes
fn clean_str(input: String) -> String {
    let mut cleaned = input.clone();
    cleaned.pop();
    cleaned.remove(0);
    return cleaned;
}

pub fn get_userid(secret: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("https://api.groupme.com/v3/users/me?token={}", secret);
    let body: String = ureq::get(&url).call()?.into_string()?;
    let resp: Value = serde_json::from_str(&body)?;
    Ok(clean_str(resp["response"]["id"].to_string()))
}

/* Returns vector of GroupInfo to create a Lists object
 * of Groups
 * secret: GroupMe secret api key
 */
pub fn get_groups(secret: String) -> Result<Vec<GroupInfo>, Box<dyn Error>> {
    let mut groups = Vec::new();

    let url = format!(
        "https://api.groupme.com/v3/groups?token={}&omit=membership&per_page=100",
        secret
    );
    let body: String = ureq::get(&url).call()?.into_string()?;

    let resp: Value = serde_json::from_str(&body)?;

    let group_info = &resp["response"];

    for i in 0..group_info.as_array().unwrap().len() {
        let name = clean_str(group_info[i]["name"].to_string());
        let id = clean_str(group_info[i]["id"].to_string());
        let lmid = clean_str(group_info[i]["messages"]["last_message_id"].to_string());
        let message_count = group_info[i]["messages"]["count"].as_u64().unwrap();

        let group = GroupInfo {
            name,
            id,
            lmid,
            message_count,
        };

        groups.push(group);
    }

    Ok(groups)
}

/* Return vector of chats that can be used to create a List Object
 * app: Main application object
 */
pub fn get_chats(secret: String) -> Result<Vec<DirectMessage>, Box<dyn Error>> {
    let mut direct_messages = Vec::new();
    let url = format!(
        "https://api.groupme.com/v3/chats?token={}&per_page=100",
        secret
    );

    let body: String = ureq::get(&url).call()?.into_string()?;

    let resp: Value = serde_json::from_str(&body)?;

    let dm_info = &resp["response"];

    for i in 0..dm_info.as_array().unwrap().len() {
        let name = clean_str(dm_info[i]["other_user"]["name"].to_string());
        let id = clean_str(dm_info[i]["other_user"]["id"].to_string());

        let dm = DirectMessage { name, id };
        direct_messages.push(dm);
    }

    Ok(direct_messages)
}

/* Return vector of MsgInfo that can be used to create a List Object
 * app: Main application object
 */
pub fn get_messages(app: &mut App<'static>, dm: bool) -> Result<(), Box<dyn Error>> {
    let mut msgs = Vec::new();
    let url = if dm {
        format!(
            "https://api.groupme.com/v3/direct_messages?other_user_id={}&token={}",
            app.dm_id, app.secret
        )
    } else {
        format!(
            "https://api.groupme.com/v3/groups/{}/messages?token={}&limit=100",
            app.group_id, app.secret
        )
    };

    let body: String = ureq::get(&url).call()?.into_string()?;

    let resp: Value = serde_json::from_str(&body)?;

    let msg_info = if dm {
        &resp["response"]["direct_messages"]
    } else {
        &resp["response"]["messages"]
    };
    for i in (0..msg_info.as_array().unwrap().len()).rev() {
        let name = clean_str(msg_info[i]["name"].to_string());
        let mut text = if msg_info[i]["text"].is_null() {
            "".to_string()
        } else {
            clean_str(msg_info[i]["text"].to_string())
        };

        let num_likes = msg_info[i]["favorited_by"].as_array().unwrap().len();
        let mut liked = "???";
        for like in msg_info[i]["favorited_by"].as_array().unwrap() {
            if clean_str(like.to_string()) == app.user_id {
                liked = "???";
            }
        }
        let id = clean_str(msg_info[i]["id"].to_string());

        let mut attachments = Vec::new();
        if !msg_info[i]["attachments"].is_null() {
            for j in 0..msg_info[i]["attachments"].as_array().unwrap().len() {
                if msg_info[i]["attachments"][j]["type"] == "image" {
                    let attachment = clean_str(msg_info[i]["attachments"][j]["url"].to_string());
                    attachments.push(attachment);
                }
            }
        }

        let indentation = " ".repeat(2);
        let option = Options::new(app.t_width.into())
            .initial_indent(&indentation)
            .subsequent_indent(&indentation);

        let mut disp = Text::styled(
            format!("{} - {} {}", name.clone(), num_likes, liked),
            Style::default().fg(Color::Blue),
        );
        text = text.replace("\\n", "\n");
        text = text.replace("\\\"", "\"");
        if !text.is_empty() {
            text = fill(&text, &option);
            disp.extend(Text::raw(text.clone()));
        }
        for j in 0..attachments.len() {
            disp.extend(Text::from(indentation.clone() + &attachments[j]));
        }

        let message = MsgInfo {
            id,
            num_likes,
            display: disp.clone(),
            attachments,
            liked: liked == "???",
        };

        msgs.push(message);
        //msgs.push(message);
    }

    app.messages.set_items(msgs);

    Ok(())
}

pub fn send_message(
    secret: String,
    id: String,
    message: String,
    dm: bool,
) -> Result<(), Box<dyn Error>> {
    let url = if dm {
        format!(
            "https://api.groupme.com/v3/direct_messages?token={}&other_user_id={}",
            secret, id
        )
    } else {
        format!(
            "https://api.groupme.com/v3/groups/{}/messages?token={}",
            id, secret
        )
    };

    let guid = Local::now().timestamp().to_string();
    let msg_json = if dm {
        ureq::json!({
            "direct_message": {
                "source_guid": &guid,
                "recipient_id": id,
                "text": message,
                "attachments": []
            }
        })
    } else {
        ureq::json!({
            "message": {
                "source_guid": &guid,
                "text": message,
                "attachments": []
            }
        })
    };
    ureq::post(&url).send_json(msg_json)?;
    Ok(())
}

pub fn like_message(app: &mut App<'static>, dm: bool) -> Result<(), Box<dyn Error>> {
    let msg = &app.messages.items[app.messages.state.selected().unwrap()];
    let id = if dm { &app.dm_id } else { &app.group_id };
    if msg.liked == true {
        let url = format!(
            "https://api.groupme.com/v3/messages/{}/{}/unlike?token={}",
            id, msg.id, app.secret
        );
        ureq::post(&url).call()?;
    } else {
        let url = format!(
            "https://api.groupme.com/v3/messages/{}/{}/like?token={}",
            id, msg.id, app.secret
        );
        ureq::post(&url).call()?;
    }
    Ok(())
}
