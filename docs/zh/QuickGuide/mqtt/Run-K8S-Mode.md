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
   
查看文档执行测试：[MQTT 功能测试](./MQTT-test.md)