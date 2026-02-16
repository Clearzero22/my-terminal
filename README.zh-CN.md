# My Terminal

> 用 Rust 编写的简单终端模拟器（MVP 版本）

![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)
![License](https://img.shields.io/badge/License-MIT-blue.svg)

---

## 📋 项目概述

这是一个最小可行产品（MVP）级别的终端模拟器，用于学习 Rust 系统编程和图形界面开发。虽然实现很粗糙，但核心功能是可用的。

### 当前功能

- ✅ 窗口显示（winit）
- ✅ PTY 集成（zsh shell）
- ✅ 双向通信
- ✅ ANSI 转义序列过滤
- ✅ 字体渲染（fontdue）
- ✅ 事件驱动重绘

### 已知限制

- ⚠️ **无滚动缓冲区** - 只显示最新输出
- ⚠️ **无颜色支持** - ANSI 颜色被过滤
- ⚠️ **无光标显示** - 看不到当前位置
- ⚠️ **软件渲染** - 性能较差
- ⚠️ **固定字体** - 只有 Roboto 14px

---

## 🚀 快速开始

### 环境要求

- Rust 1.85+
- 系统：Linux（Wayland/X11）

### 安装运行

```bash
# 克隆仓库
git clone https://github.com/Clearzero22/my-terminal.git
cd my-terminal

# 运行
cargo run

# 调试模式（查看按键日志）
RUST_LOG=debug cargo run

# 发布版本
cargo build --release
./target/release/my-terminal
```

### 操作说明

| 按键 | 功能 |
|------|------|
| `Escape` | 退出程序 |
| 字符键 | 输入字符 |
| `Enter` | 发送回车 |
| `Backspace` | 退格 |
| `Tab` | 制表符 |

---

## 🏗️ 项目结构

```
my-terminal/
├── src/
│   ├── main.rs        # 主程序和窗口管理
│   ├── pty.rs         # PTY 会话管理
│   ├── ansi.rs        # ANSI 转义序列过滤器
│   ├── buffer.rs      # 终端缓冲区
│   ├── font.rs        # 字体渲染器
│   ├── grid.rs        # 网格结构（未使用）
│   └── fonts/
│       └── Roboto-Regular.ttf  # 嵌入字体
├── Cargo.toml
├── WORKFLOW.md        # 详细的代码流程文档
└── README.md          # 英文版文档
```

---

## 🔧 技术栈

| 组件 | 库 | 版本 | 用途 |
|------|------|------|------|
| 窗口管理 | [winit](https://github.com/rust-window-team/winit) | 0.30.12 | 跨平台窗口 |
| 软件渲染 | [softbuffer](https://github.com/rust-window-team/softbuffer) | 0.4.8 | Wayland 兼容渲染 |
| PTY 集成 | [portable-pty](https://github.com/wez/wezterm) | 0.9.0 | 伪终端 |
| 字体渲染 | [fontdue](https://github.com/mooman219/fontdue) | 0.9.0 | 字体光栅化 |
| 日志 | env_logger / log | 0.11.9 / 0.4.29 | 日志记录 |

---

## 📊 代码统计

```
语言        文件    行数    代码    注释    空行
Rust           6      1085      940         45       100
```

---

## 🔄 工作原理

```
用户按键 → winit 事件 → PtySession::write()
                                    ↓
                           PTY Master → Shell
                                    ↓
PTY 输出 → Reader 线程 → AnsiFilter → Buffer
                                    ↓
                         EventLoopProxy → RedrawRequested
                                    ↓
                         FontRenderer → Softbuffer → 窗口显示
```

---

## 📚 文档

查看 [WORKFLOW.md](WORKFLOW.md) 了解：
- 系统架构图
- 模块依赖关系
- 完整的代码流程
- 线程模型
- 时序图
- 性能分析

---

## 🐛 已知问题

1. **输入重复** - 已修复（过滤 Release 事件）
2. **ANSI 颜色丢失** - 特性限制
3. **窗口大小调整后显示异常** - 未处理
4. **长输出会覆盖之前内容** - 无滚动缓冲

---

## 🚧 待改进功能

### 高优先级
- [ ] 滚动缓冲区（保存历史输出）
- [ ] 光标显示和跟随
- [ ] 窗口大小同步到 PTY

### 中优先级
- [ ] ANSI 颜色支持
- [ ] 复制粘贴
- [ ] 多标签页

### 低优先级
- [ ] 配置文件
- [ ] 主题切换
- [ ] 快捷键绑定

---

## 📝 开发历程

```
00abe70 fix: prevent duplicate keyboard input
e17066e feat: add font rendering with fontdue
e637c3f feat: add simplified rendering with ANSI filtering
2b2af60 feat: add keyboard input handling for PTY
c6b5660 feat: add basic grid structure (simplified v0.1)
1bfc592 feat: add PTY integration with zsh shell
b12f450 feat: initial winit terminal window implementation
```

---

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

---

## 🙏 致谢

本项目基于以下开源项目：

- [winit](https://github.com/rust-window-team/winit) - 窗口管理
- [softbuffer](https://github.com/rust-window-team/softbuffer) - 软件渲染
- [portable-pty](https://github.com/wez/wezterm) - PTY 实现
- [fontdue](https://github.com/mooman219/fontdue) - 字体渲染

---

## 📮 联系方式

- GitHub: [@Clearzero22](https://github.com/Clearzero22)

---

**注意**: 这是一个学习项目，代码质量不高，不建议在生产环境使用！

---

**[English README](README.md)**
