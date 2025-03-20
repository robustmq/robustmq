1. Prerequisites:

- [kubectl](https://kubernetes.io/docs/tasks/tools/)
- [jq](https://stedolan.github.io/jq/)
- [envsubst](https://www.gnu.org/software/gettext/manual/html_node/envsubst-Invocation.html) (`brew install gettext` on OSX)

- K8s :
  - [KIND](https://kind.sigs.k8s.io/docs/user/quick-start/#installation) + [Docker](https://www.docker.com) (Resources: 8 CPU / 8 GB RAM)


2. Quick Start

    To build images, you can open the `network.sh` file and modify `_IMAGE_NAME` and `IMAGE_VERSION` to change the image tags.

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

    Initialize the KIND cluster

    ```shell
        ./network.sh kind
    ```

    Load images into the KIND cluster

    ```shell
        ./network.sh kind-load-images
    ```

    Launch the KIND Kubernetes cluster

    ```shell
        ./network.sh cluster-init
    ```

    Start the RobustMQ cluster

    ```shell
        ./network.sh up
    ```

3. Cleanup

    ```shell
        ./network.sh unkind
    ```

4. Review the Process

   - Install KIND
   - Pull some necessary images
   - Initialize the KIND cluster
   - Load images into the KIND cluster
   - Launch the KIND Kubernetes cluster
   - Start the RobustMQ cluster


5. Enter docket

    For more commands, please refer to [cli-command](../../RobustMQ-Command/Mqtt-Broker.md)

    

    Publish messages
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

    Subscribe to messages
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
    
6. Verify that MQTT functions correctly

This KIND-Kubernetes cluster will start a `cli-command` pod, from which you can execute cli-command commands within the cluster.

First, get the name of the `cli-command` pod

    ```console
    % kubectl get pods -n robustmq
    NAME                                READY   STATUS    RESTARTS   AGE
    cli-command-6fbf8b6cc-7ldqp          1/1     Running   0          16m
    ```
Enter the `cli-command` pod and run commands

Check the documentation to run the test：[MQTT functional tests](./MQTT-test.md)