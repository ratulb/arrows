arg=$1
if [ "true" == "$arg" ]; then
  ARROWS_DB_PATH=/tmp cargo test -- --nocapture
else
  ARROWS_DB_PATH=/tmp cargo test
fi
