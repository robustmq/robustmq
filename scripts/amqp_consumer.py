import pika
import sys

# 配置
HOST = '127.0.0.1'
PORT = 5672
VHOST = '/'
USERNAME = 'admin'
PASSWORD = 'robustmq'
QUEUE = '/robustmq/multi/protocol'


def on_message(ch, method, properties, body):
    print(f"[收到消息] {body.decode()}")
    ch.basic_ack(delivery_tag=method.delivery_tag)


def main():
    try:
        conn = pika.BlockingConnection(pika.ConnectionParameters(
            host=HOST,
            port=PORT,
            virtual_host=VHOST,
            credentials=pika.PlainCredentials(USERNAME, PASSWORD)
        ))
        ch = conn.channel()
        print(f"已连接 {HOST}:{PORT}，监听队列: {QUEUE}")
        print("等待消息，按 Ctrl+C 退出...\n")

        ch.basic_consume(
            queue=QUEUE,
            on_message_callback=on_message,
            auto_ack=False
        )
        ch.start_consuming()

    except pika.exceptions.AMQPConnectionError as e:
        print(f"连接失败: {e}")
        sys.exit(1)
    except KeyboardInterrupt:
        print("\n已退出")
        conn.close()


if __name__ == '__main__':
    main()