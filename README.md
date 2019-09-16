This is an example reproduction of a TCP client connection leak

Instructions:
* cargo run
* open 127.0.0.1:3030/rir/0/5 from a browser - you should see [0, 0, 119, 0, 0]
* The TCP connection opened on listener port 502 will stay open (can verify with ```netstat -tn | grep 502```)
* repeated requests to the /rir endpoint will continue to leak more TCP connections



