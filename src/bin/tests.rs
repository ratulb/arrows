use arrows::{to_file, type_of, Address, Message, Ractor};
use serde::{Deserialize, Serialize};

#[async_std::main]
pub async fn main() {
    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct Input {
        arg: String,
    }
    let test_input = Input {
        arg: "just testing".to_string(),
    };
    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct Output {
        result: usize,
    }
    let invokable = |param: Message<Input>| -> Option<Message<Output>> {
        let output = match param {
            Message::Custom {
                from: _,
                to: _,
                content,
                recipients: _,
                created: _,
            } => {
                println!("Received arg: {:?}", content);
                Output {
                    result: content.unwrap().arg.len(),
                }
            }
            _ => Output { result: 0 },
        };
        Some(Message::Custom {
            from: None,
            to: None,
            content: Some(output),
            recipients: None,
            created: std::time::SystemTime::now(),
        })
    };
    let boxed_invokable = Box::new(invokable);
    let mut actor1 = Ractor::new("actor1", boxed_invokable);

    let reply = actor1
        .receive(Message::Custom {
            from: None,
            to: None,
            content: Some(test_input),
            recipients: None,
            created: std::time::SystemTime::now(),
        })
        .await;

    println!("The reply type");
    type_of(&reply);
    to_file(&reply, "reply.json").await;
    //let from_file = from_file::<Message<Output>>("reply.json").await.unwrap();
    //println!("At the end - from_file -> {:?}", from_file);
    create_reactor_test1().await;
    create_addr_test1().await;
}

async fn create_reactor_test1() {
    fn receiver<T, R>(_msg: Message<T>) -> Option<Message<R>>
    where
        T: Serialize,
        R: Serialize,
    {
        None
    }
    let ractor1: Ractor<String, bool> = Ractor::new("ractor1", Box::new(receiver));
    type_of(&ractor1);
    println!("create_reactor_test1");
}

async fn create_addr_test1() {
    let message = Message::<&str>::new("This is a test message", "add1", "to");
    let _addr1 = Address::new("add1");
    to_file(message, "msg.json").await;
}
