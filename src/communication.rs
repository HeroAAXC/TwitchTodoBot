pub struct BotMessage {
    pub reciever: Option<String>,
    pub message: String,
    pub channel: String,
}

#[derive(Clone)]
pub enum TodoUpdate {
    AddTodo {
        user: String,
        todo_message: String,
        uuid: u64,
    },
    CheckTodo(u64),
}
