## 1、创建 ISSUE
ISSUE 范围两类，需求和小的 Fixed，所以在标题层面，需要通过：RBIP-*和 MINOR 两个前缀来区分。

RBIP-*： 是标识有特性和功能添加，比如 RBIP-09，RBIP-10，后面的序号是递增的。
![image](../../images/doc-image9.png)

MINOR：标识是修复或者增加一些小的功能。则可以 MINOR：开头，接标题。
![image](../../images/doc-image10.png)

## 2、创建 Pull Request
如果PR 有关联的 ISSUE，必须在 PR 的内容中添加上：

close #issue_number
close 是固定的前缀， #也是固定的前缀，issue_number 表示这个PR 关联的 ISSUE 编号。比如：

![image](../../images/doc-image11.png)
#297，#292 就是对应的 ISSUE 编号。

比如需要提交一个解决 ISSUE #297 的 PR，则PR 内容需要包含

close #297
此时，当该 PR 被MERGE 时，这个 ISSUE 会自动被关闭。PR 合并后，PR 和 ISSUE 效果如下：

PR：
![image](../../images/doc-image12.png)

ISSUE：
![image](../../images/doc-image13.png)
