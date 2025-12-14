# PianoOS
Base on [rCore-Tutorial-Book-v3](https://rcore-os.cn/rCore-Tutorial-Book-v3/index.html)

## Quik Start
```sh
cargo all -f float -f nested_trap
```
the details is in `.cargo/config/toml`


## TODO:
- la的支持
- ch3练习：获取任务信息
- ch3练习：打印调用堆栈
- 能在内核做其他工作时进行切换，即此时的多内核栈
- 页表部分需要arch无关
- 增加更多的UT
