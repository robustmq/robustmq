1. 先决条件:

- [kubectl](https://kubernetes.io/docs/tasks/tools/)
- [jq](https://stedolan.github.io/jq/)
- [envsubst](https://www.gnu.org/software/gettext/manual/html_node/envsubst-Invocation.html) (`brew install gettext` on OSX)

- K8s :
  - [KIND](https://kind.sigs.k8s.io/docs/user/quick-start/#installation) + [Docker](https://www.docker.com) (资源: 8 CPU / 8 GRAM)


2. 快速开始


    build镜像,可以打开 `network.sh` 文件，修改 `_IMAGE_NAME` 和 `IMAGE_VERSION` 来更改镜tag

    ```shell
     cd example/test-network-k8s
     ./network.sh build-images
    ```

    拉取一些必要的镜像

    ```shell
     docker pull k8s.gcr.io/ingress-nginx/controller:v1.1.2
     docker pull quay.io/jetstack/cert-manager-webhook:v1.6.1
     docker pull quay.io/jetstack/cert-manager-controller:v1.6.1
     docker pull quay.io/jetstack/cert-manager-cainjector:v1.6.1
     docker pull k8s.gcr.io/ingress-nginx/kube-webhook-certgen:v1.1.1
    ```

    初始化kind集群

    ```shell
     ./network.sh kind
    ```

    加载镜像到kind集群

    ```shell
     ./network.sh kind-load-images
    ```

    启动 kind k8s 集群

    ```shell
     ./network.sh cluster-init
    ```

    启动 RobustMQ 集群

    ```shell
     ./network.sh up
    ```

3. 清理工作

    ```shell
     ./network.sh unkind
    ```

4. 回顾历程

   - 安装 kind
   - 拉取一些必要的镜像
   - 初始化kind集群
   - 加载镜像到kind集群
   - 启动 kind k8s 集群
   - 启动 RobustMQ 集群

5. 验证 MQTT 功能是否正常


此 KIND-Kubernetes 集群将启动一个 `cli-command` pod，您可以在pod内执行 cli-command 命令。

首先，获取 `cli-command` pod 的名称

```console
% kubectl get pods -n robustmq
NAME                                READY   STATUS    RESTARTS   AGE
cli-command-6fbf8b6cc-7ldqp          1/1     Running   0          16m
```
然后，进入 `cli-command` pod 并执行命令

发布消息

 ```console
 % kubectl exec -it cli-command-6fbf8b6cc-7ldqp -n robustmq -- /bin/sh
 # ./cli-command mqtt --server=mqtt-server-cs.robustmq.svc.cluster.local:1883 publish --username=admin --password=pwd123 --topic=test/topic1 --qos=0
 able to connect: "mqtt-server-cs.robustmq.svc.cluster.local:1883"
 you can post a message on the terminal:
 1
 > You typed: 1
 2
 > You typed: 2
 3
 > You typed: 3
 ^C>  Ctrl+C detected,  Please press ENTER to end the program.
 ```

订阅消息

 ```console
 kubectl exec -it cli-command-6fbf8b6cc-7ldqp -n robustmq -- /bin/sh
 # cd libs
 # ./cli-command mqtt --server=mqtt-server-cs.robustmq.svc.cluster.local:1883  subscribe --username=admin --password=pwd123 --topic=test/topic1 --qos=0
 able to connect: "mqtt-server-cs.robustmq.svc.cluster.local:1883"
 subscribe success
 payload: 1
 payload: 2
 payload: 3
 ^C Ctrl+C detected,  Please press ENTER to end the program.
 End of input stream.
 ```
有关更多命令，请参考 [cli-command](../../RobustMQ-Command/Mqtt-Broker.md)

查看文档执行测试：[MQTT 功能测试](./MQTT-test.md)
