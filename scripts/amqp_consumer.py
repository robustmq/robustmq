# Copyright 2023 RobustMQ Team
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import pika
import sys

# Configuration
HOST = '127.0.0.1'
PORT = 5672
VHOST = '/'
USERNAME = 'admin'
PASSWORD = 'robustmq'
QUEUE = 'robustmq.multi.protocol'


def on_message(ch, method, properties, body):
    print(f"[Message received] {body.decode()}")
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
        print(f"Connected to {HOST}:{PORT}, listening on queue: {QUEUE}")
        print("Waiting for messages, press Ctrl+C to exit...\n")

        ch.basic_consume(
            queue=QUEUE,
            on_message_callback=on_message,
            auto_ack=False
        )
        ch.start_consuming()

    except pika.exceptions.AMQPConnectionError as e:
        print(f"Connection failed: {e}")
        sys.exit(1)
    except KeyboardInterrupt:
        print("\nExiting")
        conn.close()


if __name__ == '__main__':
    main()