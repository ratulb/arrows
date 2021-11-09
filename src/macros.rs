#[macro_export]
macro_rules! register_actor {
    ($actor_name:literal, $actor_builder:path) => {
        //use crate::registry::registry::register;
        //let mut builder: Box<dyn ActorBuilder> = Box::new($actor_builder);
        let identity = $crate::Addr::new($actor_name).get_id();
        $crate::registry::registry::register(identity, $actor_builder);
    };
}

#[cfg(test)]
mod tests {
    use crate::{Actor, ActorBuilder, Msg};
    use serde::{Deserialize, Serialize};

    pub struct NewActor;

    impl Actor for NewActor {
        fn receive(&mut self, _incoming: Msg) -> std::option::Option<Msg> {
            Some(Msg::new_with_text("Reply from new actor", "from", "to"))
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
        let actor_builder = NewActorBuilder::default();
        register_actor!("new_actor", actor_builder);
        let identity = crate::Addr::new("new_actor").get_id();
        use crate::registry::registry::send;
        send(identity, Msg::Blank);
    }
}
