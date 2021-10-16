use arrows::{from_file, to_file, type_of, Actor, Address, AddressMode, Message};
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
            Message::Business {
                from: _,
                to: _,
                content,
                created: _,
                signature: _,
                addressing: _,
            additional_recipients: _,
            } => {
                println!("Received arg: {:?}", content);
                Output {
                    result: content.unwrap().arg.len(),
                }
            }
            _ => Output { result: 0 },
        };
        Some(Message::Business {
            from: None,
            to: None,
            content: Some(output),
            created: std::time::SystemTime::now(),
            signature: None,
            addressing: AddressMode::default(),
            additional_recipients: None,
        })
    };
    let boxed_invokable = Box::new(invokable);
    let mut actor1 = Actor::new("actor1", boxed_invokable);

    let reply = actor1
        .receive(Message::Business {
            from: None,
            to: None,
            content: Some(test_input),
            created: std::time::SystemTime::now(),
            signature: None,
            addressing: AddressMode::default(),
            additional_recipients: None,
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
    fn receiver<T, R>(msg: Message<T>) -> Option<Message<R>>
    where
        T: Serialize,
        R: Serialize,
    {
        None
    }
    let ractor1: Actor<String, bool> = Actor::new("ractor1", Box::new(receiver));
    type_of(&ractor1);
    println!("create_reactor_test1");
}

async fn create_addr_test1() {
    let mut message = Message::new("This is a test message", "add1", "to");
    let additional_recipients = Address::addresses_of(&["addr2","addr3"]);
    let addr1 = Address::new("add1");
    message.with_recipient("to all");
   // std::mem::replace(&mut message.additional_recipients, Some(additional_recipients));
    to_file(message, "msg.json").await;
}
