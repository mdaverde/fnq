#!/usr/bin/env bash

set -e
unset FNQ_DIR

: ${FNQ:="cargo --quiet run --"}

check() {
  msg=$1
  shift
  if eval "$@" 2>/dev/null 1>&2; then
    printf 'ok - %s\n' "$msg"
  else
    printf 'not ok - %s\n' "$msg"
    false
  fi
  true
}

setup() {
  if [ -d test.dir ]; then
    rm -rf test.dir
  fi
  mkdir test.dir
  cd test.dir
}

teardown() {
  cd ..
  rm -rf test.dir
}

setup
(
printf '# fnq tests\n'
check 'fails with no arguments' ! $FNQ
check 'succeeds enqueueing true' 'f=$($FNQ true)'
sleep 1
check 'generated a lockfile' test -f $f
check 'lockfile contains exec line' grep -q exec.*fnq.*true $f
check 'lockfile contains status line' grep -q exited.*status.*0 $f
check 'lockfile is not executable' ! test -x $f
)
teardown

setup
(
  printf '\n# queue tests \n'
  check 'enqueueing true' f1=$($FNQ true)
  check 'enqueueing sleep 500' f2=$($FNQ sleep 500)
  check 'first job is done already' $FNQ --tap $f1
  check 'running job is executable' test -x $f2
  check 'running job not done already' ! $FNQ --tap $f2
  check 'can kill running job' kill ${f2##*.}
  sleep 1
  check 'killed job is not executable anymore' ! test -x $f2
  check 'killed job contains status line' grep -q killed.*signal.*SIGTERM $f2
)
teardown


setup
(
printf "\n# env tests\n"
check 'enqueueing env' f1=$($FNQ env)
$FNQ --wait
check 'FNQJOBID is set' grep -q FNQJOBID=$f1 $f1
)
teardown

setup
(
printf '\n# killing tests\n'
check 'spawning four jobs' 'f1=$($FNQ sleep 100)'
check 'spawning four jobs' 'f2=$($FNQ sleep 1)'
check 'spawning four jobs' 'f3=$($FNQ sleep 100)'
check 'spawning four jobs' 'f4=$($FNQ sleep 1)'
check 'killing first job' kill ${f1##*.}
check 'killing third job' kill ${f3##*.}
check 'second job is running' ! $FNQ --tap $f2
$FNQ --wait $f2
check 'fourth job is running' ! $FNQ --tap $f4
check 'all jobs are done' $FNQ --wait
)
teardown

setup
(
printf '\n# --watch tests\n'
check 'spawning four jobs' 'f1=$($FNQ sleep 100)'
check 'spawning four jobs' 'f2=$($FNQ echo two)'
check 'spawning four jobs' 'f3=$($FNQ sleep 300)'
check 'spawning four jobs' 'f4=$($FNQ sleep 400)'
check '--watch tracks first job' '($FNQ --watch fnq* & p=$!; sleep 1; kill $p) | sed 3q | grep -q sleep.*100'
check 'killing first job' kill ${f1##*.}
check 'killing fourth job' kill ${f4##*.}
sleep 1
check '--watch tracks third job' '($FNQ --watch fnq* & p=$!; sleep 1; kill $p) | sed 3q | grep -q sleep.*300'
check 'killing third job' kill ${f3##*.}
sleep 1
# check '--watch outputs last job when no job running' '$FNQ --watch fnq* | sed 3q | grep -q sleep.*400'
)
teardown


