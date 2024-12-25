## Prerequisites:

- [kubectl](https://kubernetes.io/docs/tasks/tools/)
- [jq](https://stedolan.github.io/jq/)
- [envsubst](https://www.gnu.org/software/gettext/manual/html_node/envsubst-Invocation.html) (`brew install gettext` on OSX)

- K8s - either:
  - [KIND](https://kind.sigs.k8s.io/docs/user/quick-start/#installation) + [Docker](https://www.docker.com) (resources: 8 CPU / 8 GRAM)
  - [Rancher Desktop](https://rancherdesktop.io) (resources: 8 CPU / 8GRAM, mobyd, and disable Traefik)

## Quickstart

步骤回顾
	1.	搭建 Kind 集群：
	•	创建并配置 Kind 集群（如端口映射、本地镜像仓库等）。
	2.	部署 Ingress-nginx:
	•	提供 HTTP 和 HTTPS 的流量路由功能，作为负载均衡器和反向代理。
	3.	部署 cert-manager:
	•	自动管理 TLS 证书，配合 Ingress 实现 HTTPS 支持。


检测k8s集群是否就绪
kubectl get pod -n ingress-nginx
kubectl get pod -n cert-manager


删除 pod
kubectl delete pod ingress-nginx-admission-create-87l75 -n ingress-nginx

查看状态
kubectl describe pod ingress-nginx-admission-create-87l75 -n ingress-nginx
kubectl describe pod ingress-nginx-admission-patch-87l75  -n ingress-nginx


docker pull k8s.gcr.io/ingress-nginx/controller:v1.1.2
docker pull k8s.gcr.io/ingress-nginx/kube-webhook-certgen:v1.1.1

docker pull quay.io/jetstack/cert-manager-cainjector:v1.6.1
docker pull quay.io/jetstack/cert-manager-controller:v1.6.1
docker pull quay.io/jetstack/cert-manager-webhook:v1.6.1

kind load docker-image k8s.gcr.io/ingress-nginx/kube-webhook-certgen:v1.1.1 --name robustmq-kind
kind load docker-image k8s.gcr.io/ingress-nginx/controller:v1.1.2 --name robustmq-kind

kind load docker-image quay.io/jetstack/cert-manager-cainjector:v1.6.1 --name robustmq-kind
kind load docker-image quay.io/jetstack/cert-manager-controller:v1.6.1 --name robustmq-kind
kind load docker-image quay.io/jetstack/cert-manager-webhook:v1.6.1 --name robustmq-kind
