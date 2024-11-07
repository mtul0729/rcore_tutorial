# 编程作业实现总结

spawn 实际上是直接从elf文件创建一个 TaskControlBlock，并在 当前进程的 TCB 的 childern 列表之中加入它

stride 调度算法方面，暴力搜索stride最小的tcb,将其stride 加 pass， 然后调度

# 问答作业

1. 实际仍然执行p2。p2.stride 累加后溢出，变为4，小于p1.stride 于是仍然执行p2
2. 优先级>=2,则stride单次增加不超过 BigStride / 2,且每次调度stride最小的,因此差值不超过 BigStride / 2
3. 
```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let cmp = if (self.0 > other.0 && (self.0 - other.0) > BigStride / 2 )|| (self.0 < other.0 && (other.0 - self.0) > BigStride / 2 ){
            (self.0 > other.0 ).reverse()
        }else{
            self.0 > other.0 
        };
        Some(cmp)
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
``` 

# 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无

1. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
