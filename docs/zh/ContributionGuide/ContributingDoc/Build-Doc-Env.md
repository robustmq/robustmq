# 如何构建文档环境

RobustMQ使用的是[VitePress](https://vitepress.dev/)来构建对应的文档系统，
如果需要修改配置，可以参考[VitPress文档](https://vitepress.dev/guide/getting-started)
来帮助RobustMQ的文档构建的更好。

## Mac
### 前置准备

通过如下命令安装`node`
```shell
brew install node
```
### 具体步骤

1. 通过如下命令安装`VitePress`所需要的包

```shell
npm install 
```

2. 通过如下命令开启本地调试

```shell
npm run docs:dev
```

3. 打开本地链接，最终效果如下

![image](../../../images/Build-Doc-Env-1.png)


