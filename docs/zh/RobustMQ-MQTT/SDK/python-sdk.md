# 使用 Python SDK 连接 RobustMQ

## 概述

Eclipse Paho Python 是 Eclipse Paho 项目下的 Python 语言版客户端库，该库能够连接到 RobustMQ MQTT Broker 以发布消息，订阅主题并接收已发布的消息。

该客户端库简单易用，功能完整，是 Python 生态中最受欢迎的 MQTT 客户端库之一。

## 安装依赖

### 使用 pip 安装

```bash
pip install paho-mqtt
```

### 使用 conda 安装

```bash
conda install paho-mqtt
```

### 在 requirements.txt 中添加

```txt
paho-mqtt==1.6.1
```

## 基础连接示例

### 发布和订阅示例

```python
import paho.mqtt.client as mqtt
import time
import json

# 连接成功回调
def on_connect(client, userdata, flags, rc):
    print(f'连接结果代码: {rc}')
    if rc == 0:
        print('成功连接到 RobustMQ')
        # 连接成功后订阅主题
        client.subscribe('robustmq/python/test/#')
        print('已订阅主题: robustmq/python/test/#')
    else:
        print(f'连接失败，返回代码: {rc}')

# 消息接收回调
def on_message(client, userdata, msg):
    print('=== 收到消息 ===')
    print(f'主题: {msg.topic}')
    print(f'消息: {msg.payload.decode()}')
    print(f'QoS: {msg.qos}')
    print(f'保留消息: {msg.retain}')
    print('================')

# 消息发布回调
def on_publish(client, userdata, mid):
    print(f'消息发布成功，消息ID: {mid}')

# 订阅成功回调
def on_subscribe(client, userdata, mid, granted_qos):
    print(f'订阅成功，消息ID: {mid}, 授权QoS: {granted_qos}')

# 断开连接回调
def on_disconnect(client, userdata, rc):
    if rc != 0:
        print(f'意外断开连接，返回代码: {rc}')
    else:
        print('正常断开连接')

def main():
    # 创建客户端实例
    client = mqtt.Client(client_id='robustmq_python_client')
    
    # 设置用户名和密码
    client.username_pw_set('your_username', 'your_password')
    
    # 设置回调函数
    client.on_connect = on_connect
    client.on_message = on_message
    client.on_publish = on_publish
    client.on_subscribe = on_subscribe
    client.on_disconnect = on_disconnect
    
    # 连接到 RobustMQ Broker
    try:
        print('正在连接 RobustMQ...')
        client.connect('localhost', 1883, 60)
        
        # 启动网络循环
        client.loop_start()
        
        # 等待连接建立
        time.sleep(2)
        
        # 发布测试消息
        test_messages = [
            {'topic': 'robustmq/python/test/hello', 'payload': 'Hello RobustMQ!'},
            {'topic': 'robustmq/python/test/data', 'payload': json.dumps({'sensor': 'temp', 'value': 25.5})},
            {'topic': 'robustmq/python/test/status', 'payload': 'online'}
        ]
        
        for msg in test_messages:
            result = client.publish(msg['topic'], msg['payload'], qos=1)
            if result.rc == mqtt.MQTT_ERR_SUCCESS:
                print(f"消息已发布到: {msg['topic']}")
            else:
                print(f"消息发布失败: {result.rc}")
            time.sleep(1)
        
        # 保持连接一段时间以接收消息
        print('等待接收消息...')
        time.sleep(5)
        
    except Exception as e:
        print(f'连接错误: {e}')
    finally:
        # 停止网络循环并断开连接
        client.loop_stop()
        client.disconnect()
        print('已断开与 RobustMQ 的连接')

if __name__ == '__main__':
    main()
```

## 高级功能

### SSL/TLS 连接

