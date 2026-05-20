# Chinese Input Encode

高效求解特定输入法下中文文本的全局最优编码

[![Latest Release](https://img.shields.io/github/v/release/GarthTB/cn-input-encode?color=0FBF3E&label=Latest&logo=github)](https://github.com/GarthTB/cn-input-encode/releases/latest)
![Tech Stack](https://skillicons.dev/icons?i=git,rust,windows)
[![License MIT](https://img.shields.io/badge/License-MIT-750014)](https://mit-license.org)

## ✨ 特性

- 🚀 **极致性能**：每秒编码数百万字
- 🎯 **精确优化**：DP 算法确保编码全局最优
- 🌊 **内存友好**：分块流式处理，内存占用恒定
- 📊 **编码分析**：直接获取 30+ 项编码统计数据
- 🚂 **配置驱动**：通过 TOML 文件配置所有参数

## 📥 使用

### 预编译包

1. 从 [Releases](https://github.com/GarthTB/cn-input-encode/releases/latest) 下载压缩包并解压
2. 在 `cfg/config.toml` 中配置参数
3. 运行 `cn-input-encode`，结果将输出至输入文件所在目录，默认为 `{原名}-code.txt`，若已存在则自动加编号，无覆写风险

### 原理

特定输入法下，同一文本往往有多种打法（如不同的分词方案）。软件通过动态规划求出**总开销最小**的打法。这里的“开销”是一个抽象权重，由配置文件定义。

### 词库要求

1. **RIME 格式**：每行格式为 `词\t编码` 或 `词\t编码\t权重`
2. **无需选重键**：软件可自动计算选重数字和翻页符
3. **如实反映击键**：不含不代表按键的字符

### 配置说明

- **`cost-map.tsv`（键对开销表）**：定义两键连击的开销，数值越小越优，可自由修改。例如将所有值设为相同常数，即可求解“全局最短码长”。默认数据源自
  `[1]陈一凡,张鹿,周志农. 键位相关速度当量的研究[J].中文信息学报,1990,(04):12-18+11.` 的击键时间间隔实测均值。
- **词库第三列（权重）**：决定重码时的出词顺序，数字越大越优先上屏

## ℹ 关于

- 地址：https://github.com/GarthTB/cn-input-encode
- 语言：Rust
- 协议：[MIT 许可证](https://mit-license.org/)
- 作者：Garth TB | 天卜 <g-art-h@outlook.com>
- 版权：Copyright (c) 2026 Garth TB | 天卜
- 声明：本项目基于作者自用需求，追求极简高效而不承诺完备。开发者不对因使用本程序而造成的任何数据损失负责。

## 📝 版本

### 1.0.2 (20260520)

添加各平台的预编译包

### 1.0.0 (20260502)

首发

### 🕰️ 前身（废弃）：

- [ChnDPEncoder](https://github.com/GarthTB/ChnDPEncoder)
- [code_racer](https://github.com/GarthTB/code_racer)
