# kvs-demo
根据[课程 rust project-5](https://github.com/pingcap/talent-plan)修改，使用async/await新语法。
 * 添加scan key命令，key可以是正则表达式，使用[regex库](https://github.com/rust-lang/regex)
 * 客户端添加console命令，交互形式。

### 环境
 * nightly-x86_64-unknown-linux-gnu
 * rustc 1.37.0-nightly (d3e2cec29 2019-06-26)

### 问题
 * [ ] 交互模式错误处理，不应该直接退出程序。
 * [ ] scan regex命令兼容 redis keys 命令。
