#!/usr/bin/env bash
#

cleanup() {
	echo "CTRL+C pressed";
	logpid=`ps -aux | grep -Pi "tail \-f log.txt" | head -n 1 | awk '{print $2}'`

	echo handling log PID "$logpid"
	pkill termite
}

trap cleanup SIGINT 

./log.sh & cargo run --bin termite --release -- sample.rs --log log.txt
