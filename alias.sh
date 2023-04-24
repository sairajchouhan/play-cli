play_wrapper () {
  TMPFILE=$(mktemp);
  trap 'rm -f $TMPFILE' EXIT;

  first_arg=$1;
  to_compare="new";

  if [[ "$first_arg" == "$to_compare" ]]; then
    if /Users/sairaj.chouhan/mine/play-cli/target/debug/play $@ --tempfile $TMPFILE; then
      if [ -e "$TMPFILE" ]; then
        FIXED_CMD=$(cat $TMPFILE);
        echo "Running $FIXED_CMD...";
        eval "$FIXED_CMD"
      else
        echo "Apologies! Extracting command failed"
      fi  
    else
      return 1
    fi
  else
    /Users/sairaj.chouhan/mine/play-cli/target/debug/play $@;
  fi
};


alias play='play_wrapper $@'

cargo build -q

play
