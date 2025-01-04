1. Prerequisites :

- [kubectl](https://kubernetes.io/docs/tasks/tools/)
- [jq](https://stedolan.github.io/jq/)
- [envsubst](https://www.gnu.org/software/gettext/manual/html_node/envsubst-Invocation.html) (`brew install gettext` on OSX)

- K8s :
  - [KIND](https://kind.sigs.k8s.io/docs/user/quick-start/#installation) + [Docker](https://www.docker.com) ( 8 CPU / 8 GRAM)


2. Quick start


    Build images, you can open the `network.sh` file and modify `_IMAGE_NAME` and `IMAGE_VERSION` to change the image tag

    ```shell

        cd example/test-network-k8s
        ./network.sh build-images

    ```

    Pull some necessary images

    ```shell
        docker pull k8s.gcr.io/ingress-nginx/controller:v1.1.2
        docker pull quay.io/jetstack/cert-manager-webhook:v1.6.1
        docker pull quay.io/jetstack/cert-manager-controller:v1.6.1
        docker pull quay.io/jetstack/cert-manager-cainjector:v1.6.1
        docker pull k8s.gcr.io/ingress-nginx/kube-webhook-certgen:v1.1.1
    ```

    Initialize kind cluster

    ```shell
        ./network.sh kind
    ```

    Load the image to the kind cluster

    ```shell
        ./network.sh kind-load-images
    ```

    Start kind k8s cluster

    ```shell
        ./network.sh cluster-init
    ```

    Start the RobustMQ cluster

    ```shell
        ./network.sh up
    ```

3. cleanup

    ```shell
        ./network.sh unkind
    ```
