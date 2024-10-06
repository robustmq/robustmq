stop_pc_cluster(){
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

    rm -rf  /tmp/robust/placement-center-1/data
    rm -rf  /tmp/robust/placement-center-2/data
    rm -rf  /tmp/robust/placement-center-3/data
}

stop_pc_cluster