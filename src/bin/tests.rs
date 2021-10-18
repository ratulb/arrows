use arrows::{
    from_bytes, option_of_bytes, to_file, type_of, Actor, ActorBuilder, Address, Message, Ractor,
};
use bincode::serialize;
use serde::{Deserialize, Serialize};

#[async_std::main]
pub async fn main() {
    create_actor_from_from_fn_test1().await;
    write_addr_test1().await;
    //create_actor_builder_test_cloning().await;
    send_complex_msg_test_1().await;
}

async fn actor_test_with_closure() {
    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct Input {
        arg: String,
    }
    let test_input = Input {
        arg: "just testing".to_string(),
    };
    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct Output {
        result: String,
    }
    let invokable = |param: Message| -> Option<Message> {
        let output = match param {
            Message::Custom {
                from: _,
                to: _,
                content,
                recipients: _,
                created: _,
            } => {
                let content = content.unwrap();
                let content: Input = from_bytes(&content).unwrap();
                println!("Actor received {:?}", content.arg);
                Output {
                    result: content.arg,
                }
            }
            _ => Output {
                result: "".to_string(),
            },
        };
        Some(Message::Custom {
            from: None,
            to: None,
            content: option_of_bytes(&output),
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
            content: option_of_bytes(&test_input),
            recipients: None,
            created: std::time::SystemTime::now(),
        })
        .await;

    to_file(&reply, "reply.json").await;
    type_of(&reply);
    let message = reply.expect("Should contain message");
    if let Some(content) = message.get_content() {
        let actor_reply: String = from_bytes(content).expect("Should deserialize");
        println!("Actor reply -> {}", actor_reply);
    }
}

async fn create_actor_from_from_fn_test1() {
    fn receiver(_msg: Message) -> Option<Message> {
        None
    }
    let ractor1: Ractor = Ractor::new("ractor1", Box::new(receiver));
    type_of(&ractor1);
    println!("create_actor_from_from_fn_test1");
}

async fn write_addr_test1() {
    let message = Message::new(option_of_bytes("This some string"), "add1", "to");
    let _addr1 = Address::new("add1");
    to_file(message, "msg.json").await;
}

async fn create_actor_builder_test_cloning() {
    let mut actor_builder = ActorBuilder;
    let input = "This is a test message";
    let input_vectorized = option_of_bytes(&input);
    let message = Message::new(input_vectorized, "add1", "to");

    let cloned_message = message.clone();
    let mut message_updated = message.clone();
    let mut cloned_builder = actor_builder.clone();

    cloned_builder.receive(cloned_message);
    actor_builder.receive(message);

    message_updated.with_content(option_of_bytes("This is brand new content").unwrap());
    cloned_builder.receive(message_updated);
}

async fn send_complex_msg_test_1() {
    #[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
    struct Complex<T> {
        inner: T,
        elems: Vec<Simple>,
    }
    #[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
    struct Inner {
        name: String,
        children: Vec<String>,
        male: bool,
        age: u8,
    }
    #[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
    struct Simple {
        e1: i32,
        e2: usize,
        e3: Option<bool>,
    }
    let simple = Simple {
        e1: 42,
        e2: 999,
        e3: Some(false),
    };

    let inner = Inner {
        name: "Some body".to_string(),
        children: vec!["Some value".to_string()],
        male: true,
        age: 99,
    };

    let complex = Complex {
        inner,
        elems: vec![simple],
    };
    let complex = option_of_bytes(&complex);
    let msg = Message::new(complex, "addr_from", "addr_to");

    let mut actor_builder = ActorBuilder;
    actor_builder.receive(msg);
}
