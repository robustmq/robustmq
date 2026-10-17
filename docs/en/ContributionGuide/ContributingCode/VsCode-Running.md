# Running in VSCode

1. **Download the Project Code**
    - Clone the project repository to your local machine.

2. **Initialize the Environment**
    - Ensure that all necessary dependencies and tools are installed and configured.

3. **Run the broker from the repository root**
    - The `cmd` package currently exposes one server binary:
   ```
   make run
   ```

4. **Debug in VSCode**
    - Select the `broker-server` binary and pass `--conf=config/server.toml` as the program argument. The broker starts the configured `meta`, `broker`, and `engine` roles together. The terminal equivalent is `make run`.

   ![image](../../../images/VSCode-Running-1.png)

    - Start the service as needed.
