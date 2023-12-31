# 基于Rust 的 sysy 优化编译器

## 使用

生成未优化的rv64gc汇编代码:

`compiler a.sy -S -o a.s` 

生成优化后的rv64gc汇编代码:

`compiler a.sy -S -o a.s -O1`

## TODOq

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
   * [ ] 指令调度: 并行优化系数方案

5. cicd
   