```python
import paho.mqtt.client as mqtt
import ssl

def on_connect(client, userdata, flags, rc):
    print(f'SSL连接结果: {rc}')
    if rc == 0:
        print('成功建立SSL连接到 RobustMQ')

def main():
    client = mqtt.Client(client_id='robustmq_python_ssl_client')
    
    # 配置 SSL/TLS
    context = ssl.create_default_context(ssl.Purpose.SERVER_AUTH)
    context.check_hostname = False
    context.verify_mode = ssl.CERT_NONE  # 测试环境，生产环境请使用 CERT_REQUIRED
    
    client.tls_set_context(context)
    
    # 设置回调
    client.on_connect = on_connect
    
    # 连接到 RobustMQ SSL 端口
    try:
        client.connect('localhost', 1885, 60)  # SSL 端口
        client.loop_start()
        
        # 发布 SSL 测试消息
        client.publish('robustmq/ssl/test', 'SSL connection test', qos=1)
        
        time.sleep(3)
        
    except Exception as e:
        print(f'SSL连接错误: {e}')
    finally:
        client.loop_stop()
        client.disconnect()

if __name__ == '__main__':
    main()
```

### WebSocket 连接

```python
import paho.mqtt.client as mqtt

def on_connect(client, userdata, flags, rc):
    print(f'WebSocket连接结果: {rc}')
    if rc == 0:
        print('成功建立WebSocket连接到 RobustMQ')

def main():
    client = mqtt.Client(client_id='robustmq_python_ws_client', transport="websockets")
    
    # 设置回调
    client.on_connect = on_connect
    
    # 连接到 RobustMQ WebSocket 端口
    try:
        client.connect('localhost', 8083, 60)  # WebSocket 端口
        client.loop_start()
        
        # 发布 WebSocket 测试消息
        client.publish('robustmq/websocket/test', 'WebSocket connection test', qos=1)
        
        time.sleep(3)
        
    except Exception as e:
        print(f'WebSocket连接错误: {e}')
    finally:
        client.loop_stop()
        client.disconnect()

if __name__ == '__main__':
    main()
```

### 异步客户端示例

```python
import asyncio
import paho.mqtt.client as mqtt
from concurrent.futures import ThreadPoolExecutor
import json
import time

class AsyncMQTTClient:
    def __init__(self, broker, port, client_id):
        self.broker = broker
        self.port = port
        self.client_id = client_id
        self.client = None
        self.executor = ThreadPoolExecutor(max_workers=10)
        
    async def connect(self):
        """异步连接到 RobustMQ"""
        loop = asyncio.get_event_loop()
        
        def _connect():
            self.client = mqtt.Client(client_id=self.client_id)
            self.client.on_connect = self._on_connect
            self.client.on_message = self._on_message
            self.client.on_disconnect = self._on_disconnect
            
            self.client.connect(self.broker, self.port, 60)
            self.client.loop_start()
            return True
            
        await loop.run_in_executor(self.executor, _connect)
        
    def _on_connect(self, client, userdata, flags, rc):
        if rc == 0:
            print('异步连接到 RobustMQ 成功')
        else:
            print(f'异步连接失败: {rc}')
            
    def _on_message(self, client, userdata, msg):
        print(f'异步接收消息 - 主题: {msg.topic}, 内容: {msg.payload.decode()}')
        
    def _on_disconnect(self, client, userdata, rc):
        print(f'异步断开连接: {rc}')
    
    async def publish(self, topic, payload, qos=1):
        """异步发布消息"""
        loop = asyncio.get_event_loop()
        
        def _publish():
            result = self.client.publish(topic, payload, qos)
            return result.rc == mqtt.MQTT_ERR_SUCCESS
            
        return await loop.run_in_executor(self.executor, _publish)
    
    async def subscribe(self, topic, qos=1):
        """异步订阅主题"""
        loop = asyncio.get_event_loop()
        
        def _subscribe():
            result, mid = self.client.subscribe(topic, qos)
            return result == mqtt.MQTT_ERR_SUCCESS
            
        return await loop.run_in_executor(self.executor, _subscribe)
    
    async def disconnect(self):
        """异步断开连接"""
        if self.client:
            self.client.loop_stop()
            self.client.disconnect()
        self.executor.shutdown(wait=True)

async def main():
    # 创建异步客户端
    client = AsyncMQTTClient('localhost', 1883, 'robustmq_async_python_client')
    
    try:
        # 异步连接
        await client.connect()
        await asyncio.sleep(1)  # 等待连接建立
        
        # 异步订阅
        await client.subscribe('robustmq/async/python/test')
        
        # 异步发布多条消息
        messages = [
            'Hello from async Python client!',
            json.dumps({'temperature': 23.5, 'humidity': 65}),
            'Async message processing test'
        ]
        
        for i, msg in enumerate(messages):
            topic = f'robustmq/async/python/test/{i}'
            success = await client.publish(topic, msg)
            if success:
                print(f'异步发布消息成功: {topic}')
            await asyncio.sleep(0.5)
        
        # 等待接收消息
        await asyncio.sleep(3)
        
    except Exception as e:
        print(f'异步操作错误: {e}')
    finally:
        await client.disconnect()

if __name__ == '__main__':
    asyncio.run(main())
```

