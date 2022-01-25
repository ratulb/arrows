#sudo fuser -k -n tcp 7171

cargo run --example shutdown

ps -ef | grep arrows
sudo netstat -antup | grep 7171
