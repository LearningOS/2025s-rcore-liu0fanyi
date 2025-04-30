# 简单总结你实现的功能（200字以内，不要贴代码）。
* 添加要求的sys_call trace的读/写/计数功能
* 在TaskControlBlock里添加一个syscall_counter数组用于计数
* 数组的index用已有的sys_call的id
* 添加plus和读取syscall_counter的接口
* 在os的syscall里，plus计数

# 完成问答题。
1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 三个 bad 测例 (ch2b_bad_*.rs) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。
  * sbi Version 0.2.0-alpha.2
  * ch2b_bad_address，显示PageFault in application, 错误地址和错误指令
  * ch2b_bad_instructions 是非法instruction
  * ch2b_bad_register 是非法instruction
2.
  1. L40：刚进入 __restore 时，sp 代表了什么值。请指出 __restore 的两种使用情景。
    * 刚进入_restore的时候，sp是S stack栈顶
    * 用于sys_call后trap结束从S恢复到U
    * 用于切换不同的task时启动app
  2. L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。  
    * 特殊处理了sscratch, sstatus, sepc
    * sscratch保存了U的sp
    * sstatus保存了特权级信息，指示现在要返回的是U态
    * sepc保存了trap结束后执行的下一条instruction地址

  3. L50-L56：为何跳过了 x2 和x4？  
    * x2是sp，需要使用sscratch恢复
    * x4一般不用

  4. L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？  
    * 交换了彼此，sp变成U stack而sscratch变成S stack

  5. __restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？  
    * sret
    * 因为sp指向了U态，又trap里修改了sepc，指向了ecall的下一条instruction

  6. L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？
    * sp变成S stack而sscratch变成了U stack
  7. 从 U 态进入 S 态是哪一条指令发生的？
    * ecall

# 加入 荣誉准则 的内容。否则，你的提交将视作无效，本次实验的成绩将按“0”分计。
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
  无
2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
  无
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
