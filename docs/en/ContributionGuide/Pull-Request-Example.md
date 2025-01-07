# PR Submission Example

Since you've made it this far (assuming you have already set up the basic development environment as described in [ContributingCode/Build-Develop-Env.md](ContributingCode/Build-Develop-Env.md)), if you've found a bug in the project or something that bothers you, you might want to get involved and submit a Pull Request to help us improve.

Here is a complete example of the Pull Request process, specifically for fixing a bug:

1. **Fork the Project**

   Start by clicking the Star button on [https://github.com/robustmq/robustmq](https://github.com/robustmq/robustmq), then click the Fork button to copy your forked repository address: [https://github.com/your-username/robustmq.git](https://github.com/your-username/robustmq.git).

2. **Clone Your Repository**

   Create a `work` directory:

   ```shell
   mkdir work
   cd work
   git clone https://github.com/your-username/robustmq.git
   ```

3. **Create a New Branch**

   ```shell
   cd robustmq
   git checkout -b pr-example
   ```

4. **Modify Files**

   ```rust
   #[tokio::main]
   async fn main() {
       // Add new code here
       println!("Hello, robustmq!");
       let args = ArgsParams::parse();
       init_placement_center_conf_by_path(&args.conf);
       init_placement_center_log();
       let (stop_send, _) = broadcast::channel(2);
       let mut pc = PlacementCenter::new();
       pc.start(stop_send).await;
   }
   ```

5. **Commit Changes**

   If you have `pre-commit` configured in your development environment, it will help check code style and automatically run unit tests `make test`.

   ```shell
   git add .
   git commit -m "fix bug"
   ```

   If everything is fine, we can proceed to integration testing.

6. **Integration Testing**

   | Test Item                      | Command                 |
      |----------------------------|---------------------|
   | Unit Tests                  | make test           |
   | Integration Test MQTT Broker | make mqtt-ig-test   |
   | Integration Test Placement Center | make place-ig-test  |
   | Integration Test Journal Engine | make journal-ig-test|

   ```shell
   # Integration testing
   make mqtt-ig-test
   make place-ig-test
   make journal-ig-test
   # If all these steps are successful, you can submit your Pull Request.

   git push origin pr-example
   ```

7. **(Optional) What if the main branch of robustmq is updated before you are ready to submit your PR?**

   Merge the main branch of robustmq into your branch, then push.

   ```shell
   cd robustmq
   # Add the original project repository as a remote repository (if not already added):
   git remote add upstream https://github.com/robustmq/robustmq.git
   # Verify if the upstream remote repository has been successfully added:
   git remote -v
   # Fetch the latest changes from the original project (upstream):
   git fetch upstream
   # Ensure you are on the branch you want to merge into:
   git checkout pr-example
   # Merge the main branch of the original repository into your branch
   git merge upstream/main
   ```

   At this point, you should go back to section `6` and perform integration testing again.

   ```shell
   # Commit the merge
   git commit -m "Merge upstream main into pr-example"
   # Push
   git push origin pr-example
   ```

8. **Create a Pull Request**

   Let's recall the [GitHub Contribution Guide](./GitHub-Contribution-Guide.md). Open your forked repository, click the New Pull Request button, fill in the title and content, and click the Create Pull Request button to complete.

   ```md
   Title: fix: fix bug example
   Content: Fix bug blablabla
   ```

9. **Wait for Merge**

   During this process, GitHub CI will check the code in your branch again. If the check passes, your Pull Request will be merged. Wait for the RobustMQ maintainers to review it. If approved, your Pull Request will be merged.