### 多线程处理示例

```python
import paho.mqtt.client as mqtt
import threading
import queue
import json
import time
from datetime import datetime

class MQTTWorker:
    def __init__(self, broker, port, client_id, worker_count=5):
        self.broker = broker
        self.port = port
        self.client_id = client_id
        self.message_queue = queue.Queue()
        self.worker_count = worker_count
        self.workers = []
        self.client = None
        
    def on_connect(self, client, userdata, flags, rc):
        if rc == 0:
            print('多线程客户端连接到 RobustMQ 成功')
            client.subscribe('robustmq/worker/+/task')
        else:
            print(f'连接失败: {rc}')
    
    def on_message(self, client, userdata, msg):
        # 将消息放入队列，由工作线程处理
        message_data = {
            'topic': msg.topic,
            'payload': msg.payload.decode(),
            'qos': msg.qos,
            'timestamp': datetime.now()
        }
        self.message_queue.put(message_data)
        print(f'消息已加入处理队列: {msg.topic}')
    
    def worker_thread(self, worker_id):
        """工作线程函数"""
        print(f'工作线程 {worker_id} 已启动')
        
        while True:
            try:
                # 从队列获取消息
                message = self.message_queue.get(timeout=1)
                
                # 处理消息
                print(f'工作线程 {worker_id} 正在处理消息: {message["topic"]}')
                
                # 模拟消息处理
                if 'data' in message['topic']:
                    # 处理数据消息
                    try:
                        data = json.loads(message['payload'])
                        print(f'处理数据: {data}')
                    except json.JSONDecodeError:
                        print(f'数据格式错误: {message["payload"]}')
                
                elif 'command' in message['topic']:
                    # 处理命令消息
                    print(f'执行命令: {message["payload"]}')
                
                # 处理完成
                self.message_queue.task_done()
                print(f'工作线程 {worker_id} 处理完成')
                
            except queue.Empty:
                continue
            except Exception as e:
                print(f'工作线程 {worker_id} 处理错误: {e}')
    
    def start(self):
        """启动 MQTT 客户端和工作线程"""
        # 创建 MQTT 客户端
        self.client = mqtt.Client(client_id=self.client_id)
        self.client.on_connect = self.on_connect
        self.client.on_message = self.on_message
        
        # 连接到 RobustMQ
        self.client.connect(self.broker, self.port, 60)
        self.client.loop_start()
        
        # 启动工作线程
        for i in range(self.worker_count):
            worker = threading.Thread(target=self.worker_thread, args=(i,), daemon=True)
            worker.start()
            self.workers.append(worker)
        
        print(f'已启动 {self.worker_count} 个工作线程')
        
        # 发布一些测试消息
        time.sleep(2)
        self.publish_test_messages()
    
    def publish_test_messages(self):
        """发布测试消息"""
        test_data = [
            {'topic': 'robustmq/worker/1/data', 'payload': json.dumps({'sensor': 'temperature', 'value': 25.3})},
            {'topic': 'robustmq/worker/2/command', 'payload': 'restart_service'},
            {'topic': 'robustmq/worker/3/data', 'payload': json.dumps({'sensor': 'humidity', 'value': 60.1})},
        ]
        
        for msg in test_data:
            self.client.publish(msg['topic'], msg['payload'], qos=1)
            print(f"发布测试消息: {msg['topic']}")
            time.sleep(0.5)
    
    def stop(self):
        """停止客户端"""
        if self.client:
            self.client.loop_stop()
            self.client.disconnect()
        print('多线程 MQTT 客户端已停止')

# 使用示例
if __name__ == '__main__':
    worker = MQTTWorker('localhost', 1883, 'robustmq_worker_client', worker_count=3)
    
    try:
        worker.start()
        
        # 运行10秒
        time.sleep(10)
        
    except KeyboardInterrupt:
        print('收到中断信号')
    finally:
        worker.stop()
```

