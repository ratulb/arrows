#[macro_export]
macro_rules! define_actor {
    ($actor_name:literal, $actor_builder:path) => {{
        let identity = $crate::Addr::new($actor_name).get_id();
        let addr = $crate::Addr::new($actor_name);
        let res = $crate::catalog::define_actor(identity, addr, $actor_builder);
        res
    }};
    ($actor_addr:expr, $actor_builder:path) => {{
        let actor_addr: $crate::Addr = $actor_addr;
        let identity = actor_addr.get_id();
        let res = $crate::catalog::define_actor(identity, actor_addr, $actor_builder);
        res
    }};
}

#[macro_export]
macro_rules! send_to {
    ($actor_name:literal, $msg:expr) => {
        let msg: $crate::Msg = $msg;
        let identity = $crate::Addr::new($actor_name).get_id();
        $crate::catalog::send(identity, msg);
    };
    ($actor_addr:expr, $msg:expr) => {
        let msg: $crate::Msg = $msg;
        let actor_addr: $crate::Addr = $actor_addr;
        let identity = actor_addr.get_id();
        $crate::catalog::send(identity, msg);
    };
}

#[macro_export]
macro_rules! send {
    ($($actor_name:literal, ($($msg:expr),*)),*)  => {
        $crate::send!(@DELEGATE; $($crate::send!(@TO_ADDR; $actor_name), ($($msg),*)),*);
    };

    ($($actor_name:literal, $($msg:expr),*),*) => {
        $crate::send!(@DELEGATE; $($crate::send!(@TO_ADDR; $actor_name), ($($msg),*)),*);
    };

    ($($addr:expr, ($($msg:expr),*)),*)  => {
        $crate::send!(@DELEGATE; $($addr, ($($msg),*)),*);
    };

    ($($addr:expr, $($msg:expr),*),*) => {
        $crate::send!(@DELEGATE; $($addr, ($($msg),*)),*);
    };

    (@DELEGATE; $($addr:expr, ($($msg:expr),*)),*) => {
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

    (@TO_ADDR; $actor_name:literal) => {
        $crate::Addr::new($actor_name)
    };
}

/***pub(crate) fn recv(msgs: HashMap<&Addr, Vec<Msg>>) {
    for (k, v) in msgs.iter() {
        println!("Key({:?}) and msg count({:?})", k.get_name(), v.len());
    }
}***/

#[cfg(test)]
mod tests {
    use crate::{Actor, ActorBuilder, Addr, Mail, Msg};
    use serde::{Deserialize, Serialize};

    pub struct NewActor;

    impl Actor for NewActor {
        fn receive(&mut self, _incoming: Mail) -> std::option::Option<Mail> {
            Some(Msg::new_with_text("Reply from new actor", "from", "to").into())
        }
    }

    #[derive(Debug, Serialize, Deserialize, Default)]
    struct NewActorBuilder;

    #[typetag::serde(name = "new_actor_builder")]
    impl ActorBuilder for NewActorBuilder {
        fn build(&mut self) -> Box<dyn Actor> {
            Box::new(NewActor)
        }
    }
    #[test]
    fn macro_register_actor_test1() {
        let builder = NewActorBuilder::default();
        define_actor!("new_actor", builder);

        let builder = NewActorBuilder::default();
        define_actor!(Addr::new("new_actor"), builder);

        let builder = NewActorBuilder::default();
        let addr = Addr::new("new_actor");
        define_actor!(addr, builder);

        send_to!("new_actor", Msg::default());
        send_to!(Addr::new("new_actor"), Msg::default());
        let addr = Addr::new("new_actor");
        send_to!(addr, Msg::default());
    }
}
