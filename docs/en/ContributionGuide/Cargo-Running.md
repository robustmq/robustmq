# Cargo Running
## Overview
1. Run standalone by placement - center
```
cargo run --package cmd --bin placement - center -- --conf=config/placement - center.toml
```
The following information will be output, indicating that the placement - center has been successfully started:
![image](../../images/doc - image6.png)

2. Run standalone by mqtt - server
```
cargo run --package cmd --bin mqtt - server -- --conf=config/mqtt - server.toml
```
The following information will be output, indicating that the mqtt - server has been successfully started:
![image](../../images/doc - image7.png)
