#!/bin/sh
rm -rf /tmp/logs/tests/
rm -rf /tmp/robust/tests/placement-center/data
rm -rf /tmp/robust/tests/placement-center/logs

placement_center_process_name="placement-center"
mqtt_server_process_name="mqtt-server"

# Start placement-center
nohup cargo run --package cmd --bin $placement_center_process_name -- --conf=tests/config/$placement_center_process_name.toml >/dev/null 2>&1 &
sleep 5
while ! ps aux | grep -v grep | grep "$placement_center_process_name" > /dev/null; do
    echo "Process $placement_center_process_name has not started yet, wait 1s...."
    sleep 1  
done
echo "Process $placement_center_process_name starts successfully and starts running the test case"

# Start mqtt-broker
nohup cargo run --package cmd --bin $mqtt_server_process_name -- --conf=tests/config/$mqtt_server_process_name.toml >/dev/null 2>&1 &
sleep 5
while ! ps aux | grep -v grep | grep "$mqtt_server_process_name" > /dev/null; do
    echo "Process $mqtt_server_process_name has not started yet, wait 1s...."
    sleep 1  
done
echo "Process $mqtt_server_process_name starts successfully and starts running the test case"

# Run Cargo Test
cargo test --package mqtt-broker

# Stop mqtt-broker
sleep 2
mqtt_no=`ps aux | grep -v grep | grep "$mqtt_server_process_name" | awk '{print $2}'`
echo "mqtt server num: $mqtt_no"
kill $mqtt_no
sleep 3

while ps aux | grep -v grep | grep "$mqtt_server_process_name" > /dev/null; do
    echo "”Process $mqtt_server_process_name stopped successfully"
    sleep 1  
done

# Stop placement-center
sleep 2
pc_no=`ps aux | grep -v grep | grep "$placement_center_process_name" | awk '{print $2}'`
echo "placement center num: $pc_no"
kill $pc_no
sleep 3

while ps aux | grep -v grep | grep "$placement_center_process_name" > /dev/null; do
    echo "”Process $placement_center_process_name stopped successfully"
    sleep 1  
done