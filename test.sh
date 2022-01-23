arg=$1
if [ "true" == "$arg" ]; then
  cargo test -- --nocapture
else
  cargo test
fi
