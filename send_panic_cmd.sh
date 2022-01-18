#ARROWS_DB_PATH=/tmp cargo run --example send_panic_cmd
#sudo fuser -k -n tcp 7171

cargo run --example send_panic_cmd

ps -ef | grep 7171
ps -ef | grep arrows
sudo netstat -antup | grep 7171
