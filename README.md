# WordCloud

WordCloud 是一个基于 WebAssembly 的词云生成库，使用 Rust 编写，并通过 wasm-bindgen 与 JavaScript 进行交互。该库允许用户自定义词云的大小、字体、旋转范围和螺旋类型等参数，并生成相应的词云布局。
js 版算法可以参考：https://github.com/timdream/wordcloud2.js

## 特性

- 自定义词云大小
- 自定义字体和字体粗细
- 自定义词云中词条的最小和最大尺寸
- 自定义词条的旋转范围
- 支持多种螺旋类型

## 安装

要使用 WordCloud，您需要先安装 Rust 和 wasm-pack。然后，您可以按照以下步骤构建和运行项目：
1. 安装 Rust 和 wasm-pack：

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   cargo install wasm-pack
   ```

2. 构建项目：

   ```sh
   wasm-pack build --target web
   ```

3.运行js项目

    ```sh
    cd web
    npx http-server . -p 8080
    ```