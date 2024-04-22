CREATE DATABASE test;
USE test;

CREATE TABLE `t_mqtt_user` (
`id` int(11) unsigned NOT NULL AUTO_INCREMENT,
`username` varchar(100) DEFAULT NULL,
`password` varchar(100) DEFAULT NULL,
`salt` varchar(35) DEFAULT NULL,
`is_superuser` tinyint(1) DEFAULT 0,
`created` datetime DEFAULT NULL,
PRIMARY KEY (`id`),
UNIQUE KEY `mqtt_username` (`username`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `t_mqtt_acl` (
`id` int(11) unsigned NOT NULL AUTO_INCREMENT,
`allow` int(1) DEFAULT 1 COMMENT '0: deny, 1: allow',
`ipaddr` varchar(60) DEFAULT NULL COMMENT 'IpAddress',
`username` varchar(100) DEFAULT NULL COMMENT 'Username',
`clientid` varchar(100) DEFAULT NULL COMMENT 'ClientId',
`access` int(2) NOT NULL COMMENT '1: subscribe, 2: publish, 3: pubsub',
`topic` varchar(100) NOT NULL DEFAULT '' COMMENT 'Topic Filter',
PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `storage_stream_record` (
`id` int(11) unsigned NOT NULL AUTO_INCREMENT,
`msgid` varchar(64) DEFAULT NULL,
`payload` blob,
`create_time` int(11) NOT NULL,
PRIMARY KEY (`id`),
INDEX topic_index(`id`, `topic`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;

CREATE TABLE `storage_kv` (
`id` int(11) unsigned NOT NULL AUTO_INCREMENT,
`data_key` varchar(64) DEFAULT NULL,
`data_value` blob,
`create_time` int(11) NOT NULL,
PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;
 

