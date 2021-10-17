use arrows::{to_file, type_of, Actor, ActorBuilder, Address, Message, Ractor};
use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::BufWriter;

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
                let content: Input = deserialize(&content[..]).unwrap();
                println!("Actor received {:?}", content.arg);
                Output {
                    result: content.arg,
                }
            }
            _ => Output { result: "".to_string() },
        };
        Some(Message::Custom {
            from: None,
            to: None,
            content: Some(serialize(&output).unwrap()),
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
            content: Some(serialize(&test_input).unwrap()),
            recipients: None,
            created: std::time::SystemTime::now(),
        })
        .await;

    to_file(&reply, "reply.json").await;
    let content = reply.unwrap();
    let content = content.get_content();
    match content {
      Some(v) => {
        let v: String = deserialize(&v[..]).unwrap();
        println!("Reply from actor  = {}", v);
      },
      _ => println!("Failed to get reply"),
    }
    //create_reactor_test1().await;
    //create_addr_test1().await;
    //create_actor_builder_test().await;
    //send_complex_msg_test_1().await;
}

async fn create_reactor_test1() {
    fn receiver(_msg: Message) -> Option<Message> {
        None
    }
    let ractor1: Ractor = Ractor::new("ractor1", Box::new(receiver));
    type_of(&ractor1);
    println!("create_reactor_test1");
}

async fn create_addr_test1() {
    let message = Message::new(br#"This is a test message"#.to_vec(), "add1", "to");
    let _addr1 = Address::new("add1");
    to_file(message, "msg.json").await;
}

async fn create_actor_builder_test() {
    let mut actor_builder = ActorBuilder;
    let mut cloned_builder = actor_builder.clone();
    let message = Message::new(br#"This is a test message"#.to_vec(), "add1", "to");
    let mut cloned_msg = message.clone();

    cloned_builder.receive(message);
    println!("Original getting msg");
    // br#"e{"ddie"}"#.to_vec();
    cloned_msg.with_content(br#"This is brand new content"#.to_vec());
    actor_builder.receive(cloned_msg);
    println!("{:?}", actor_builder);
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
    let complex = serialize(&complex).unwrap();
    let msg = Message::new(complex, "addr_from", "addr_to");
    let cloned_msg = msg.clone();
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("msg.txt")
        .expect("Complex msg write failure");
    let mut bufwriter = BufWriter::new(file);
    assert_eq!(msg.write_sync(&mut bufwriter).expect("Should get ()"), ());
    let encoded: Vec<u8> = serialize(&cloned_msg).unwrap();

    let mut actor_builder = ActorBuilder;
    let message = Message::new(encoded.clone(), "add1", "to");
    actor_builder.receive(message);

    let decoded: Message = deserialize(&encoded[..]).unwrap();
    let decoded_cloned = decoded.clone();
    match decoded_cloned {
        Message::Custom { content, .. } => {
            if let Some(complex) = content {
                {
                    let complex_decoded: Complex<Inner> = deserialize(&complex[..]).unwrap();
                    println!("Inner = {:?}", complex_decoded.inner);
                    let elems = complex_decoded.elems;
                    println!("Elems {:?} ", elems);
                    type_of(&elems);
                    println!("Elems len {:?} ", elems.len());
                    println!("At position 0 {:?} ", elems[0]);
                }
            }
        }
        _ => (),
    }

    println!("===========================");
    println!("{:?} *****", decoded.get_content());
    println!("===========================");
    type_of(&decoded);
}
