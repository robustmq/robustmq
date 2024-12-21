既然你已经看到这里了（[默认基础开发环境您已经配置好了](./Build-Develop-Env.md)），如果你发现了项目的bug或者让你不开心的地方，是不是想亲自下场，那么你可以提交一个 Pull Request 来帮助我们改进。

下面是一个完整的 Pull Request 流程示例，一个修改bug的示例：

1. Fork 项目

   先点击 https://github.com/robustmq/robustmq 的 Star 按钮，然后点击 Fork 按钮，复制你的 Fork 仓库地址 https://github.com/你的用户名/robustmq.git

2. 克隆你的仓库

     创建一个 work 目录

     ```shell
     mkdir work
     cd work
     git clone https://github.com/你的用户名/robustmq.git
     ```

3. 创建新分支

     ```shell
     cd robustmq
     git checkout -b pr-example
     ```

4. 修改文件

     ```rust
        #[tokio::main]
        async fn main() {
            #我们在这里添加新代码
            println!("Hello, robustmq!");
            let args = ArgsParams::parse();
            init_placement_center_conf_by_path(&args.conf);
            init_placement_center_log();
            let (stop_send, _) = broadcast::channel(2);
            let mut pc = PlacementCenter::new();
            pc.start(stop_send).await;
        }
     ```

5. 提交修改

   如果你开发环境已经配置好了`pre-commit`, 会帮你检查代码风格并自动进行单元测试 `make test`

   ```shell
   git add .
   git commit -m "fix bug"

   ```
   如果这些都没问题，我们可以进行集成测试了

6. 集成测试

    | 测试项                      | 命令                 |
    |----------------------------|---------------------|
    | 单元测试                    | make test           |
    | 集成测试 MQTT Broker        | make mqtt-ig-test   |
    | 集成测试 Placement Center   | make place-ig-test  |
    | 集成测试 Journal Engin      | make journal-ig-test|


   ```shell

   # 集成测试
   make mqtt-ig-test
   make place-ig-test
   make journal-ig-test
   # 如果这么多步都成功了，那么你的 Pull Request 就可以提交了。

   git push origin pr-example

   ```

7. (选读)如果你遇到测试完成后准备发起pr时robustmq的主分支更新了怎么办

    合并 robustmq 的主分支到你的分支，然后再 push

    ```shell
    cd robustmq
    # 添加原始项目仓库作为远程仓库（如果尚未添加）：
    git remote add upstream https://github.com/robustmq/robustmq.git
    # 验证是否成功添加了 upstream 远程仓库：
    git remote -v
    # 获取原始项目（upstream）所有分支的最新更改：
    git fetch upstream
    # 确保你当前的工作分支是你要合并到的分支
    git checkout pr-example
    # 合并原始仓库的 main 分支到你的分支
    git merge upstream/main
    ```
    此时你应该回到第`6`小节，进行集成测试之后

    ```shell
    # 提交合并
    git commit -m "Merge upstream main into pr-example"
    # push
    git push origin pr-example
    ```


8. 创建 Pull Request

    让我们回忆一下[githu贡献指南](./GitHub-Contribution-Guide.md),打开你fork地址，点击 New Pull Request 按钮，填写标题和内容，点击 Create Pull Request 按钮，就可以完成了。

    ```md
    标题 fix: fix bug example
    内容  fix bug balabala
    ```

9.  等待合并

    这个过程github ci 会再次对你分支的代码进行检查，如果检查通过，那么你的 Pull Request 就会被合并了。等待 RobustMQ 的维护者审核，如果通过了，你的 Pull Request 就会被合并了。
