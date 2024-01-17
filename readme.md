# 基于Rust 的 sysy 优化编译器

## 使用

生成未优化的rv64gc汇编代码:

`compiler a.sy -S -o a.s` 

生成优化后的rv64gc汇编代码:

`compiler a.sy -S -o a.s -O1`

## 验证

```
# 拉取更新子模块内容,可以看到当前项目路径下data文件夹中出现很多测试用例集文件夹
git submodule update --init --recursive
# 使用rv64gc-linux-gnu-gcc-12交叉编译器编译目标用例
# 如下命令会编译./data/functional中的用例,并结构化的把编译产物放到各个文件夹中
./scripts/test.sh functional
# 与自身编译器输出进行对比,需要把编译后的编译器放入目标用例文件夹下面
cargo build --release && cp ./target/release/compiler ./data/functional/compiler 
./scripts/test.sh functional

## 更多使用方式
# ./scripts/test.sh $target_dir $task_script_path
# $target_dir 用于 指定 测试的 用例集 为 ./data/$target_dir
# $task_script_path 指定 测试任务执行 时 镜像中 所在测试用例目录下执行的脚本
# 可以通过配置这两个参数实现各种自定义的编译测试任务
```

## TODO

1. 架构

    * [ ] 完善前后端分离设计方便组合前中后端代码
    * [ ] 

2. 前端
    
    * [ ] 前端源代码解析方案调研
    * [ ] 前端IR设计与代码实现
    * [ ] 完成对c语言的解析
    * [ ] 完成对sy与语言的解析
    * [ ] 中端IR生成
    * [ ] 兼容sy,c的源代码解析方案
    * [ ] 基础优化: 常量传播
    * [ ] 基础优化: 常量折叠

3. 中端
    * [ ] 中端IR设计与代码实现
    * [ ] 中端IR验证方案: 生成llvm IR
    * [ ] 数据流分析框架
    * [ ] 基础优化: 死代码删除
    * [ ] 基础优化: 函数内联
    * [ ] 基础优化: 尾递归
    * [ ] 循环优化: 循环不变量外提
    * [ ] 循环优化: 循环展开
    * [ ] 循环优化: 自动并行化
    * [ ] 循环优化: 结构优化

4. 后端
   * [x] 搭建汇编生成框架
   * [ ] 完成后端IR设计和实现
   * [ ] 完成后端IR验证方案: 解析LLVM IR?
   * [ ] 后端IR生成
   * [ ] 基础优化: 乘除法优化
   * [ ] 基础优化: 块重排
   * [ ] 寄存器分配: 图着色分配
   * [ ] 寄存器分配: 统一代价模型+ILP
   * [ ] 指令调度: 相邻指令数据依赖优化
   * [ ] 指令调度: CPI 检测 方案

5. cicd
   

