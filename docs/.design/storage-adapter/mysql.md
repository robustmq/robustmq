# schema design

We will create three kinds of SQL tables. 

The first kind of tables will store all records in a shard under a namespace. (i.e., we create a new table when we create a new shard under a namespace)

Schema:

```sql
CREATE TABLE `record_{namespace}_{shard}` (
  `offset` int(11) unsigned NOT NULL AUTO_INCREMENT,
  `key` varchar(255) DEFAULT NULL, -- May need to adjust length based on key length constraint
  `data` blob,
  `header` blob,
  `tags` blob,
  `ts` timestamp NOT NULL,
  PRIMARY KEY (`offset`),
  UNIQUE INDEX `key_idx` (`key`),
  INDEX `ts_idx` (`ts`),
  INDEX `ts_offset_idx` (`ts`, `offset`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;
```

The second kind of tables will store tag information for records in a shard under a namespace. This table will be created along with the first kind.

Schema:

```sql
CREATE TABLE `tag_{namespace}_{shard}` (
  `m_offset` int(11) unsigned NOT NULL,
  `tag` varchar(255) NOT NULL,
  PRIMARY KEY (`m_offset`, `tag`),
  INDEX `tag_offset_idx` (`tag`, `m_offset`),
  CONSTRAINT `fk_tag_offset`
    FOREIGN KEY (`m_offset`)
    REFERENCES `record_{namespace}_{shard}` (`m_offset`)
    ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;

```

The third kind of tables will store group information.

Schema:

```sql
CREATE TABLE `group_{group}` {
    `namespace` varchar(255) NOT NULL,
    `shard` varchar(255) NOT NULL,
    `offset` int(11) unsigned NOT NULL
}
```

# Queries for each operation defined in `StorageAdapter`:

## create_shard

First check whether the table exists:

```sql
SHOW TABLES LIKE `{record}_{namespace}_{shard}`

SHOW TABLES LIKE `tag_{namespace}_{shard}`
```

Given the namespace and shard, execute the following sql queries:

```sql
CREATE TABLE `record_{namespace}_{shard}` (
  `offset` int(11) unsigned NOT NULL AUTO_INCREMENT,
  `key` varchar(255) DEFAULT NULL, -- May need to adjust length based on key length constraint
  `data` blob,
  `header` blob,
  `tags` blob,
  `ts` timestamp NOT NULL,
  PRIMARY KEY (`offset`),
  UNIQUE INDEX `key_idx` (`key`),
  INDEX `ts_idx` (`ts`),
  INDEX `ts_offset_idx` (`ts`, `offset`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;

CREATE TABLE `tag_{namespace}_{shard}` (
  `m_offset` int(11) unsigned NOT NULL,
  `tag` varchar(255) NOT NULL,
  PRIMARY KEY (`m_offset`, `tag`),
  INDEX `tag_offset_idx` (`tag`, `m_offset`),
  CONSTRAINT `fk_tag_offset`
    FOREIGN KEY (`m_offset`)
    REFERENCES `record_{namespace}_{shard}` (`m_offset`)
    ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;
```

## delete_shard

We just need to drop the two tabled created by `create_shard`

## write

Just as what we did in rocksdb, we will spawn a writer thread which will handle all `write` and `batch_write` operations. 

To insert a record, we execute:

```sql
REPLACE INTO `record_{namespace}_{shard}` (m_key,m_data,m_header,m_tags,m_ts) VALUES (:key,:data,:header,:tags,:ts)
```

The values of `key`, `data`, `header`, `tags` and `ts` are passed in through the application code.

To insert all tags for a record, we execute the following for each tag in `message.tags` (by calling `conn.exec_batch` in rust):

```sql
REPLACE INTO `tag_{namespace}_{shard}` (m_offset,tag) VALUES (:offset,:tag)
```

## batch_write

Similar to the `write` operation. 

We use `exec_batch` to insert a batch of records.

We loop over a batch of records and call `exec_batch` to insert all tags for all records.

## read_by_offset

Given `namespace`, `shard`, starting offset `offset` and the maximal number of records to fetch, we execute the following sql query:

```sql
SELECT (offset,key,data,header,tags,ts) 
FROM `record_{namespace}_{shard}`
WHERE offset > :offset
ORDER BY offset
LIMIT read_config.max_record_num
```

The above query will be efficient since the `offset` column is the primary key of `record_{namespace}_{shard}`, meaning its values are sorted and indexed.

## read_by_tag

First get the list of offsets:
```sql
SELECT (r.offset,r.key,r.data,r.header,r.tags,r.ts)
FROM 
    `tag_{namespace}_{shard}` l LEFT JOIN `record_{namespace}_{shard}` r on l.m_offset = r.offset
WHERE l.tag = :tag and l.m_offset > :offset
ORDER BY l.m_offset
LIMIT read_config.max_record_num
```

The above query will be efficient because:

- The join condition `l.m_offset = r.offset` is efficient since `l.m_offset` is part of the PRIMARY KEY in the left table and `r.offset` is the PRIMARY KEY in the right table
- The composite index `tag_offset_idx` is used for filtering the tag and range condition on m_offset.

## read_by_key

```sql
SELECT (offset,key,data,header,tags,ts) 
FROM `record_{namespace}_{shard}`
WHERE key = :key and offset > :offset
ORDER BY offset
LIMIT read_config.max_record_num
```

The above query will be efficient since we created an unique inde on the `key` column, meaning at most one row will be returned.

## get_offset_by_timestamp

```sql
SELECT offset
FROM `record_{namespace}_{shard}`
WHERE ts >= :ts
ORDER BY ts
LIMIT 1
```

The above query will be efficient since we created an index on the `ts` column and a composite index on (`ts` and `offset`), meaning we can perform range lookup 
on the `ts` column and get the `offset` value without additional lookup.

## get_offset_by_group

```sql
SELECT * 
FROM `group_{group}`
```

## commit_offset

```sql
REPLACE INTO `group_{group}` (namespace,shard,offset) VALUES (:namespace, :shard, :offset)
```

## close

Just stop the writer thread