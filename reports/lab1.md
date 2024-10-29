# 功能总结

在`TaskControlBlock` struct里增加了`syscall_times`和`init_time`，分别用于记录该task的任务使用的系统调用及调用次数和首次被sheduled的时间.同时在`run_first_task`和`增加*更新这两个成员变量的逻辑
run_next_task`中增加更新这两个成员变量的逻辑代码。
- 对于任务状态，代码逻辑比较简单，不作介绍
- 对于任务使用的系统调用及调用次数，每次syscall都更新`syscall_times`中对应的值，再增加一个获取`syscall_times`的函数即可。
- 对于系统调用时刻距离任务第一次被调度时刻的时长，将调用sys_task_info的时间减去`init_time`就可得到。

# 简答题

1. 使用课程默认的sbi版本
  - `ch2_bad_address.rs`向0x0地址写数据，触发StoreFault Exception，os输出如下错误:
    ```
    [kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
    ```
  - `ch2_bad_instruction.rs`执行S特权级的指令`sret`,触发IllegalInstruction Exception，os输出如下错误:
    ```
    [kernel] IllegalInstruction in application, kernel killed it.
    ```
  - `ch2b_bad_register.rs`对S特权级的寄存器`sstatus`执行写操作,触发IllegalInstruction Exception，os输出如下错误:
    ```
    [kernel] IllegalInstruction in application, kernel killed it.

    ```
2.  
  1. 此时a0代表上一次运行的任务的任务上下文的指针。`__restore`的两种使用情景：
    - os执行完成trap后，从`trap_handler`返回到`__restore`恢复trap上下文
    - 切换任务时,由`__restore`恢复下一个任务的上下文
  2. 特殊处理了sstatus、sepc和sscratch。
    - sstatus的SPP等字段给出 Trap 发生之前 CPU 处在哪个特权级,`sret`指令将据此切换到对应特权级
    - sepc保存trap发生时的pc值，用以计算进入用户态时应该执行的指令地址
    - sscratch保存user stack,将其读到sp,为用户态程序指示对应的user stack
  3. x2(sp)将在最后从sscratch中读取,因此跳过; x4(tp)一般不会用到,因此不加载
  4. sp指向user stack,sscatch指向kernel stack
  5. `sret`指令。sstatus的SPP字段指示trap之前处于U特权级,且当前处于S特权级，因此`sret`将导致进入用户态
  6. sscatch指向user stack,sp指向kernel stack
  7. `ecall`

# 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

[riscv-isa-manual](https://github.com/riscv/riscv-isa-manual/releases/)

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
