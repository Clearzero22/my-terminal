# My Terminal 项目总结

## 项目概述

创建一个使用 Rust 和 winit 的简单终端窗口应用程序，实现基本的事件循环和窗口管理功能。

## 项目要求

### 任务清单
- [x] 引入 winit 依赖（窗口处理库）
- [x] 引入 env_logger 依赖（日志记录）
- [x] 创建 Event Loop
- [x] 创建并显示窗口
- [x] 监听键盘输入事件
- [x] 按 Esc 键退出程序

### 通关标准
- [x] 运行 `cargo run` 能弹出窗口
- [x] 窗口正常显示
- [x] 按 Esc 键能够关闭窗口

## 技术栈

| 依赖库 | 版本 | 用途 |
|--------|------|------|
| winit | 0.30.12 | 跨平台窗口创建和事件处理 |
| env_logger | 0.11.9 | 日志记录实现 |
| log | 0.4.29 | 日志门面（Facade） |
| softbuffer | 0.4.8 | 软件渲染缓冲区，用于绘制窗口内容 |

## 关键问题与解决方案

### 问题 1：窗口创建后不显示

**现象**：
- 程序运行成功
- 日志显示 "Window created successfully"
- 但屏幕上看不到窗口

**根本原因**：
根据 winit 官方文档：
> "Launching a window **without drawing to it** has unpredictable results varying from platform to platform"

在 **Wayland** 显示服务器上，如果不向窗口绘制任何内容，窗口可能不会显示。

**解决方案**：
1. 添加 `softbuffer` 依赖用于绘制窗口内容
2. 在 `WindowEvent::RedrawRequested` 事件中填充窗口缓冲区
3. 窗口创建后立即请求重绘

### 问题 2：API 使用错误

**错误 1**：使用了已弃用的 API
```rust
// ❌ 错误：已弃用的 API
event_loop.run(move |event, elwt| { ... })

// ✅ 正确：使用新的 ApplicationHandler trait
event_loop.run_app(&mut app)
```

**错误 2**：生命周期管理问题
```rust
// ❌ 错误：Surface 的生命周期问题
struct Application {
    window: Option<Window>,
    surface: Option<Surface<Window, Window>>,
}

// ✅ 正确：使用 Rc 共享所有权
struct Application {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
}
```

**错误 3**：使用了错误的回调方法
```rust
// ❌ 错误：can_create_surfaces 在当前版本不可用
fn can_create_surfaces(&mut self, event_loop: &ActiveEventLoop)

// ✅ 正确：使用 resumed
fn resumed(&mut self, event_loop: &ActiveEventLoop)
```

## 最终实现代码

### 项目结构
```
my-terminal/
├── Cargo.toml
└── src/
    └── main.rs
```

### 核心实现要点

1. **使用 ApplicationHandler trait**：winit 0.30 推荐的现代方式
2. **在 resumed() 中创建窗口**：正确的生命周期钩子
3. **使用 softbuffer 绘制内容**：确保窗口在 Wayland 上可见
4. **处理 RedrawRequested 事件**：响应重绘请求

### 关键代码片段

```rust
// 1. 创建窗口和绘制表面
fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window = event_loop.create_window(...)?;
    let window = Rc::new(window);

    let context = Context::new(window.clone())?;
    let surface = Surface::new(&context, window.clone())?;

    self.window = Some(window);
    self.surface = Some(surface);

    // 请求初始重绘
    self.window.as_ref().unwrap().request_redraw();
}

// 2. 处理重绘事件
WindowEvent::RedrawRequested => {
    let size = window.inner_size();
    surface.resize(width, height)?;
    let mut buffer = surface.buffer_mut()?;
    buffer.fill(0xff181818); // 填充深灰色
    buffer.present()?;
}
```

## 运行方式

### 基本运行
```bash
cargo run
```

### 带日志运行
```bash
RUST_LOG=info cargo run
```

### 调试模式
```bash
RUST_LOG=debug cargo run
```

## 使用方式

1. **启动程序**：运行 `cargo run`
2. **查看窗口**：一个 800x600 的深灰色窗口会显示
3. **退出程序**：
   - 按 `Esc` 键
   - 或点击窗口的关闭按钮

## 环境信息

- **操作系统**：Linux (Manjaro)
- **显示服务器**：Wayland
- **Rust 版本**：1.93.0
- **Edition**：2024

## 学到的经验

1. **Wayland 平台特殊性**：不绘制内容的窗口可能不会显示
2. **API 版本差异**：winit 0.30 有重大 API 变更，需要使用 `ApplicationHandler`
3. **生命周期管理**：使用 `Rc` 来处理 softbuffer 和 window 之间的共享所有权
4. **事件驱动架构**：理解 `RedrawRequested` 事件的重要性

## 参考资料

- [winit 官方文档](https://docs.rs/winit/)
- [winit GitHub 仓库示例](https://github.com/rust-windowing/winit/tree/master/examples)
- [softbuffer 文档](https://docs.rs/softbuffer/)

## 下一步扩展

- [ ] 添加文本渲染功能
- [ ] 实现终端仿真器逻辑
- [ ] 支持 ANSI 颜色代码
- [ ] 添加窗口大小调整的响应处理
- [ ] 实现复制粘贴功能

---

**项目完成时间**：2026-02-16
**状态**：✅ 完成
