CREATE DATABASE test;
USE test;

CREATE TABLE `storage_stream_record` (
`id` int(11) unsigned NOT NULL AUTO_INCREMENT,
`msgid` varchar(64) DEFAULT NULL,
`header` String DEFAULT NULL,
`msg_key` String DEFAULT NULL,
`payload` blob,
`create_time` int(11) NOT NULL,
PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;

CREATE TABLE `storage_kv` (
`id` int(11) unsigned NOT NULL AUTO_INCREMENT,
`data_key` varchar(128) DEFAULT NULL,
`data_value` blob,
`create_time` int(11) NOT NULL,
`update_time` int(11) NOT NULL,
PRIMARY KEY (`id`),
unique index data_key(data_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;
 

