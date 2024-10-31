# 编程作业

## 重写 sys_get_time 和 sys_task_info

逐页copy数据。其他代码逻辑基本不变。

## mmap 和 munmap 匿名映射

Each process has its own 'user address space' (i.e. taskcontrolblock.memoryset), `mmap()` append a `MapArray` to it.

MemorySet is allocated by `FrameAllocator`
