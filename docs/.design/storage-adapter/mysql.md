# schema design

We create two kinds of SQL tables. 

The first table stores all records in a shard under a namespace. (i.e., we create a new table when we create a new shard under a namespace)

Schema:

```sql
CREATE TABLE `record_{namespace}_{shard}` (
`id` int(11) unsigned NOT NULL AUTO_INCREMENT, -- message offset
`msg_key` String DEFAULT NULL,
`payload` blob,
`create_time` int(11) NOT NULL,
PRIMARY KEY (`id`)
unique index msg_key(msg_key) -- create an index on message key for fast lookup (unique)
) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;
```

The second table stores all other information in the form of KV pairs (e.g., shard offset, tag offsets, etc.)

Schema:

```sql
CREATE TABLE `storage_kv` (
`id` int(11) unsigned NOT NULL AUTO_INCREMENT,
`data_key` varchar(128) DEFAULT NULL,
`data_value` blob,
`create_time` int(11) NOT NULL,
`update_time` int(11) NOT NULL,
PRIMARY KEY (`id`),
unique index data_key(data_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;
```

value types (after deserialize):

- shard offset: `u64`
- tag offsets: `Vec<u64>`