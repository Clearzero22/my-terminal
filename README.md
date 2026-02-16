# My Terminal

> A simple terminal emulator written in Rust (MVP version)

![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)
![License](https://img.shields.io/badge/License-MIT-blue.svg)

---

## 📋 Project Overview

This is a Minimum Viable Product (MVP) level terminal emulator for learning Rust systems programming and GUI development. The implementation is rough, but core functionality works.

### Current Features

- ✅ Window display (winit)
- ✅ PTY integration (zsh shell)
- ✅ Bidirectional communication
- ✅ ANSI escape sequence filtering
- ✅ Font rendering (fontdue)
- ✅ Event-driven redraw

### Known Limitations

- ⚠️ **No scroll buffer** - Only shows latest output
- ⚠️ **No color support** - ANSI colors are filtered
- ⚠️ **No cursor display** - Can't see current position
- ⚠️ **Software rendering** - Poor performance
- ⚠️ **Fixed font** - Only Roboto 14px

---

## 🚀 Quick Start

### Requirements

- Rust 1.85+
- System: Linux (Wayland/X11)

### Installation

```bash
# Clone repository
git clone https://github.com/Clearzero22/my-terminal.git
cd my-terminal

# Run
cargo run

# Debug mode (view key logs)
RUST_LOG=debug cargo run

# Release build
cargo build --release
./target/release/my-terminal
```

### Controls

| Key | Function |
|-----|----------|
| `Escape` | Exit program |
| Character keys | Input character |
| `Enter` | Send carriage return |
| `Backspace` | Backspace |
| `Tab` | Tab character |

---

## 🏗️ Project Structure

```
my-terminal/
├── src/
│   ├── main.rs        # Main program and window management
│   ├── pty.rs         # PTY session management
│   ├── ansi.rs        # ANSI escape sequence filter
│   ├── buffer.rs      # Terminal buffer
│   ├── font.rs        # Font renderer
│   ├── grid.rs        # Grid structure (unused)
│   └── fonts/
│       └── Roboto-Regular.ttf  # Embedded font
├── Cargo.toml
├── WORKFLOW.md        # Detailed code workflow documentation
└── README.md          # This file
```

---

## 🔧 Tech Stack

| Component | Library | Version | Purpose |
|-----------|---------|---------|---------|
| Window Management | [winit](https://github.com/rust-window-team/winit) | 0.30.12 | Cross-platform window |
| Software Rendering | [softbuffer](https://github.com/rust-window-team/softbuffer) | 0.4.8 | Wayland-compatible rendering |
| PTY Integration | [portable-pty](https://github.com/wez/wezterm) | 0.9.0 | Pseudo-terminal |
| Font Rendering | [fontdue](https://github.com/mooman219/fontdue) | 0.9.0 | Font rasterization |
| Logging | env_logger / log | 0.11.9 / 0.4.29 | Logging |

---

## 📊 Code Statistics

```
Language    Files    Lines    Code    Comments    Blanks
Rust           6      1085      940         45       100
```

---

## 🔄 How It Works

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

## 📚 Documentation

See [WORKFLOW.md](WORKFLOW.md) for:
- System architecture diagram
- Module dependencies
- Complete code flow
- Thread model
- Sequence diagrams
- Performance analysis

---

## 🐛 Known Issues

1. **Duplicate input** - Fixed (filter Release events)
2. **ANSI color loss** - Feature limitation
3. **Window resize display issues** - Not handled
4. **Long output overwrites content** - No scroll buffer

---

## 🚧 TODO

### High Priority
- [ ] Scroll buffer (save history)
- [ ] Cursor display and follow
- [ ] Window size sync to PTY

### Medium Priority
- [ ] ANSI color support
- [ ] Copy and paste
- [ ] Multiple tabs

### Low Priority
- [ ] Configuration file
- [ ] Theme switching
- [ ] Custom key bindings

---

## 📝 Development History

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

## 📄 License

MIT License - see [LICENSE](LICENSE) file

---

## 🙏 Acknowledgments

This project is based on the following open-source projects:

- [winit](https://github.com/rust-window-team/winit) - Window management
- [softbuffer](https://github.com/rust-window-team/softbuffer) - Software rendering
- [portable-pty](https://github.com/wez/wezterm) - PTY implementation
- [fontdue](https://github.com/mooman219/fontdue) - Font rendering

---

## 📮 Contact

- GitHub: [@Clearzero22](https://github.com/Clearzero22)

---

**Note**: This is a learning project with rough code quality. Not recommended for production use!

---

**[中文版 README](README.zh-CN.md)**