## 高级功能

### 配置管理

```python
import paho.mqtt.client as mqtt
import configparser
import os

class MQTTConfig:
    def __init__(self, config_file='mqtt_config.ini'):
        self.config = configparser.ConfigParser()
        self.config_file = config_file
        self.load_config()
    
    def load_config(self):
        """加载配置文件"""
        if os.path.exists(self.config_file):
            self.config.read(self.config_file)
        else:
            self.create_default_config()
    
    def create_default_config(self):
        """创建默认配置"""
        self.config['MQTT'] = {
            'broker': 'localhost',
            'port': '1883',
            'client_id': 'robustmq_python_client',
            'username': '',
            'password': '',
            'keepalive': '60',
            'clean_session': 'True'
        }
        
        self.config['SSL'] = {
            'enabled': 'False',
            'ca_cert_path': '',
            'cert_path': '',
            'key_path': ''
        }
        
        with open(self.config_file, 'w') as f:
            self.config.write(f)
        print(f'已创建默认配置文件: {self.config_file}')
    
    def get_mqtt_options(self):
        """获取 MQTT 连接选项"""
        return {
            'broker': self.config.get('MQTT', 'broker'),
            'port': self.config.getint('MQTT', 'port'),
            'client_id': self.config.get('MQTT', 'client_id'),
            'username': self.config.get('MQTT', 'username'),
            'password': self.config.get('MQTT', 'password'),
            'keepalive': self.config.getint('MQTT', 'keepalive'),
            'clean_session': self.config.getboolean('MQTT', 'clean_session')
        }

class ConfigurableMQTTClient:
    def __init__(self, config_file='mqtt_config.ini'):
        self.config = MQTTConfig(config_file)
        self.client = None
        
    def connect(self):
        """连接到 RobustMQ"""
        opts = self.config.get_mqtt_options()
        
        self.client = mqtt.Client(client_id=opts['client_id'], 
                                 clean_session=opts['clean_session'])
        
        if opts['username']:
            self.client.username_pw_set(opts['username'], opts['password'])
        
        self.client.on_connect = self._on_connect
        self.client.on_message = self._on_message
        
        try:
            self.client.connect(opts['broker'], opts['port'], opts['keepalive'])
            self.client.loop_start()
            return True
        except Exception as e:
            print(f'连接失败: {e}')
            return False
    
    def _on_connect(self, client, userdata, flags, rc):
        print(f'配置化客户端连接结果: {rc}')
    
    def _on_message(self, client, userdata, msg):
        print(f'收到配置化消息: {msg.topic} -> {msg.payload.decode()}')
```

### 数据处理管道

