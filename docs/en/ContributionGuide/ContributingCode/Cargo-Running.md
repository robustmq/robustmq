# Running with Cargo

1. **Run standalone by meta-service**

   ```
   cargo run --package cmd --bin meta-service -- --conf=config/meta-service.toml
   ```

   If the following output is displayed, it indicates that the meta-service has started successfully:
   ![image](../../../images/Cargo-Running-1.png)

2. **Run standalone by mqtt-server**

   ```
   cargo run --package cmd --bin mqtt-server -- --conf=config/mqtt-server.toml
   ```

   If the following output is displayed, it indicates that the mqtt-server has started successfully:![Cargo-Running-run-mqtt-server-2.png](../../../images/Cargo-Running-run-mqtt-server-2.png)


3. **Run standalone by journal-server**

   ```
   cargo run --package cmd --bin journal-server -- --conf=config/journal-server.toml
   ```
   The following output indicates that the journal-server has started successfully:
   ![Cargo-Running-run-journal-server-3.png](../../../images/Cargo-Running-run-journal-server-3.png)
