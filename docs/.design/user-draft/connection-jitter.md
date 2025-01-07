# 消息抖动功能用户文档
## 功能介绍
RobustMQ支持自动封禁那些被检测到短时间内频繁登录的客户端，
并且在一段时间内拒绝这些客户端的登录，
以避免此类客户端过多占用服务器资源而影响其他客户端的正常使用。

对于连接抖动检测功能当前只会封禁客户端 ID，
并不封禁用户名和 IP 地址，
即该机器只要更换客户端的ID后依然能够继续登录。

抖动检测功能默认关闭，您可以通过命令行开启对应的抖动检测功能,
配置文件支持连接抖动检测暂未支持。

## 如何使用连接抖动检测
### 通过命令行开启连接抖动检测功能
我们通过如下命令启用连接抖动检测功能
```shell
robust-cli mqtt connection-jitter --enable=true
```
默认情况下拥有三个相关的参数内容:
- 检测时间窗口: 您可以指定系统监视客户端抖动行为的持续时间。默认值为`1`分钟。
- 最大断连次数: 您可以指定在检测窗口时间内允许的`MQTT`客户端的最大断开连接次数。
  它允许您设定准确的标准来识别和响应表现出抖动行为的客户端。
  默认值为`15`。
- 封禁时长: 您可以指定客户端被封禁的时间长度。默认值为`5`分钟。

您可以通过如下的命令修改上述的几个命令配置:
- 修改检测时间窗口(默认单位为分钟)
```shell
roubst-cli mqtt connection-jitter --window-time=1
```
- 修改最大断连次数
```shell
roubst-cli mqtt connection-jitter --max-client-connections=15
```
- 修改封禁时长
```shell
roubst-cli mqtt connection-jitter --ban-time=5
```
- 修改设置单位(是否考虑)
```shell
roubst-cli mqtt connection-jitter --time-unit=seconds/minutes/hours/days/months
```
### TODO 配置文件支持连接抖动检测
