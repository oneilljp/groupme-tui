use crate::api::*;
use crate::lists::*;

pub struct App<'a> {
    pub groups: Lists<GroupInfo>,
    pub dms: Lists<DirectMessage>,
    pub messages: Lists<MsgInfo<'a>>,
    pub user_id: String,
    pub group_id: String,
    pub dm_id: String,
    pub input: String,
    pub input_pos: usize,
    pub secret: String,
    pub t_width: u16,
    pub mode: Modes,
    pub disp: DispMode,
    pub dm: bool,
}

#[derive(PartialEq)]
pub enum Modes {
    GroupNav,
    DirectNav,
    MessageNav,
    Inputting,
}

#[derive(PartialEq)]
pub enum DispMode {
    Main,
    Help,
    Startup,
}

impl App<'static> {
    pub fn new(secret: String, t_width: u16) -> App<'static> {
        let groups = Lists::new(get_groups(secret.to_string()).unwrap(), true);
        let dms = Lists::new(get_chats(secret.to_string()).unwrap(), true);
        let group_id = groups.items[groups.state.selected().unwrap()].id.clone();
        let dm_id = dms.items[dms.state.selected().unwrap()].id.clone();
        let mut app = App {
            groups,
            dms,
            messages: Lists::new(Vec::new(), false),
            user_id: get_userid(&secret).unwrap(),
            group_id,
            dm_id,
            secret,
            input: "▏".to_string(),
            input_pos: 0,
            t_width,
            mode: Modes::GroupNav,
            disp: DispMode::Startup,
            dm: false,
        };
        get_messages(&mut app, false).unwrap();
        app.messages.previous();
        app
    }

    /* Used to update internal message List */
    pub fn update_msgs(&mut self) {
        self.group_id = self.groups.items[self.groups.state.selected().unwrap()]
            .id
            .clone();
        get_messages(self, false).unwrap();
        self.messages.previous();
    }

    /* Update internal message list with direct messages */
    pub fn update_dmsgs(&mut self) {
        self.dm_id = self.dms.items[self.dms.state.selected().unwrap()]
            .id
            .clone();
        get_messages(self, true).unwrap();
        self.messages.previous();
    }

    /* Send message stored in Input to group stored in GroupID, Updates Stored Messages, and Clears
     * Input
     */
    pub fn send_msg(&mut self) {
        self.group_id = self.groups.items[self.groups.state.selected().unwrap()]
            .id
            .clone();
        send_message(
            self.secret.to_string(),
            self.group_id.to_string(),
            self.input.to_string(),
            false,
        )
        .unwrap();
        self.update_msgs();
        self.input.clear();
    }

    pub fn send_dmsg(&mut self) {
        self.dm_id = self.dms.items[self.dms.state.selected().unwrap()]
            .id
            .clone();
        send_message(
            self.secret.to_string(),
            self.dm_id.to_string(),
            self.input.to_string(),
            true,
        )
        .unwrap();
        self.update_dmsgs();
        self.input.clear();
    }

    /* Like/unlike selected message through API */
    pub fn like(&mut self) {
        self.group_id = self.groups.items[self.groups.state.selected().unwrap()]
            .id
            .clone();
        like_message(self, false).unwrap();
        self.update_msgs();
    }

    pub fn dlike(&mut self) {
        self.dm_id = self.dms.items[self.dms.state.selected().unwrap()]
            .id
            .clone();
        like_message(self, true).unwrap();
        self.update_dmsgs();
    }
}
