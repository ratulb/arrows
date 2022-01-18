#sudo fuser -k -n tcp 7171

cargo run --example shutdown
rm /tmp/arrows.db
ps -ef | grep arrows
sudo netstat -antup | grep 7171