```python
import paho.mqtt.client as mqtt
import json
import sqlite3
import threading
from datetime import datetime

class DataPipeline:
    def __init__(self, broker, port, db_path='mqtt_data.db'):
        self.broker = broker
        self.port = port
        self.db_path = db_path
        self.client = None
        self.db_lock = threading.Lock()
        self.init_database()
        
    def init_database(self):
        """初始化数据库"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS mqtt_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                topic TEXT NOT NULL,
                payload TEXT NOT NULL,
                qos INTEGER,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        ''')
        
        conn.commit()
        conn.close()
        print('数据库初始化完成')
    
    def save_message(self, topic, payload, qos):
        """保存消息到数据库"""
        with self.db_lock:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            cursor.execute('''
                INSERT INTO mqtt_messages (topic, payload, qos)
                VALUES (?, ?, ?)
            ''', (topic, payload, qos))
            
            conn.commit()
            conn.close()
    
    def on_connect(self, client, userdata, flags, rc):
        if rc == 0:
            print('数据管道连接到 RobustMQ 成功')
            # 订阅多个数据主题
            topics = [
                ('robustmq/sensors/+/temperature', 1),
                ('robustmq/sensors/+/humidity', 1),
                ('robustmq/events/+', 2),
                ('robustmq/logs/+', 0)
            ]
            
            for topic, qos in topics:
                client.subscribe(topic, qos)
                print(f'订阅主题: {topic}')
    
    def on_message(self, client, userdata, msg):
        """处理接收到的消息"""
        topic = msg.topic
        payload = msg.payload.decode()
        qos = msg.qos
        
        print(f'处理消息: {topic}')
        
        try:
            # 尝试解析 JSON 数据
            if payload.startswith('{'):
                data = json.loads(payload)
                print(f'解析JSON数据: {data}')
                
                # 根据主题类型进行不同处理
                if 'temperature' in topic:
                    self.process_temperature_data(topic, data)
                elif 'humidity' in topic:
                    self.process_humidity_data(topic, data)
                elif 'events' in topic:
                    self.process_event_data(topic, data)
            
            # 保存原始消息到数据库
            self.save_message(topic, payload, qos)
            
        except json.JSONDecodeError:
            print(f'非JSON消息: {payload}')
            self.save_message(topic, payload, qos)
        except Exception as e:
            print(f'消息处理错误: {e}')
    
    def process_temperature_data(self, topic, data):
        """处理温度数据"""
        if 'value' in data and data['value'] > 30:
            # 高温告警
            alert = {
                'alert_type': 'high_temperature',
                'sensor': topic.split('/')[2],
                'value': data['value'],
                'timestamp': datetime.now().isoformat()
            }
            
            self.client.publish('robustmq/alerts/temperature', 
                              json.dumps(alert), qos=2)
            print(f'发送高温告警: {alert}')
    
    def process_humidity_data(self, topic, data):
        """处理湿度数据"""
        if 'value' in data and data['value'] < 30:
            # 低湿度告警
            alert = {
                'alert_type': 'low_humidity',
                'sensor': topic.split('/')[2],
                'value': data['value'],
                'timestamp': datetime.now().isoformat()
            }
            
            self.client.publish('robustmq/alerts/humidity', 
                              json.dumps(alert), qos=2)
            print(f'发送低湿度告警: {alert}')
    
    def process_event_data(self, topic, data):
        """处理事件数据"""
        print(f'处理事件: {data}')
        
        # 事件转发到日志主题
        log_data = {
            'source': 'mqtt_pipeline',
            'event': data,
            'processed_at': datetime.now().isoformat()
        }
        
        self.client.publish('robustmq/logs/events', 
                          json.dumps(log_data), qos=1)
    
    def start(self):
        """启动数据管道"""
        self.client = mqtt.Client(client_id='robustmq_data_pipeline')
        self.client.on_connect = self.on_connect
        self.client.on_message = self.on_message
        
        try:
            self.client.connect(self.broker, self.port, 60)
            self.client.loop_forever()
        except Exception as e:
            print(f'数据管道启动失败: {e}')
    
    def stop(self):
        """停止数据管道"""
        if self.client:
            self.client.loop_stop()
            self.client.disconnect()
        print('数据管道已停止')

# 使用示例
if __name__ == '__main__':
    pipeline = DataPipeline('localhost', 1883)
    
    try:
        pipeline.start()
    except KeyboardInterrupt:
        print('收到中断信号')
        pipeline.stop()
```

