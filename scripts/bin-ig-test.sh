build_server(){
    make build
}

build_server
sleep 5

cd build
tar -xzvf robustmq-0.1.14.tar.gz
cd robustmq-0.1.14
pwd
bin/robust-server place stop
bin/robust-server mqtt stop
bin/robust-server journal stop
sleep 10

bin/robust-server place start
sleep 10
bin/robust-server mqtt start
sleep 10
bin/robust-server journal start

# place
cargo nextest run --package grpc-clients --package robustmq-test --test mod -- placement && \
cargo nextest run --package robustmq-test --test mod -- place_server && \
cargo nextest run --package storage-adapter --lib -- placement

# journal
cargo nextest run  --package grpc-clients --test mod -- journal && \
cargo nextest run  --package robustmq-test --test mod -- journal_client && \
cargo nextest run  --package robustmq-test --test mod -- journal_server

sleep 10
bin/robust-server place stop
bin/robust-server mqtt stop
bin/robust-server journal stop