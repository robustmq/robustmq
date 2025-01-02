# Github pull request
## Overview
Now that you've seen this ([build your develop](./Build-Develop-Env.md)), if you've found a project bug or something that upsets you, don't you want to get involved and help us improve by submitting a Pull Request?

Below is a complete Pull Request process example, an example of fixing a bug:

1. Fork the project

   Go to https://github.com/robustmq/robustmq and click the Star button, then click the Fork button, and copy the address of your Fork repository https://github.com/your_username/robustmq.git

2. Clone your repository

   Create a work directory

   ```shell
   mkdir work
   cd work
   git clone https://github.com/your_username/robustmq.git
   ```

3. Create a new branch

   ```shell
   cd robustmq
   git checkout -b pr-example
   ```

4. Modify the file

   ```rust
   #[tokio::main]
   async fn main() {
       # Add new code here
       println!("Hello, robustmq!");
       let args = ArgsParams::parse();
       init_placement_center_conf_by_path(&args.conf);
       init_placement_center_log();
       let (stop_send, _) = broadcast::channel(2);
       let mut pc = PlacementCenter::new();
       pc.start(stop_send).await;
   }
   ```

5. Commit the changes

   If your development environment has `pre-commit` configured, it will check your code style and automatically run unit tests `make test`

   ```shell
   git add .
   git commit -m "fix bug"
   ```
   If all goes well, we can proceed to integration testing

6. Integration testing

   | Test item                     | Command                |
      |------------------------------|-----------------------|
   | Unit tests                   | make test             |
   | Integration test MQTT Broker | make mqtt-ig-test     |
   | Integration test Placement Center | make place-ig-test   |
   | Integration test Journal Engine | make journal-ig-test |

   ```shell

   # Integration testing
   make mqtt-ig-test
   make place-ig-test
   make journal-ig-test
   # If all these steps are successful, then your Pull Request can be submitted.

   git push origin pr-example

   ```

7. (Optional) What if you encounter the main branch of robustmq updating after the test is completed and ready to initiate a pr

   Merge the main branch of robustmq into your branch, and then push

   ```shell
   cd robustmq
   # Add the original project repository as a remote repository (if not already added):
   git remote add upstream https://github.com/robustmq/robustmq.git
   # Verify that the upstream remote repository has been successfully added:
   git remote -v
   # Fetch the latest changes from all branches of the original project (upstream):
   git fetch upstream
   # Ensure that the branch you are currently working on is the branch you want to merge into
   git checkout pr-example
   # Merge the main branch of the original repository into your branch
   git merge upstream/main
   ```
   At this point, you should go back to section `6`, conduct integration testing, and then

   ```shell
   # Commit the merge
   git commit -m "Merge upstream main into pr-example"
   # Push
   git push origin pr-example
   ```

8. Create a Pull Request

   Recall the [GitHub Contribution Guide](./GitHub-Contribution-Guide.md), open your fork address, click the New Pull Request button, fill in the title and content, and click the Create Pull Request button to complete.

   ```md
   Title: fix: fix bug example
   Content: fix bug balabala
   ```

9. Wait for merge

   This process will have GitHub CI check your branch's code again, and if the check passes, your Pull Request will be merged. Wait for the maintainers of RobustMQ to review, and if it passes, your Pull Request will be merged.
