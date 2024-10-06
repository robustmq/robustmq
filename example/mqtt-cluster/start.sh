no1=`ps -ef | grep placement-center  | grep node-1 | grep -v grep | awk '{print $2}'`
if [[ -n $no1 ]]
then
    echo "kill $no1"
    kill $no1
fi

no2=`ps -ef | grep placement-center  | grep node-2 | grep -v grep | awk '{print $2}'`
if [[ -n $no2 ]]
then
    echo "kill $no2"
    kill $no2
fi

no3=`ps -ef | grep placement-center  | grep node-3 | grep -v grep | awk '{print $2}'`
if [[ -n $no3 ]]
then
    echo "kill $no3"
    kill $no3
fi

sleep 3

nohup cargo run --package cmd --bin placement-center -- --conf=example/mqtt-cluster/placement-center/node-1.toml 2>/tmp/1.log &
nohup cargo run --package cmd --bin placement-center -- --conf=example/mqtt-cluster/placement-center/node-2.toml 2>/tmp/2.log &
nohup cargo run --package cmd --bin placement-center -- --conf=example/mqtt-cluster/placement-center/node-3.toml 2>/tmp/3.log &
sleep 3

no1=`ps -ef | grep placement-center  | grep node-1 | grep -v grep | awk '{print $2}'`
if [[ -n $no1 ]]
then
    echo "placement-center node 1 started successfully. process no: $no1"
fi

no2=`ps -ef | grep placement-center  | grep node-2 | grep -v grep | awk '{print $2}'`
if [[ -n $no2 ]]
then
    echo "placement-center node 2 started successfully. process no: $no2"
fi

no3=`ps -ef | grep placement-center  | grep node-3 | grep -v grep | awk '{print $2}'`
if [[ -n $no3 ]]
then
    echo "placement-center node 3 started successfully. process no: $no3"
fi

echo "\nNode 1:"
resp1=$(curl -s http://127.0.0.1:2217/v1/cluster/status)
role=$(echo $resp1 | jq -r '.data.Ok.state')
echo "Role:"${role}
echo ${resp1}

echo "\n-------------------------------------\n"

echo "\nNode 2:"
resp2=$(curl -s http://127.0.0.1:2227/v1/cluster/status)
role=$(echo $resp2 | jq -r '.data.Ok.state')
echo "Role:"${role}
echo ${resp2}

echo "\n-------------------------------------\n"
echo "\nNode 3:"
resp3=$(curl -s http://127.0.0.1:2237/v1/cluster/status)
role=$(echo $resp3 | jq -r '.data.Ok.state')
echo "Role:"${role}
echo ${resp3}

echo "\n-------------------------------------\n"

membership=$(echo $resp3 | jq -r '.data.Ok.membership_config.membership.configs[0]')
nodes=$(echo $resp3 | jq -r '.data.Ok.membership_config.membership.nodes')
current_leader=$(echo $resp3 | jq -r '.data.Ok.current_leader')
echo "Placement Center cluster Leader: "${current_leader}
echo "Placement Center cluster Members: "${membership}
echo "Placement Center cluster Nodes: "${nodes}


sleep 10
