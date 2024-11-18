use std::sync::Arc;

use tokio::sync::{mpsc::Sender, Mutex};
use twitch_irc::message::PrivmsgMessage;

use std::hash::{DefaultHasher, Hasher};

use crate::{
    communication::TodoUpdate,
    lang::lang::{self, YOUR_TODOS},
};

use super::Data;

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
    Some(format_message_reply(data.lock().await.get(sender)))
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
    todo_subscribers: &Arc<Mutex<Vec<Sender<TodoUpdate>>>>,
) -> Option<String> {
    if let Some(text) = text {
        let mut data_locked = data.lock().await;
        match data_locked.get_mut(&msg.sender.login) {
            Some(s) => s.push(text.to_owned()),
            None => {
                data_locked.insert(msg.sender.login.clone(), vec![text.to_owned()]);
            }
        }

        let mut subscriber_lock = todo_subscribers.lock().await;

        let todo_update = TodoUpdate::AddTodo {
            user: msg.sender.login.clone(),
            uuid: hash_message(&msg.sender.login, &text),
            todo_message: text,
        };

        for subscriber in subscriber_lock.clone().into_iter() {
            let _ = subscriber.send(todo_update.clone()).await;
        }
        subscriber_lock.retain(|e| !e.is_closed());
        drop(subscriber_lock);
    }
    None
}

pub fn format_message_reply(todos: Option<&Vec<String>>) -> String {
    match todos {
        None => lang::NO_TODOS_ADDEDD_YET.to_owned(),
        Some(todos) => {
            let mut todos_str = String::new();
            let mut index = 1;
            for todo in todos {
                todos_str.push_str(format!("({}) ", index).as_str());
                todos_str.push_str(todo.as_str());
                todos_str.push(' ');
                index += 1;
            }
            format!("{YOUR_TODOS} {todos_str}")
        }
    }
}

pub async fn handle_check_command(
    text: Option<String>,
    data: Data,
    msg: &PrivmsgMessage,
    todo_subscribers: &Arc<Mutex<Vec<Sender<TodoUpdate>>>>,
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
            return Some(lang::TASK_INDEX_DOESNT_EXIST.to_owned());
        }
        let checked_todo = user_todos.remove(index);

        let mut subscriber_lock = todo_subscribers.lock().await;
        let todo_update = TodoUpdate::CheckTodo(hash_message(&msg.channel_login, &checked_todo));

        for subscriber in subscriber_lock.clone().into_iter() {
            let _ = subscriber.send(todo_update.clone()).await;
        }
        subscriber_lock.retain(|e| !e.is_closed());

        return Some(format!(
            "{} {} {checked_todo} {}",
            msg.sender.login,
            lang::FINISHED_TODO.0,
            lang::FINISHED_TODO.1
        ));
    }
    None
}

pub fn hash_message(name: &String, text: &String) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(text.as_bytes());
    hasher.write(name.as_bytes());
    hasher.finish()
}
