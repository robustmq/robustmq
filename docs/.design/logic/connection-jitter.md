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

获取当前的时间作为最近请求时间`current_request_time`
获取当前的请求次数`current_counter`

从Map中获取对应`client-id`的结构信息`ClientConnectInfo`，如果没有的情况下
我们需要自己构建`ClientConnectInfo`, 并将`ClientConnectInfo`放入map中

如果`current_request_time` - `request_time` < `windows_time` 并且
`counter` - `connect_count` > `max_count`：
 将对应的client_id放入到黑名单当中，并指定封禁时间
> 这里的两个校验分别是：是否在窗口时间内以及是否超过窗口时间内的最大连接次数


# 定时清理map
## 目的
设定一个线程定时对map进行清理，防止map过大

## 核心逻辑

线程会在窗口时间触发后执行，主要的执行逻辑如下：
清理时间和首次请求时间已经大于当前的时间窗口了，此时就可以进行清理操作
