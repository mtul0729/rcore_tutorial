# rCore-Camp-Code-2024A

个人学习项目。代码在branch ch[1-8]

## 实验环境配置

rCore开发的环境配置比较复杂，训练营介绍的配置方法步骤繁琐，我使用nix定义了一个开发环境[flake.nix](./flake.nix)。

flake的使用方法可以参考：<https://wiki.nixos.org/wiki/Flakes>

## 学习总结

### 第一/二章

阅读这两章需要先了解riscv，特别是特权级相关设计。

主要参考:

* [RISC-V开放架构设计之道][1]
* [The RISC-V Reader: An Open Architecture Atlas][2] 前一个的英文原著，而且有能下载到单独的RISC-V Green Card，方便查阅
* [RISC-V Instruction Set Manual][3] 完整详细

### 第三章：多道程序与分时多任务

学习了系统调用，陷入等知识，上下文切换过程中通用寄存器和CSR的使用加深了我对riscv特权级设计的理解。

本章练习较为简单。

### 第四章：地址空间

SV39的设计又引入了若干相关的寄存器，如satp, pmp csr。查阅[riscv manul][3]以加深理解。

本章练习中，为了处理*请求映射已经被映射的页*的错误，我使用了`Result`错误传递，无法想象如果不使用`Result`和`?`语法糖我的代码会多么丑陋。然而很奇怪，整个rcore中极少使用`Result`。

### 第五章：进程及进程管理

本章内容比较轻松，完善了TCB的设计并实现`fork()`和`exec()`系统调用。

本章练习也比较简单。

### 第六章：文件系统

easy-fs is **NOT** easy！层层抽象几乎让我晕头转向！

尽管如此，easy-fs囊括了superblock、索引节点、blockcache等现代文件系统中的基础概念，似乎不能再精简了。

link和unlink操作主要是查找inode并创建/删除目录项。在inode_block里创建与删除目录项无非是一些线性序列的操作，但由于没有封装成`&[DirEntry]`，需要手动操作，比较费劲。将来有空我会尝试改进一下。

改进方法可能类似于： `impl From<DiskInode> for &[DirEntry]`

### 第七章：进程间通信

本章内容较少，但进程间通信是个大话题，还需拓展学习。

### 第八章：并发

学习了多线程的同步与互斥。

练习：学习了死锁检测算法

[1]: https://ysyx.oscc.cc/books/riscv-reader.html
[2]: http://www.riscvbook.com/
[3]: https://github.com/riscv/riscv-isa-manual

## 学习资料

非常感谢开源操作系统训练营提供的优质课程：

* [Soure Code of labs for 2024A](https://github.com/LearningOS/rCore-Camp-Code-2024A)
* Concise Manual: [rCore-Camp-Guide-2024A](https://LearningOS.github.io/rCore-Camp-Guide-2024A/)
* Detail Book [rCore-Tutorial-Book-v3](https://rcore-os.github.io/rCore-Tutorial-Book-v3/)
* [ch1](https://learningos.github.io/rCore-Camp-Code-2024A/ch1/os/index.html) [ch2](https://learningos.github.io/rCore-Camp-Code-2024A/ch2/os/index.html) [ch3](https://learningos.github.io/rCore-Camp-Code-2024A/ch3/os/index.html) [ch4](https://learningos.github.io/rCore-Camp-Code-2024A/ch4/os/index.html)
* [ch5](https://learningos.github.io/rCore-Camp-Code-2024A/ch5/os/index.html) [ch6](https://learningos.github.io/rCore-Camp-Code-2024A/ch6/os/index.html) [ch7](https://learningos.github.io/rCore-Camp-Code-2024A/ch7/os/index.html) [ch8](https://learningos.github.io/rCore-Camp-Code-2024A/ch8/os/index.html)
* [Learning Resource](https://github.com/LearningOS/rust-based-os-comp2022/blob/main/relatedinfo.md)
