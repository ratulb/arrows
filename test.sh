arg=$1
if [ "true" == "$arg" ]; then
   DB_PATH=/tmp cargo test -- --nocapture
else
  DB_PATH=/tmp cargo test
fi
