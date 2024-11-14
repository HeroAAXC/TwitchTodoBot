use twitch_irc::message::PrivmsgMessage;

use crate::Data;

pub async fn handle_list_todos(
    text: Option<String>,
    data: Data,
    msg: &PrivmsgMessage,
) -> Option<String> {
    let sender = match text {
        None => &msg.sender.login,
        Some(s) => {
            let s = s.replace(" ", "");
            if s != "".to_owned() {
                &s.clone()
            } else {
                &msg.sender.login
            }
        }
    };
    Some(format_message_reply(msg, data.lock().await.get(sender)))
}

pub fn split_command_message(message: String) -> (String, Option<String>) {
    return if message.contains(" ") {
        let (cmd, text) = message.split_once(" ").unwrap();
        (cmd[1..].to_owned(), Some(text.to_owned()))
    } else {
        (message[1..].to_owned(), None)
    };
}

pub async fn handle_add_todo(
    text: Option<String>,
    data: Data,
    msg: &PrivmsgMessage,
) -> Option<String> {
    if let Some(text) = text {
        let mut data_locked = data.lock().await;
        match data_locked.get_mut(&msg.sender.login) {
            Some(s) => s.push(text.to_owned()),
            None => {
                data_locked.insert(msg.sender.login.clone(), vec![text.to_owned()]);
            }
        }
    }
    None
}

pub fn format_message_reply(msg: &PrivmsgMessage, todos: Option<&Vec<String>>) -> String {
    match todos {
        None => {
            format!("@{} Du hast noch keine todos hinzugefügt, du kannst mit !todo <Nachricht> todos speichern", msg.sender.login)
        }
        Some(todos) => {
            let mut todos_str = String::new();
            let mut index = 1;
            for todo in todos {
                todos_str.push_str(format!("({}) ", index).as_str());
                todos_str.push_str(todo.as_str());
                todos_str.push(' ');
                index += 1;
            }
            format!("@{} Du hast folgende todos: {todos_str}", msg.sender.login)
        }
    }
}

pub async fn handle_check_command(
    text: Option<String>,
    data: Data,
    msg: &PrivmsgMessage,
) -> Option<String> {
    let index: usize = match text {
        Some(s) => match s.parse() {
            Ok(r) => {
                if r > 0 {
                    r - 1
                } else {
                    r
                }
            }
            Err(_) => 0,
        },
        None => 0,
    };
    if let Some(user_todos) = data.lock().await.get_mut(&msg.sender.login) {
        if index >= user_todos.len() {
            return Some("Die Tasknummer existiert nicht :( (Bitte beachte, dass beim Löschen die Nummern weiter rutschen)".to_owned());
        }
        let checked_todo = user_todos.remove(index);
        return Some(format!(
            "{} hat {checked_todo} geschafft :D ",
            msg.sender.login
        ));
    }
    None
}
