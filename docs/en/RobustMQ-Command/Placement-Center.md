# Placement Center Command

## 1. Placement status

See the Placement cluster for operational information.

```
$ bin/robust-ctl place status
{"running_state":{"Ok":null},"id":1,"current_term":1,"vote":{"leader_id":{"term":1,"node_id":1},"committed":true},"last_log_index":28,"last_applied":{"leader_id":{"term":1,"node_id":1},"index":28},"snapshot":null,"purged":null,"state":"Leader","current_leader":1,"millis_since_quorum_ack":0,"last_quorum_acked":1742005289409447084,"membership_config":{"log_id":{"leader_id":{"term":0,"node_id":0},"index":0},"membership":{"configs":[[1]],"nodes":{"1":{"node_id":1,"rpc_addr":"127.0.0.1:1228"}}}},"heartbeat":{"1":1742005289032346459},"replication":{"1":{"leader_id":{"term":1,"node_id":1},"index":28}}}
```

## 2. Add learner

Add a new node as a learner for Placement cluster expansion.

```
$ bin/robust-ctl place add-learner -n 2 -r 127.0.0.1:1229
Placement center add leaner successfully
```

## 3. Change membership

Change the Placement cluster's membership.

```
$ bin/robust-ctl place change-membership -m 2 -r
Placement center change membership successfully
```