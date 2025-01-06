# Running with Cargo

1. **Run standalone by placement-center**

   ```
   cargo run --package cmd --bin placement-center -- --conf=config/placement-center.toml
   ```

   If the following output is displayed, it indicates that the placement-center has started successfully:
   ![image](../../../images/Cargo-Running-1.png)

2. **Run standalone by mqtt-server**

   ```
   cargo run --package cmd --bin mqtt-server -- --conf=config/mqtt-server.toml
   ```

   If the following output is displayed, it indicates that the mqtt-server has started successfully:
   ![image](../../../images/Cargo-Running-2.png)
