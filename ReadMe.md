A cpu 3D renderer based on pure rust.

使用纯Rust编写的3D软渲染器。

## 目前的效果

Blinn-Phong着色中的定向光:

![dir_light](./snapshot/dir_light.png)

Blinn-Phong着色中的点光源:

![dot_light](./snapshot/dot_light.png)

Blinn-Phong着色中的聚光光源:

![spot_light](./snapshot/spot_light.png)

## 目前实现的功能

- [x] obj文件加载
- [x] 线框模型的绘制（直线绘制使用`Bresenham`算法）
- [x] 剔除
    - [x] 面剔除（模型空间面剔除）
    - [x] 视锥剔除（相机空间面剔除）
- [x] 裁剪
    - [x] 直线裁剪（使用`Cohen-Sutherland`算法）
    - [x] 近平面三角面裁剪
- [x] 摄像机
    - [x] 基于欧拉角的自由旋转摄像机
    - [x] 基于视点和坐标的LookAt相机
- [x] 深度测试
- [x] 纹理绘制
- [x] 光栅化
- [x] 可编程着色器
- [ ] 着色与光照
    - [x] BlinnPhong光照
    - [ ] 法线贴图
    - [x] 平行光
    - [x] 点光源
    - [x] 聚光

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
