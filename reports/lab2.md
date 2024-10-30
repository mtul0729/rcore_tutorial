# 编程作业

## 重写 sys_get_time 和 sys_task_info

## mmap 和 munmap 匿名映射

所有进程通过每次mmap申请的内存用一个`MapArea`管理，所有的`MapArea`由OS通过一个`MemorySet`类型的静态全局变量`USER_HEAP`管理
