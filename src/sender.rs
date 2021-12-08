use crate::{Addr, Msg};
use std::collections::HashMap;

#[macro_export]
macro_rules! send {
    ($($actor_name:literal, ($($msg:expr),*)),*)  => {
        let mut actor_msgs = HashMap::new();
            $(
                let addr = $crate::Addr::new($actor_name);
                let size = $crate::send![@SIZE; $($msg),*];
                let msgs = actor_msgs.entry(&addr)
                                     .or_insert(Vec::with_capacity(size));
                $(
                    let msg: $crate::Msg = $msg;
                    msgs.push(msg);
                )*
            )*
        $crate::recv(actor_msgs);
    };

    ($($actor_name:literal, $($msg:expr),*),*) => {
        let mut actor_msgs = HashMap::new();
            $(
                let addr = $crate::Addr::new($actor_name);
                let size = $crate::send![@SIZE; $($msg),*];
                let msgs = actor_msgs.entry(&addr)
                                     .or_insert(Vec::with_capacity(size));

                $(
                    let msg: $crate::Msg = $msg;
                    msgs.push(msg);
                )*
            )*
        $crate::recv(actor_msgs);
    };

    ($($addr:expr, ($($msg:expr),*)),*)  => {
        let mut actor_msgs = HashMap::new();
            $(
                let addr: $crate::Addr = $addr;
                let size = $crate::send![@SIZE; $($msg),*];
                let msgs = actor_msgs.entry(&addr)
                                     .or_insert(Vec::with_capacity(size));
                $(
                    let msg: $crate::Msg = $msg;
                    msgs.push(msg);
                )*
            )*
        $crate::recv(actor_msgs);
    };

    ($($addr:expr, $($msg:expr),*),*) => {
        let mut actor_msgs = HashMap::new();
            $(
                let addr: $crate::Addr = $addr;
                let size = $crate::send![@SIZE; $($msg),*];
                let msgs = actor_msgs.entry(&addr)
                                     .or_insert(Vec::with_capacity(size));
                $(
                    let msg: $crate::Msg = $msg;
                    msgs.push(msg);
                )*
            )*
        $crate::recv(actor_msgs);
    };

    (@SIZE; $($msg:expr),*) => {
        <[()]>::len(&[$($crate::send![@SUB; $msg]),*])
    };

    (@SUB; $_msg:expr) => {()};
}

pub(crate) fn recv(msgs: HashMap<&Addr, Vec<Msg>>) {
    for (k, v) in msgs.iter() {
        println!("Key({:?}) and msg count({:?})", k.get_name(), v.len());
    }
}
