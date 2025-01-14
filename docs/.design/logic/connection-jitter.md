# 不涉及删除结构体
构建一个结构体
create a struct
```rust
struct ClientConnectInfo {
    client_id: u64,
    connect_count: u8,
    request_time: u64,
}
```

创建或获取当前的counter

counter值增加1

获取当前时间`now_time`作为`current_request_time`

从Map中获取对应`client-id`的结构信息`ClientConnectInfo`

如果`current_request_time` - `request_time` < `windows_time` - 在窗口时间内
- 如果`counter` - `connect_count` > `max_count`：
 将对应的client放入到黑名单当中，并指定封禁时间， 并删除存放在map中的内容

否则不在窗口时间内，此时更新request_time和connect_count

重新放入map中

# 定时清理map
## 目的
设定一个线程定时对map进行清理，防止map过大

## 核心逻辑

获取now_time

如果now_time - request_time > windows_time, 删除map中的元素
