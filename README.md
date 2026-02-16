# My Terminal

> A simple terminal emulator written in Rust (MVP version) / 用 Rust 编写的简单终端模拟器（MVP 版本）

![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)
![License](https://img.shields.io/badge/License-MIT-blue.svg)

---

## 📋 Project Overview / 项目概述

This is a Minimum Viable Product (MVP) level terminal emulator for learning Rust systems programming and GUI development. The implementation is rough, but core functionality works.

这是一个最小可行产品（MVP）级别的终端模拟器，用于学习 Rust 系统编程和图形界面开发。虽然实现很粗糙，但核心功能是可用的。

### Current Features / 当前功能

- ✅ Window display (winit) / 窗口显示
- ✅ PTY integration (zsh shell) / PTY 集成
- ✅ Bidirectional communication / 双向通信
- ✅ ANSI escape sequence filtering / ANSI 转义序列过滤
- ✅ Font rendering (fontdue) / 字体渲染
- ✅ Event-driven redraw / 事件驱动重绘

### Known Limitations / 已知限制

- ⚠️ **No scroll buffer** - Only shows latest output / 只显示最新输出
- ⚠️ **No color support** - ANSI colors are filtered / ANSI 颜色被过滤
- ⚠️ **No cursor display** - Can't see current position / 看不到当前位置
- ⚠️ **Software rendering** - Poor performance / 性能较差
- ⚠️ **Fixed font** - Only Roboto 14px / 只有 Roboto 14px

---

## 🚀 Quick Start / 快速开始

### Requirements / 环境要求

- Rust 1.85+
- System: Linux (Wayland/X11)

### Installation / 安装运行

```bash
# Clone repository / 克隆仓库
git clone https://github.com/Clearzero22/my-terminal.git
cd my-terminal

# Run / 运行
cargo run

# Debug mode (view key logs) / 调试模式
RUST_LOG=debug cargo run

# Release build / 发布版本
cargo build --release
./target/release/my-terminal
```

### Controls / 操作说明

| Key / 按键 | Function / 功能 |
|------------|-----------------|
| `Escape` | Exit program / 退出程序 |
| Character keys | Input character / 输入字符 |
| `Enter` | Send carriage return / 发送回车 |
| `Backspace` | Backspace / 退格 |
| `Tab` | Tab character / 制表符 |

---

## 🏗️ Project Structure / 项目结构

```
my-terminal/
├── src/
│   ├── main.rs        # Main program and window management / 主程序和窗口管理
│   ├── pty.rs         # PTY session management / PTY 会话管理
│   ├── ansi.rs        # ANSI escape sequence filter / ANSI 转义序列过滤器
│   ├── buffer.rs      # Terminal buffer / 终端缓冲区
│   ├── font.rs        # Font renderer / 字体渲染器
│   ├── grid.rs        # Grid structure (unused) / 网格结构（未使用）
│   └── fonts/
│       └── Roboto-Regular.ttf  # Embedded font / 嵌入字体
├── Cargo.toml
├── WORKFLOW.md        # Detailed code workflow / 详细的代码流程文档
└── README.md          # This file / 本文件
```

---

## 🔧 Tech Stack / 技术栈

| Component / 组件 | Library / 库 | Version / 版本 | Purpose / 用途 |
|-------------------|--------------|----------------|----------------|
| Window Management | [winit](https://github.com/rust-window-team/winit) | 0.30.12 | Cross-platform window / 跨平台窗口 |
| Software Rendering | [softbuffer](https://github.com/rust-window-team/softbuffer) | 0.4.8 | Wayland-compatible rendering |
| PTY Integration | [portable-pty](https://github.com/wez/wezterm) | 0.9.0 | Pseudo-terminal / 伪终端 |
| Font Rendering | [fontdue](https://github.com/mooman219/fontdue) | 0.9.0 | Font rasterization / 字体光栅化 |
| Logging | env_logger / log | 0.11.9 / 0.4.29 | Logging / 日志记录 |

---

## 📊 Code Statistics / 代码统计

```
Language    Files    Lines    Code    Comments    Blanks
Rust           6      1085      940         45       100
```

---

## 🔄 How It Works / 工作原理

```
User Input → winit Event → PtySession::write()
                                    ↓
                           PTY Master → Shell
                                    ↓
PTY Output → Reader Thread → AnsiFilter → Buffer
                                    ↓
                         EventLoopProxy → RedrawRequested
                                    ↓
                         FontRenderer → Softbuffer → Window Display
```

---

## 📚 Documentation / 文档

See [WORKFLOW.md](WORKFLOW.md) for:
查看 [WORKFLOW.md](WORKFLOW.md) 了解：

- System architecture diagram / 系统架构图
- Module dependencies / 模块依赖关系
- Complete code flow / 完整的代码流程
- Thread model / 线程模型
- Sequence diagrams / 时序图
- Performance analysis / 性能分析

---

## 🐛 Known Issues / 已知问题

1. **Duplicate input** - Fixed (filter Release events) / 已修复
2. **ANSI color loss** - Feature limitation / 特性限制
3. **Window resize display issues** - Not handled / 未处理
4. **Long output overwrites content** - No scroll buffer / 无滚动缓冲

---

## 🚧 TODO / 待改进功能

### High Priority / 高优先级
- [ ] Scroll buffer (save history) / 滚动缓冲区
- [ ] Cursor display and follow / 光标显示和跟随
- [ ] Window size sync to PTY / 窗口大小同步

### Medium Priority / 中优先级
- [ ] ANSI color support / ANSI 颜色支持
- [ ] Copy and paste / 复制粘贴
- [ ] Multiple tabs / 多标签页

### Low Priority / 低优先级
- [ ] Configuration file / 配置文件
- [ ] Theme switching / 主题切换
- [ ] Custom key bindings / 快捷键绑定

---

## 📝 Development History / 开发历程

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

## 📄 License / 许可证

MIT License - see [LICENSE](LICENSE) file

---

## 🙏 Acknowledgments / 致谢

This project is based on the following open-source projects:
本项目基于以下开源项目：

- [winit](https://github.com/rust-window-team/winit) - Window management / 窗口管理
- [softbuffer](https://github.com/rust-window-team/softbuffer) - Software rendering / 软件渲染
- [portable-pty](https://github.com/wez/wezterm) - PTY implementation / PTY 实现
- [fontdue](https://github.com/mooman219/fontdue) - Font rendering / 字体渲染

---

## 📮 Contact / 联系方式

- GitHub: [@Clearzero22](https://github.com/Clearzero22)

---

**Note**: This is a learning project with rough code quality. Not recommended for production use!
**注意**: 这是一个学习项目，代码质量不高，不建议在生产环境使用！
