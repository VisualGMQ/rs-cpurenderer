这里是一些Benchmark的结果，用来记录一些可以优化的点：

# 清屏的效率

首先是每一次渲染循环进入时，对整个`ColorAttachment`清屏的效率（这个贼慢，不清屏有1000+fps，清了之后骤降）：

|存储类|debug-fps|release-fps|在绘制Sonic的情况下debug-fps|在绘制Sonic的情况下release-fps|
|--|--|--|--|--|
|ImageBase<Color>|60|300|8|110|
|PureU8Image|16|300|11|140|
|GroupedImage|24|700|5|110|

现在用的第二种方法，在Release模式下最快。
