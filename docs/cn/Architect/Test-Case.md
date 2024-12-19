## 安装 cargo-nextest
RobustMQ 的集成测试依赖cargo-nextest来加快测试用例的运行速度，所以跑集成测试的时候，需要安装 cargo-nextest。

安装教程，请参考这个文档：https://nexte.st/docs/installation/pre-built-binaries/

- cargo-binstall
```
cargo binstall cargo-nextest --secure
```

- Linux 安装
```
curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
```

- Linux aarch64
```
curl -LsSf https://get.nexte.st/latest/linux-arm | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
```

- mac 安装
```
curl -LsSf https://get.nexte.st/latest/mac | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
```

- windows

```
curl -LsSf https://get.nexte.st/latest/windows-tar | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
```

## 运行测试

在根目录输入相应的make指令

- 单元测试

  所有测试单元测试用例
  ```
  make test
  ```

- 集成测试

  MQTT Broker:
  ```
  make mqtt-ig-test
  ```

  Placement Center:
  ```
  make place-ig-test
  ```

  Journal Engine:
  ```
  make journal-ig-test
  ```
