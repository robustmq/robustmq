# Queue Purge & Delete

`Queue.Purge` and `Queue.Delete` are easily confused but semantically very different: the former only empties a queue's contents, the latter removes the queue itself. RobustMQ implements real semantics for both, with safety checks.

## Queue.Purge: Empty While Keeping the Queue

`Queue.Purge` doesn't remove the queue's metadata or declaration — it only deletes every message currently piled up in the queue:

- The implementation deletes each partition's records up to that partition's current high watermark — effectively "clear up to the latest position" rather than a logical tombstone.
- The queue's declaration, bindings, and shared consume group are all left untouched — publishing and consuming can resume right after purging.
- `Queue.Purge-Ok` returns the number of purged messages (`message_count`), a real count, not a placeholder.

## Queue.Delete: Removing the Queue Itself

`Queue.Delete` completely removes the queue, which involves:

1. Deleting the queue's metadata (`storage.delete_queue()`).
2. Removing it from the broker's local cache.
3. Explicitly destroying the underlying storage shard (`delete_storage_resource`) — this step matters: without cleaning up the underlying shard, redeclaring a queue with the same name could "resurrect" old messages. RobustMQ explicitly avoids this.

### Safety Checks: if-unused and if-empty

`Queue.Delete` supports two protective flags, both genuinely validated by RobustMQ:

- `if-empty=true`: if the queue still has messages (`message_count > 0`), the delete is rejected with `406 PRECONDITION_FAILED`.
- `if-unused=true`: if the queue still has consumers (determined via the shared consume group's member count), the delete is rejected, also with `406 PRECONDITION_FAILED`.

Both checks run before the delete actually executes, preventing an in-use queue from being accidentally removed.

## Example (Java Client)

```java
// Empty while keeping the queue
AMQP.Queue.PurgeOk purgeOk = channel.queuePurge("orders-queue");
System.out.println("purged: " + purgeOk.getMessageCount());

// Only deletes if the queue is empty and has no consumers
channel.queueDelete("orders-queue", true, true);
```

## Further Reading

- [Storage](./Storage.md)
- [Core Concepts](./AMQPCoreConcepts.md)
- [Shared Queue Group](./SharedQueueGroup.md)
