A cpu 3D renderer based on pure rust.

使用纯Rust编写的3D软渲染器。

[B站视频教程](https://www.bilibili.com/video/BV1Nv4y1p79o/?spm_id_from=333.999.0.0&vd_source=cb60c670b9b482f512a7f10d235b9b30)

之前写了一份完整的，源码放在Release下了。这个是重置版本。重置完之后会删掉Release版本。

## 目前的显示

![snapshot](./snapshot/snapshot.gif)

## 工程运行

只有一个例子：`examples/sandbox.rs`，用

```bash
cargo run --example sandbox
```

运行。

本工程由两种软渲染：纯粹为了在CPU上快速运行的CPU软渲染(`./src/cpu_renderer.rs`)，以及模拟GPU原理的GPU软渲染(`./src/gpu_renderer.rs`)。使用`features`可以指定运行某种：

```bash
cargo run --example sandbox --features cpu
cargo run --example sandbox --features gpu
```

默认是CPU渲染。

## 参考

书籍：
* 《3D游戏编程大师技巧》
* 《Fundamentals of Computer Graphics》

视频：
* [【GAMES101-现代计算机图形学入门-闫令琪】](https://www.bilibili.com/video/BV1X7411F744/?share_source=copy_web&vd_source=e1b8baee842192a0e6b2b7d9ef8e10ef)中关于光栅化的部分。

齐次空间裁剪:
* [clipping using homogeneous coordinates](https://citeseerx.ist.psu.edu/document?repid=rep1&type=pdf&doi=497a973878c87e357ff4741b394eb106eb510177)
* [polygon codec/homogeneous clipping](https://fabiensanglard.net/polygon_codec/)

投影矩阵推导：
* [gl_perspective_matrix](http://www.songho.ca/opengl/gl_projectionmatrix.html)

透视投影矫正：
* [韦易笑大神的博客](https://www.skywind.me/blog/archives/1363)
* [维基上关于仿射纹理变换和矫正部分](https://en.wikipedia.org/wiki/Texture_mapping#Affine_texture_mapping)
* [Kavita Bala的文章](https://www.cs.cornell.edu/courses/cs4620/2015fa/lectures/PerspectiveCorrectZU.pdf)

其他:
* [OpenGL 和 DirectX 是如何在只知道顶点的情况下得出像素位置的](https://www.zhihu.com/question/48299522/answer/799333394)
* [RenderHelp项目](https://github.com/skywind3000/RenderHelp)
* [mini3d项目](https://github.com/skywind3000/mini3d)
* [光栅化实现](https://www.scratchapixel.com/lessons/3d-basic-rendering/rasterization-practical-implementation/overview-rasterization-algorithm.html)