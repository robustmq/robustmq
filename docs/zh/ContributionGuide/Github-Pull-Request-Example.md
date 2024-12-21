既然你已经看到这里了（[默认基础开发环境您已经配置好了](./Build-Develop-Env.md)），如果你发现了项目的bug或者让你不开心的地方，是不是想亲自下场，那么你可以提交一个 Pull Request 来帮助我们改进。

下面是一个完整的 Pull Request 流程示例，下面是一个修改bug的示例：

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

   如果你开发环境已经配置好了，pre-commit 会帮你检查代码风格和单元测试 `make test`

   ```shell
   git add .
   git commit -m "fix bug"

   ```
   如果这些都没问题，我们可以进行集成测试了

6. 集成测试

   ```shell

   # 集成测试
   make mqtt-ig-test
   make place-ig-test
   make journal-ig-test
   # 如果这么多步都成功了，那么你的 Pull Request 就可以提交了。

   git push origin pr-example

   ```


7. 创建 Pull Request

    让我们回忆一下[githu贡献指南](./GitHub-Contribution-Guide.md),打开你fork地址，点击 New Pull Request 按钮，填写标题和内容，点击 Create Pull Request 按钮，就可以完成了。

    ```md
    标题 fix: fix bug example
    内容  fix bug balabala
    ```

8. 等待合并

    这个过程github ci 会再次对你分支的代码进行检查，如果检查通过，那么你的 Pull Request 就会被合并了。等待 RobustMQ 的维护者审核，如果通过了，你的 Pull Request 就会被合并了。
