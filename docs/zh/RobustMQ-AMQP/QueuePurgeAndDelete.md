# 队列清空与删除

`Queue.Purge` 和 `Queue.Delete` 是两个容易混淆但语义完全不同的操作:前者只清空队列内容,后者连队列本身一起删除。RobustMQ 对两者都实现了真实语义,并带有安全检查。

## Queue.Purge:清空但保留队列

`Queue.Purge` 不会删除队列的元数据或声明,只删除队列里当前堆积的所有消息:

- 实现方式是把每个分区的消息删到该分区当前的 high watermark 为止,相当于"清空到最新位置",而不是逻辑上标记删除。
- 队列的声明、绑定关系、共享消费组都保持不变,清空之后可以继续发布和消费。
- `Queue.Purge-Ok` 会带回被清空的消息数量(`message_count`),这个值是真实统计出来的,不是占位符。

## Queue.Delete:删除队列本身

`Queue.Delete` 会彻底删除队列,包括:

1. 删除队列元数据(`storage.delete_queue()`)。
2. 从 Broker 本地缓存中移除。
3. 显式销毁底层存储分片(`delete_storage_resource`)——这一步很关键:如果不清理底层分片,同名队列被重新声明时可能会"复活"旧消息,RobustMQ 显式避免了这个问题。

### 安全检查:if-unused 与 if-empty

`Queue.Delete` 支持两个保护性参数,RobustMQ 会真实校验:

- `if-empty=true`:如果队列当前还有消息(`message_count > 0`),删除会被拒绝,返回 `406 PRECONDITION_FAILED`。
- `if-unused=true`:如果队列当前还有消费者(通过共享消费组成员数判断),删除会被拒绝,同样返回 `406 PRECONDITION_FAILED`。

两个检查都在真正执行删除之前完成,避免误删还在使用中的队列。

## 示例(Java 客户端)

```java
// 清空但保留队列
AMQP.Queue.PurgeOk purgeOk = channel.queuePurge("orders-queue");
System.out.println("purged: " + purgeOk.getMessageCount());

// 只有队列为空且无消费者时才删除
channel.queueDelete("orders-queue", true, true);
```

## 延伸阅读

- [存储](./Storage.md)
- [核心概念](./AMQPCoreConcepts.md)
- [共享队列组](./SharedQueueGroup.md)
