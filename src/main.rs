use tokio::sync::{mpsc, oneshot};

enum Message {
    GetUniqueId { respond_to: oneshot::Sender<u32> },
}

struct Actor {
    receiver: mpsc::Receiver<Message>,
    next_id: u32,
}

impl Actor {
    fn new(receiver: mpsc::Receiver<Message>) -> Self {
        Actor { receiver, next_id: 0 }
    }

    fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::GetUniqueId { respond_to } => {
                self.next_id += 1;
                let _ = respond_to.send(self.next_id);
            }
        }
    }
}

async fn run_actor(mut actor: Actor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg);
    }
}

#[derive(Clone)]
pub struct Handle {
    sender: mpsc::Sender<Message>,
}

impl Handle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = Actor::new(receiver);
        tokio::spawn(run_actor(actor));

        Self { sender }
    }

    pub async fn get_unique_id(&self) -> u32 {
        let (send, recv) = oneshot::channel();
        let msg = Message::GetUniqueId { respond_to: send };

        let _ = self.sender.send(msg).await;
        recv.await.unwrap()
    }
}

#[tokio::main]
async fn main() {
    let handle = Handle::new();

    for _ in 0..10 {
        let id = handle.get_unique_id().await;
        println!("next id is {id}");
    }
}
