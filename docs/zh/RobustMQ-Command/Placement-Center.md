# Placement Center Command

## 1. Placement状态

查看Placement集群的运行信息。

```
$ bin/robust-ctl place status
{"running_state":{"Ok":null},"id":1,"current_term":1,"vote":{"leader_id":{"term":1,"node_id":1},"committed":true},"last_log_index":28,"last_applied":{"leader_id":{"term":1,"node_id":1},"index":28},"snapshot":null,"purged":null,"state":"Leader","current_leader":1,"millis_since_quorum_ack":0,"last_quorum_acked":1742005289409447084,"membership_config":{"log_id":{"leader_id":{"term":0,"node_id":0},"index":0},"membership":{"configs":[[1]],"nodes":{"1":{"node_id":1,"rpc_addr":"localhost:1228"}}}},"heartbeat":{"1":1742005289032346459},"replication":{"1":{"leader_id":{"term":1,"node_id":1},"index":28}}}
```

## 2. 添加学习者

为Placement集群扩展添加一个新节点作为学习者。

```
$ bin/robust-ctl place add-learner -n 2 -r localhost:1229
Placement center add leaner successfully
```

## 3. 更改成员关系

更改Placement集群的成员关系。

```
$ bin/robust-ctl place change-membership -m 2 -r
Placement center change membership successfully
```
