# arrows
Arrows - asynchrous actor framework in rust with message persistence and actor resurrection.

Work in progress.
1) Crash recovery(WIP)
2) Remoting(No peer awareness - other systems should up or remote delivery fails and get retried - at system start up)
3) Actor behaviour can be swapped out at runtime with another actor definition(Actor binaries has to be available in system - no runtime injection of bi-naries)
4) Swapped in actor - resumes from where the swapped out instance left off.
5) Actor loading/Unloading is based on typetag(A huge thanks to in
6) Binany(serde + bincode) or Text messages
7) Message durability is intrinsic(can not be opted out of). Based on fast sqlite embedded instance 
8) Multiple instances of the same actor - with different named identifier
9) Resume message consumption left of state on system restart
10) No out of sequence delivery of messages 
11) Macro for defining actor(define_actor!)
12) Macro for sending message(s) to actor(s) with one single macro(send!)
13) Panicking Actor ejection(flagging of payload)
14) Sequencing of incoming user/actor messages
15) Span out processing of received messages 
16) Ingress gateway
17) Messenger responsible sending out outgoing messages
18) 