## 连接参数配置

### 基础连接参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `host` | RobustMQ Broker 地址 | localhost |
| `port` | 连接端口 | 1883 |
| `client_id` | 客户端唯一标识 | 自动生成 |
| `keepalive` | 心跳间隔（秒） | 60 |
| `clean_session` | 是否清除会话 | True |

### RobustMQ 支持的协议端口

| 协议 | 端口 | 连接方式 |
|------|------|----------|
| MQTT | 1883 | `client.connect('localhost', 1883)` |
| MQTT over SSL | 1885 | `client.connect('localhost', 1885)` + SSL配置 |
| MQTT over WebSocket | 8083 | `transport="websockets"` |
| MQTT over WSS | 8084 | WebSocket + SSL配置 |

## 最佳实践

### 1. 异常处理和重连

```python
import paho.mqtt.client as mqtt
import time

class RobustMQTTClient:
    def __init__(self, broker, port, client_id):
        self.broker = broker
        self.port = port
        self.client_id = client_id
        self.client = None
        self.max_retries = 5
        
    def connect_with_retry(self):
        """带重试的连接"""
        retry_count = 0
        
        while retry_count < self.max_retries:
            try:
                self.client = mqtt.Client(client_id=self.client_id)
                self.client.on_connect = self.on_connect
                self.client.on_disconnect = self.on_disconnect
                
                self.client.connect(self.broker, self.port, 60)
                self.client.loop_start()
                return True
                
            except Exception as e:
                retry_count += 1
                print(f'连接尝试 {retry_count} 失败: {e}')
                
                if retry_count < self.max_retries:
                    time.sleep(2 ** retry_count)  # 指数退避
                    
        print(f'连接失败，已重试 {self.max_retries} 次')
        return False
    
    def on_connect(self, client, userdata, flags, rc):
        if rc == 0:
            print('连接成功')
        else:
            print(f'连接失败: {rc}')
    
    def on_disconnect(self, client, userdata, rc):
        if rc != 0:
            print(f'意外断开，尝试重连: {rc}')
            self.connect_with_retry()
```

### 2. 消息验证和过滤

```python
import paho.mqtt.client as mqtt
import json
import re
from jsonschema import validate, ValidationError

class MessageValidator:
    def __init__(self):
        # 定义消息模式
        self.schemas = {
            'sensor_data': {
                'type': 'object',
                'properties': {
                    'sensor_id': {'type': 'string'},
                    'value': {'type': 'number'},
                    'unit': {'type': 'string'},
                    'timestamp': {'type': 'string'}
                },
                'required': ['sensor_id', 'value']
            }
        }
    
    def validate_message(self, topic, payload):
        """验证消息格式"""
        try:
            data = json.loads(payload)
            
            if 'sensors' in topic:
                validate(instance=data, schema=self.schemas['sensor_data'])
                return True, data
            
            return True, data
            
        except json.JSONDecodeError:
            return False, f'JSON格式错误: {payload}'
        except ValidationError as e:
            return False, f'消息验证失败: {e.message}'
        except Exception as e:
            return False, f'验证错误: {e}'

def on_message(client, userdata, msg):
    validator = MessageValidator()
    
    # 验证消息
    is_valid, result = validator.validate_message(msg.topic, msg.payload.decode())
    
    if is_valid:
        print(f'有效消息: {msg.topic} -> {result}')
        # 处理有效消息
    else:
        print(f'无效消息: {result}')
        # 记录错误或丢弃消息
```

