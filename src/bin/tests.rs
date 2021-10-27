use arrow_commons::utils::{from_bytes, option_of_bytes, type_of};
use arrows::etc::Ractor;
use arrows::{Actor, Message};
use serde::{Deserialize, Serialize};

#[async_std::main]
pub async fn main() {
    system_startup_test().await;
    //actor_test_with_closure().await;
    //create_actor_from_from_fn_test1().await;
    //send_msg_within_msg_test_1().await;
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
                id: _,
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
            id: 1000,
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
            id: 1000,
            from: None,
            to: None,
            content: option_of_bytes(&test_input),
            recipients: None,
            created: std::time::SystemTime::now(),
        })
        .await;

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

async fn send_msg_within_msg_test_1() {
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
    let complex_as_opt = option_of_bytes(&complex);
    let msg = Message::new(complex_as_opt, "addr_from", "addr_to");
    let mut msg_container = Message::new(option_of_bytes(&msg), "addr_from", "addr_to");

    struct NewActor;

    impl Actor for NewActor {
        fn receive(&mut self, msg: &mut Message) -> Option<Message> {
            //println!("New actor received msg ->");
            //println!();
            //println!("{:?}", msg);
            let msg = msg;
            let inner_msg_option: Option<Vec<u8>> = msg.get_content_out();
            let inner_vec = inner_msg_option.unwrap();
            let mut inner_msg: Message = from_bytes(&inner_vec).ok().unwrap();
            let inner_content_option = inner_msg.get_content_out();
            let bytes_for_complex = inner_content_option.unwrap();
            let nested_complex: Complex<Inner> = from_bytes(&bytes_for_complex).ok().unwrap();

            //println!("The nested complex: {:?}", nested_complex);
            let returned_complex_bytes = option_of_bytes(&nested_complex);
            let returned_msg = Message::new(returned_complex_bytes, "addr_from", "addr_to");
            Some(returned_msg)
        }
    }
    let mut new_actor = NewActor;
    let call_result = new_actor.receive(&mut msg_container);
    let call_result = call_result.unwrap().get_content_out().unwrap();
    let result_complex: Complex<Inner> = from_bytes(&call_result).ok().unwrap();
    assert_eq!(result_complex, complex);
}

async fn system_startup_test() {
    let _result = arrows::start().await;
}
