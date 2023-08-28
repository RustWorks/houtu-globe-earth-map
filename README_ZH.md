<div align="center">

  <h1><code>后土</code></h1>

  <strong>基于webgpu的高性能的真实地球渲染引擎</strong>

  <h3>
    <a href="#">暂无文档</a>
    <span> | </span>
    <a href="https://imdodo.com/s/211509">dodo交流群-后土地球</a>
  </h3>
</div>

# **注意：本项目还在试验阶段，请斟酌使用。**

## 截图

web墨卡托图层，瓦片资源来自[omniscale](https://maps.omniscale.net)，感谢。

![瓦片网格](./www/public/assets/i53pd-qxcsr.gif)

## 🔥介绍
使用bevy作为渲染引擎，面向web端的开源免费三维地球渲染引擎。

项目极早期阶段，望与诸位才子共建未来。

## 🚀特性
1. bevy作为渲染引擎，高度可拆卸，定制自己需要的功能。
2. 使用wasm+webgpu渲染web端，主打高性能高颜值。
3. 参考cesium，具备实用性，GIS图形的精确性。
## 🌎路线
详情查看仓库的[Projects](https://github.com/users/catnuko/projects/1)
1. - [x] 3d globe
2. - [x] 相机控制
3. - [ ] 基本几何图形，多边形，折线，点，圆，球，椭球等形状
4. - [x] 栅格瓦片图层，支持wgs84和web墨卡托投影的切片地图
5. - [ ] 矢量瓦片图层
6. - [ ] 倾斜摄影模型
7. - [ ] 地形
## 📖文档
1. 本仓库的GIS理论基础，[理论3D地球](https://www.taihe.one/tag/%E7%90%86%E8%AE%BA%E5%9C%B0%E7%90%83)

## 💻开发
```bash
# 运行程序
cd houtu-app
cargo run

# 暂时无法在浏览器中运行

# 用trunk在浏览器中运行
cd houtu-app
cargo install trunk wasm-bindgen-cli # 已有可不安装
trunk serve # 启动服务，控制台将给出服务地址，http://127.0.0.1:8080

# 用wasm-server-runner在浏览器中运行
cd houtu-app
cargo run --target wasm32-unknown-unknown
wasm-server-runner ../target/wasm32-unknown-unknown/debug/houtu-app.wasm

# 构建
cd houtu-app
cargo build

// 运行网站（暂无内容）
cd www
pnpm install
pnpm dev
```

## 💓贡献
佛系参与，强烈欢迎。👏👏👏