### 3. 性能监控

```python
import paho.mqtt.client as mqtt
import time
import threading
from collections import defaultdict

class MQTTMetrics:
    def __init__(self):
        self.start_time = time.time()
        self.messages_published = 0
        self.messages_received = 0
        self.publish_errors = 0
        self.topic_stats = defaultdict(int)
        self.lock = threading.Lock()
    
    def increment_published(self):
        with self.lock:
            self.messages_published += 1
    
    def increment_received(self, topic):
        with self.lock:
            self.messages_received += 1
            self.topic_stats[topic] += 1
    
    def increment_errors(self):
        with self.lock:
            self.publish_errors += 1
    
    def print_stats(self):
        with self.lock:
            elapsed = time.time() - self.start_time
            
            print('=== MQTT 性能统计 ===')
            print(f'运行时间: {elapsed:.2f} 秒')
            print(f'已发布消息: {self.messages_published}')
            print(f'已接收消息: {self.messages_received}')
            print(f'发布错误: {self.publish_errors}')
            
            if elapsed > 0:
                print(f'发布速率: {self.messages_published/elapsed:.2f} 消息/秒')
                print(f'接收速率: {self.messages_received/elapsed:.2f} 消息/秒')
            
            print('主题统计:')
            for topic, count in self.topic_stats.items():
                print(f'  {topic}: {count} 条消息')
            print('===================')

# 全局指标实例
metrics = MQTTMetrics()

def on_publish(client, userdata, mid):
    metrics.increment_published()

def on_message(client, userdata, msg):
    metrics.increment_received(msg.topic)
```

## 常见问题

### Q: 如何处理连接断开？

A: 设置断开连接回调和自动重连：

```python
def on_disconnect(client, userdata, rc):
    if rc != 0:
        print(f'意外断开连接: {rc}')
        # 实现重连逻辑

client.on_disconnect = on_disconnect
```

### Q: 如何设置消息质量等级 (QoS)？

A: RobustMQ 支持 MQTT 标准的三种 QoS 等级：

```python
# QoS 0: 最多一次传递
client.publish(topic, payload, qos=0)

# QoS 1: 至少一次传递
client.publish(topic, payload, qos=1)

# QoS 2: 恰好一次传递
client.publish(topic, payload, qos=2)
```

### Q: 如何处理大量消息？

A: 使用多线程和消息队列：

1. 使用 `client.loop_start()` 启用后台线程
2. 实现消息队列缓冲高峰流量
3. 使用工作线程池处理消息

## 运行和部署

### 项目结构

```
robustmq-python-client/
├── main.py
├── config.py
├── mqtt_client.py
├── requirements.txt
├── mqtt_config.ini
└── README.md
```

### 运行示例

```bash
# 安装依赖
pip install -r requirements.txt

# 运行基础示例
python main.py

# 运行异步示例
python async_client.py

# 运行数据管道
python data_pipeline.py
```

## MQTT 5.0 支持

目前 Paho Python 对 MQTT 5.0 的支持还在开发中。对于需要 MQTT 5.0 特性的场景，可以考虑使用以下替代库：

- **paho-mqtt 2.0+**: Paho 项目的新版本（开发中）
- **aiomqtt**: 基于 asyncio 的现代 MQTT 客户端
- **gmqtt**: 支持 MQTT 5.0 的异步客户端

## 总结

Eclipse Paho Python 是 Python 生态中最成熟稳定的 MQTT 客户端库。该库API简洁易用，功能完整，非常适合快速开发 MQTT 应用程序。

通过本文档的示例，您可以快速上手使用 Python 连接 RobustMQ MQTT Broker，并实现从简单的消息收发到复杂的数据处理管道等各种应用场景。结合 Python 丰富的生态系统，可以轻松构建功能强大的 IoT 和消息处理系统。

