# arrows
Arrows - a fast, lightweight, intuitive actor framework in rust. Currently has the following functionalities:

1) Crash recovery(WIP)
2) Remoting(No peer awareness - other systems should be up or remote delivery fails and gets retried at system start up)
3) Actor behaviour can be swapped out at runtime with another actor definition(Actor binaries has to be available in system - no runtime injection of bi-naries)
4) Swapped in actor resumes from where the swapped out instance left off.
5) Actor loading/Unloading is based on typetag(https://github.com/dtolnay/typetag). A huge thanks to the ingenuity of its author https://github.com/dtolnay
6) Binany(serde + bincode) or Text messages
7) Message durability is intrinsic(can not be opted out of). Based on fast sqlite embedded instance.
8) Multiple instances of the same actor - with different named identifier
9) No out of sequence delivery of messages 
10) Macro for defining actor(define_actor!)
11) Macro for sending message(s) to actor(s) with one single macro(send!)
12) Panicking Actor ejection(flagging of payload)
13) Sequencing of incoming user/actor messages
14) Spread out processing of received messages 

