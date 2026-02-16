# My Terminal 代码运行流程文档

## 目录
1. [系统架构](#系统架构)
2. [模块依赖关系](#模块依赖关系)
3. [程序启动流程](#程序启动流程)
4. [用户输入流程](#用户输入流程)
5. [PTY 输出流程](#pty-输出流程)
6. [渲染流程](#渲染流程)
7. [线程模型](#线程模型)
8. [数据流向图](#数据流向图)

---

## 系统架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         Application                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────┐│
│  │  Window  │ │  Context │ │  Surface │ │   Font   │ │ Proxy ││
│  │  (winit) │ │(softbuf) │ │(softbuf) │ │(fontdue) │ │(winit)││
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └───┬───┘│
│       │            │            │            │           │    │
│       └────────────┴────────────┴────────────┴───────────┘    │
├─────────────────────────────────────────────────────────────────┤
│                     Terminal State                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │   PTY    │ │  Buffer  │ │  Filter  │ │   Grid   │          │
│  │ Session  │ │          │ │ (ANSI)   │ │ (future) │          │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └──────────┘          │
│       │            │            │                              │
└───────┼────────────┼────────────┼──────────────────────────────┘
        │            │            │
        ▼            ▼            ▼
    ┌────────┐  ┌────────┐  ┌────────┐
    │  zsh   │  │ String │  │  Text  │
    │ Shell  │  │ Buffer │  │  Lines │
    └────────┘  └────────┘  └────────┘
```

---

## 模块依赖关系

```
┌────────────────────────────────────────────────────────────────┐
│                          main.rs                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │   winit      │  │  softbuffer  │  │  std         │        │
│  │ - Window     │  │ - Context    │  │ - sync       │        │
│  │ - EventLoop  │  │ - Surface    │  │ - rc         │        │
│  └──────────────┘  └──────────────┘  └──────────────┘        │
│         │                  │                  │               │
│         ├──────────────────┼──────────────────┤               │
│         ▼                  ▼                  ▼               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │   pty.rs     │  │  buffer.rs   │  │   font.rs    │        │
│  │              │  │              │  │              │        │
│  │ - PtySession │  │ - TerminalBuf│  │ - FontRender │        │
│  │ - PTY I/O    │  │ - Arc<Mutex> │  │ - Rasterize  │        │
│  └──────┬───────┘  └──────┬───────┘  └──────────────┘        │
│         │                  │                                  │
│         ▼                  ▼                                  │
│  ┌──────────────┐  ┌──────────────┐                          │
│  │  ansi.rs     │  │ portable-pty │                          │
│  │              │  │ fontdue      │                          │
│  │ - AnsiFilter │  │              │                          │
│  │ - State Mach │  │              │                          │
│  └──────────────┘  └──────────────┘                          │
└────────────────────────────────────────────────────────────────┘
```

---

## 程序启动流程

```
main()
    │
    ├─→ env_logger::init()                    // 初始化日志
    │
    ├─→ EventLoop::with_user_event()          // 创建事件循环
    │   │
    │   └─→ create_proxy()                    // 创建事件代理
    │
    ├─→ Application::new()                     // 创建应用状态
    │   │
    │   └─→ 所有字段初始化为 None
    │
    └─→ event_loop.run_app(&mut app)          // 运行事件循环
            │
            ├─→ [resumed 回调]
            │   │
            │   ├─→ WindowAttributes::default()
            │   │       .with_title("My Terminal")
            │   │       .with_inner_size(800, 600)
            │   │
            │   ├─→ event_loop.create_window() // 创建窗口
            │   │   │
            │   │   ├─→ Rc::new(window)
            │   │   │
            │   │   ├─→ Context::new()          // 创建 softbuffer 上下文
            │   │   │
            │   │   ├─→ Surface::new()          // 创建 softbuffer 表面
            │   │   │
            │   │   └─→ self.window = Some(window)
            │   │       self.context = Some(context)
            │   │       self.surface = Some(surface)
            │   │
            │   ├─→ [初始化字体渲染器]
            │   │   │
            │   │   └─→ FontRenderer::with_size(14.0)
            │   │       │
            │   │       └─→ include_bytes!("fonts/Roboto-Regular.ttf")
            │   │           └─→ Font::from_bytes()
            │   │
            │   ├─→ [初始化终端缓冲区]
            │   │   │
            │   │   ├─→ TerminalBuffer::new()
            │   │   │   │
            │   │   │   ├─→ content: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()))
            │   │   │   │
            │   │   │   └─→ filter: Arc<Mutex<AnsiFilter>> = Arc::new(Mutex::new(AnsiFilter::new()))
            │   │   │
            │   │   └─→ buffer.clone()          // 为回调创建克隆
            │   │
            │   ├─→ [初始化 PTY 会话]
            │   │   │
            │   │   ├─→ 创建回调函数
            │   │   │   │
            │   │   │   └─→ |data: &[u8]| {
            │   │   │           buffer_clone.write(data);     // 写入缓冲区
            │   │   │           proxy.send_event(NewOutput);  // 触发重绘
            │   │   │       }
            │   │   │
            │   │   └─→ PtySession::with_output_callback(callback)
            │   │       │
            │   │       ├─→ native_pty_system()              // 获取平台 PTY 系统
            │   │       │
            │   │       ├─→ pty_system.openpty(size)        // 创建 PTY 对
            │   │       │   │
            │   │       │   └─→ master: reader + writer
            │   │       │       slave: 用于 spawn shell
            │   │       │
            │   │       ├─→ CommandBuilder::new("zsh")
            │   │       │   .env("TERM", "xterm-256color")
            │   │       │
            │   │       ├─→ pty_pair.slave.spawn_command(cmd) // 启动 zsh
            │   │       │
            │   │       ├─→ [启动 Reader 线程]
            │   │       │   │
            │   │       │   └─→ thread::spawn(move || {
            │   │       │           loop {
            │   │       │               reader.read(&mut buffer)
            │   │       │               │
            │   │       │               ├─→ Ok(0) → break      // EOF
            │   │       │               ├─→ Ok(n) → callback(data)  // 调用回调
            │   │       │               └─→ Err(e) → break
            │   │       │           }
            │   │       │       })
            │   │       │
            │   │       └─→ self.pty = Some(pty)
            │   │
            │   └─→ window.request_redraw()         // 请求初始重绘
            │
            └─→ [事件循环]
                │
                ├─→ [用户事件] user_event()
                │   │
                │   └─→ AppEvent::NewOutput
                │       └─→ window.request_redraw()  // 触发重绘
                │
                ├─→ [窗口事件] window_event()
                │   │
                │   ├─→ CloseRequested → event_loop.exit()
                │   │
                │   ├─→ KeyboardInput
                │   │   │
                │   │   ├─→ [状态过滤]
                │   │   │   │
                │   │   │   └─→ if state != Pressed || repeat → return
                │   │   │
                │   │   ├─→ Escape → event_loop.exit()
                │   │   │
                │   │   └─→ 其他键 → pty.write()
                │   │       │
                │   │       ├─→ Enter → b'\r'
                │   │       ├─→ Backspace → 0x08
                │   │       ├─→ Tab → b'\t'
                │   │       └─→ Character → write_str(c)
                │   │
                │   ├─→ Resized → window.request_redraw()
                │   │
                │   └─→ RedrawRequested
                │       │
                │       └─→ [渲染流程]
                │           │
                │           ├─→ surface.resize(width, height)
                │           │
                │           ├─→ buffer.fill(0xff181818)  // 深灰背景
                │           │
                │           ├─→ buffer.content()         // 获取缓冲区内容
                │           │   │
                │           │   └─→ String (已过滤 ANSI)
                │           │
                │           ├─→ font::split_lines()      // 分割为行
                │           │   │
                │           │   └─→ Vec<String>
                │           │
                │           ├─→ font.render_lines()      // 渲染文本
                │           │   │
                │           │   └─→ [逐字符渲染]
                │           │       │
                │           │       ├─→ font.rasterize(c) → (metrics, bitmap)
                │           │       │
                │           │       ├─→ [Alpha 混合]
                │           │       │   │
                │           │       │   └─→ new_pixel = src * alpha + dst * (1-alpha)
                │           │       │
                │           │       └─→ buffer[idx] = new_pixel
                │           │
                │           └─→ surface.present()
                │
                └─→ [应用退出]
                    │
                    └─→ Application::drop()
                        │
                        └─→ PtySession::drop()
                            │
                            └─→ [等待 Reader 线程结束]
                                │
                                └─→ timeout 2 秒后继续
```

---

## 用户输入流程

```
用户按下键盘
    │
    ├─→ 硬件产生键盘事件
    │
    ├─→ 操作系统捕获事件
    │
    ├─→ winit 接收事件
    │
    └─→ WindowEvent::KeyboardInput {
            event: KeyEvent {
                logical_key,     // 按键逻辑值
                state,           // Pressed/Released
                repeat,          // 是否重复
                ...
            }
        }
        │
        ├─→ [状态过滤] main.rs:126
        │   │
        │   ├─→ state != ElementState::Pressed → return
        │   │   └─→ 忽略释放事件
        │   │
        │   └─→ repeat == true → return
        │       └─→ 忽略长按重复
        │
        ├─→ [按键匹配] main.rs:136
        │   │
        │   ├─→ Escape → event_loop.exit()
        │   │
        │   ├─→ Enter → pty.write(b'\r')
        │   │
        │   ├─→ Backspace → pty.write(0x08)
        │   │
        │   ├─→ Tab → pty.write(b'\t')
        │   │
        │   └─→ Character(c) → pty.write_str(c)
        │       │
        │       └─→ [写入 PTY] pty.rs:158
        │           │
        │           ├─→ writer.lock()          // 获取 Mutex 锁
        │           │
        │           ├─→ writer.write_all(bytes)
        │           │
        │           └─→ writer.flush()
        │
        └─→ [PTY 处理]
            │
            ├─→ PTY master 接收输入
            │
            ├─→ 传递给 slave (shell 进程)
            │
            └─→ shell 处理命令
                │
                ├─→ 解释命令
                │
                ├─→ 执行操作
                │
                └─→ 生成输出 → 发送到 PTY master
```

---

## PTY 输出流程

```
Shell 产生输出
    │
    ├─→ 写入 PTY slave
    │
    ├─→ PTY master 接收数据
    │
    └─→ [Reader 线程] pty.rs:108
        │
        ├─→ reader.lock()                // 获取 Mutex 锁
        │
        ├─→ reader.read(&mut buffer)     // 阻塞读取
        │
        ├─→ [数据到达]
        │   │
        │   ├─→ Ok(0)                    // EOF
        │   │   └─→ break  // 退出循环
        │   │
        │   ├─→ Ok(n)                    // 读取到 n 字节
        │   │   │
        │   │   ├─→ drop(reader_guard)   // 释放锁
        │   │   │
        │   │   └─→ [调用回调] main.rs:92
        │   │       │
        │   │       ├─→ callback.lock()
        │   │       │
        │   │       ├─→ callback(&data)  // 执行回调
        │   │       │   │
        │   │       │   └─→ [写入缓冲区] buffer.rs:29
        │   │       │       │
        │   │       │       ├─→ filter.lock()
        │   │       │       │
        │   │       │       ├─→ [ANSI 过滤] ansi.rs:58
        │   │       │       │   │
        │   │       │       │   ├─→ process(byte)
        │   │       │       │   │   │
        │   │       │       │   │   ├─→ state: Normal
        │   │       │       │   │   │   ├─→ 0x1b → state = Escape
        │   │       │       │   │   │   ├─→ 0x07 (BEL) → None
        │   │       │       │   │   │   ├─→ 0x08 (BS) → Some('\x08')
        │   │       │       │   │   │   ├─→ 0x09 (HT) → Some('\t')
        │   │       │       │   │   │   ├─→ 0x0a-0x0d → Some(char)
        │   │       │       │   │   │   ├─→ 0x20-0x7e → Some(char)
        │   │       │       │   │   │   └─→ 其他 → None
        │   │       │       │   │   │
        │   │       │       │   │   ├─→ state: Escape
        │   │       │       │   │   │   ├─→ b'[' → state = Csi
        │   │       │       │   │   │   └─→ 其他 → state = Normal
        │   │       │       │   │   │
        │   │       │       │   │   └─→ state: Csi
        │   │       │       │   │       ├─→ 0x40-0x7e (final) → state = Normal
        │   │       │       │   │       └─→ 其他 → 保持 Csi
        │   │       │       │   │
        │   │       │       │   └─→ Some(char) → 累积到 String
        │   │       │       │
        │   │       │       └─→ content.lock().push_str(&filtered)
        │   │       │
        │   │       └─→ proxy.send_event(AppEvent::NewOutput)
        │   │           │
        │   │           └─→ [触发用户事件]
        │   │
        │   └─→ 继续循环
        │
        └─→ Err(e)                       // 读取错误
            └─→ break
```

---

## 渲染流程

```
WindowEvent::RedrawRequested
    │
    ├─→ [获取渲染资源] main.rs:170
    │   │
    │   ├─→ self.window
    │   ├─→ self.surface
    │   ├─→ self.buffer
    │   └─→ self.font
    │
    ├─→ [获取窗口尺寸]
    │   │
    │   ├─→ window.inner_size()
    │   │
    │   └─→ (width, height) : NonZeroU32
    │
    ├─→ [调整 Surface 大小] main.rs:180
    │   │
    │   └─→ surface.resize(width, height)
    │
    ├─→ [获取像素缓冲区]
    │   │
    │   └─→ surface.buffer_mut()
    │       │
    │       └─→ &mut [u32]  // ARGB 格式
    │
    ├─→ [填充背景] main.rs:185
    │   │
    │   └─→ buffer.fill(0xff181818)  // 深灰色
    │
    ├─→ [获取缓冲区内容]
    │   │
    │   ├─→ buffer.content()  // String
    │   │
    │   └─→ [分割为行] font.rs:230
    │       │
    │       ├─→ split_lines(&content)
    │       │
    │       └─→ Vec<String>
    │           │
    │           ├─→ 遍历字符
    │           │   ├─→ '\n' → 新行
    │           │   ├─→ '\r' → 新行
    │           │   └─→ 其他 → 累积
    │
    ├─→ [计算可见行数]
    │   │
    │   ├─→ char_height = font.char_height()  // 14px
    │   │
    │   └─→ visible_lines = height / char_height
    │
    ├─→ [渲染文本] font.rs:210
    │   │
    │   └─→ font.render_lines()
    │       │
    │       ├─→ 遍历每一行
    │       │
    │       └─→ [渲染单行] font.rs:168
    │           │
    │           └─→ render_text()
    │               │
    │               ├─→ 遍历每个字符
    │               │
    │               ├─→ [渲染单字符] font.rs:93
    │               │   │
    │               │   └─→ render_char(c, x, y, buffer, ...)
    │               │       │
    │               │       ├─→ font.rasterize(c, font_size)
    │               │       │   │
    │               │       │   └─→ (metrics, bitmap: Vec<u8>)
    │               │       │       │
    │               │       │       └─→ bitmap: alpha 值数组
    │               │       │
    │               │       ├─→ [提取颜色分量]
    │               │       │   │
    │               │       │   ├─→ text_color = 0xff00ff00 (ARGB)
    │               │       │   │
    │               │       │   ├─→ a = (color >> 24) & 0xFF
    │               │       │   ├─→ r = (color >> 16) & 0xFF
    │               │       │   ├─→ g = (color >> 8) & 0xFF
    │               │       │   └─→ b = color & 0xFF
    │               │       │
    │               │       ├─→ [遍历像素]
    │               │       │   │
    │               │       │   └─→ for (i, &alpha) in bitmap.iter()
    │               │       │       │
    │               │       │       ├─→ glyph_x = x + (i % metrics.width)
    │               │       │       ├─→ glyph_y = y + (i / metrics.width)
    │               │       │       │
    │               │       │       ├─→ [边界检查]
    │               │       │       │   │
    │               │       │       │   └─→ if glyph_x >= width || glyph_y >= height → continue
    │               │       │       │
    │               │       │       ├─→ idx = glyph_y * width + glyph_x
    │               │       │       │
    │               │       │       ├─→ existing = buffer[idx]
    │               │       │       │
    │               │       │       └─→ [Alpha 混合]
    │               │       │           │
    │               │       │           ├─→ inv_alpha = 255 - alpha
    │               │       │           │
    │               │       │           ├─→ new_a = (a * alpha + existing_a * inv_alpha) / 255
    │               │       │           ├─→ new_r = (r * alpha + existing_r * inv_alpha) / 255
    │               │       │           ├─→ new_g = (g * alpha + existing_g * inv_alpha) / 255
    │               │       │           └─→ new_b = (b * alpha + existing_b * inv_alpha) / 255
    │               │       │
    │               │       └─→ buffer[idx] = (new_a << 24) | (new_r << 16) | (new_g << 8) | new_b
    │               │
    │               └─→ x += char_width  // 移动到下一个字符位置
    │
    └─→ [显示缓冲区] main.rs:213
        │
        └─→ surface.present()
            │
            └─→ softbuffer 将像素数据发送到窗口系统
```

---

## 线程模型

```
┌─────────────────────────────────────────────────────────────────┐
│                        主线程 (Main Thread)                     │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │           winit Event Loop (阻塞)                        │  │
│  │                                                           │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │  │
│  │  │  resumed()  │  │window_event()│ │user_event() │      │  │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘      │  │
│  │         │                │                │              │  │
│  │         ▼                ▼                ▼              │  │
│  │  ┌─────────────────────────────────────────────────┐    │  │
│  │  │         渲染操作 (RedrawRequested)              │    │  │
│  │  │                                                   │    │  │
│  │  │  • 获取缓冲区内容 (跨线程共享)                   │    │  │
│  │  │  • 字体渲染 (CPU 光栅化)                          │    │  │
│  │  │  • 像素操作 (Alpha 混合)                         │    │  │
│  │  │  • Surface 显示                                   │    │  │
│  │  └─────────────────────────────────────────────────┘    │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │            用户输入处理 (KeyboardInput)                  │  │
│  │                                                           │  │
│  │  • 状态过滤 (Pressed/Released)                           │  │
│  │  • PTY 写入 (跨线程同步)                                 │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            │
                            │ Arc<Mutex<PtySession>>
                            │ 共享访问
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                   PTY Reader 线程 (后台线程)                     │
│                                                                 │
│  创建位置: pty.rs:108 (thread::spawn)                           │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                  无限循环 (loop)                         │  │
│  │                                                           │  │
│  │  ┌─────────────────────────────────────────────────┐    │  │
│  │  │         PTY 读取 (reader.read)                   │    │  │
│  │  │                                                   │    │  │
│  │  │  • 阻塞等待数据                                   │    │  │
│  │  │  • 获取 Mutex 锁                                  │    │  │
│  │  │  • 读取到缓冲区                                   │    │  │
│  │  │  • 释放锁                                         │    │  │
│  │  └─────────────────────────────────────────────────┘    │  │
│  │                          │                               │  │
│  │                          ▼                               │  │
│  │  ┌─────────────────────────────────────────────────┐    │  │
│  │  │            回调处理 (callback)                   │    │  │
│  │  │                                                   │    │  │
│  │  │  • 调用 AnsiFilter (状态机)                       │    │  │
│  │  │  • 写入 TerminalBuffer (Arc<Mutex<String>>)      │    │  │
│  │  │  • 发送 NewOutput 事件                            │    │  │
│  │  │  • 触发窗口重绘                                   │    │  │
│  │  └─────────────────────────────────────────────────┘    │  │
│  │                                                           │  │
│  │  退出条件:                                                │  │
│  │  • Ok(0) - EOF (shell 退出)                             │  │
│  │  • Err(e) - 读取错误                                     │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
│  清理: pty.rs:190 (Drop trait)                                │
│  • 等待线程结束 (timeout 2 秒)                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 数据流向图

### 完整数据流

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            用户输入流                                    │
└─────────────────────────────────────────────────────────────────────────┘

用户键盘
    │
    ▼
┌───────────────┐
│  winit 事件   │
│  KeyboardInput│
└───────┬───────┘
        │
        ▼ [状态过滤]
    ┌─────────┐
    │ Pressed │
    │ !repeat │
    └────┬────┘
         │
         ▼
    ┌──────────┐
    │ PtySession│
    │ writer   │
    └────┬─────┘
         │
         ▼
    ┌──────────┐
    │  PTY     │
    │  Master  │
    └────┬─────┘
         │
         ▼
    ┌──────────┐
    │  Shell   │
    │ (zsh)    │
    └──────────┘


┌─────────────────────────────────────────────────────────────────────────┐
│                            PTY 输出流                                    │
└─────────────────────────────────────────────────────────────────────────┘

Shell
    │
    ▼ 原始字节 (包含 ANSI 序列)
┌───────────────┐
│  PTY Master   │
└───────┬───────┘
        │
        ▼ [Reader 线程]
    ┌─────────┐
    │ reader  │
    │ .read() │
    └────┬────┘
         │
         ▼ [回调]
    ┌──────────┐
    │ AnsiFilter│
    │ 状态机过滤 │
    └────┬─────┘
         │
         ▼ 纯文本 (去除 ANSI)
    ┌──────────┐
    │ Terminal │
    │ Buffer   │
    │ Arc<Mutex│
    └────┬─────┘
         │
         ▼ [事件触发]
    ┌──────────┐
    │ EventLoop│
    │ Proxy    │
    └────┬─────┘
         │
         ▼
    ┌──────────┐
    │ RedrawReq│
    └────┬─────┘
         │
         ▼ [渲染]
    ┌──────────┐
    │ FontRend │
    │ 光栅化   │
    └────┬─────┘
         │
         ▼ 像素数据
    ┌──────────┐
    │ Surface  │
    └────┬─────┘
         │
         ▼
    ┌──────────┐
    │  Window  │
    └──────────┘
```

### 共享数据结构

```
┌─────────────────────────────────────────────────────────────────┐
│                     Arc<Mutex<T>> 共享数据                       │
└─────────────────────────────────────────────────────────────────┘

主线程                      PTY Reader 线程
    │                              │
    │    ┌──────────────────┐      │
    │    │ Arc<Mutex<        │      │
    │    │   PtySession >>   │      │
    │    │                   │      │
    ├────┤ writer: Arc<Mutex├──────┤
    │    │   <Box<Write>>>   │      │
    │    │                   │      │
    │    └──────────────────┘      │
    │         ▲                    │
    │         │ write()            │
    │         │                    │
    │    用户输入处理              │
    │                              │
    │    ┌──────────────────┐      │
    ├────┤ Arc<Mutex<        │      │
    │    │   TerminalBuffer  │      │
    │    │   >>              │      │
    │    │                   │      │
    │    │ content: Arc<Mutex├──────┤
    │    │   <String>>       │      │
    │    │                   │      │
    │    │ filter: Arc<Mutex│      │
    │    │   <AnsiFilter>>   │      │
    │    │                   │      │
    │    └──────────────────┘      │
    │         ▲                    │
    │         │ content()          │
    │         │                    │
    │    渲染读取                  │
    │                              │
    └──────────────────────────────┘
```

---

## 关键函数调用链

### 初始化链

```
main()
  └─→ EventLoop::with_user_event()
      └─→ Application::new()
          └─→ event_loop.run_app()
              └─→ resumed()
                  ├─→ FontRenderer::with_size()
                  │   └─→ Font::from_bytes()
                  │
                  ├─→ TerminalBuffer::new()
                  │   └─→ AnsiFilter::new()
                  │
                  └─→ PtySession::with_output_callback()
                      ├─→ native_pty_system()
                      ├─→ pty_system.openpty()
                      ├─→ CommandBuilder::new("zsh")
                      ├─→ pty_pair.slave.spawn_command()
                      └─→ thread::spawn() [Reader 线程]
```

### 键盘输入链

```
WindowEvent::KeyboardInput
  └─→ window_event()
      ├─→ [状态过滤] ElementState::Pressed && !repeat
      │
      ├─→ Escape
      │   └─→ event_loop.exit()
      │
      └─→ 其他键
          └─→ PtySession::write() / write_str()
              └─→ writer.lock()
                  └─→ writer.write_all()
                      └─→ writer.flush()
```

### PTY 输出链

```
[Reader 线程]
  └─→ loop
      └─→ reader.read()
          └─→ callback()
              ├─→ TerminalBuffer::write()
              │   └─→ AnsiFilter::process_slice()
              │       ├─→ process(byte)
              │       │   └─→ [状态机过滤]
              │       └─→ content.push_str()
              │
              └─→ proxy.send_event(NewOutput)
                  └─→ user_event()
                      └─→ window.request_redraw()
                          └─→ RedrawRequested
                              └─→ [渲染流程]
```

### 渲染链

```
RedrawRequested
  └─→ window_event()
      └─→ [渲染]
          ├─→ surface.buffer_mut()
          ├─→ buffer.fill()
          ├─→ buffer.content()
          │   └─→ content.lock()
          │       └─→ String::clone()
          │
          ├─→ font::split_lines()
          │   └─→ Vec<String>
          │
          └─→ font.render_lines()
              └─→ render_text()
                  └─→ render_char()
                      ├─→ font.rasterize()
                      │   └─→ (metrics, bitmap)
                      │
                      └─→ [Alpha 混合]
                          └─→ buffer[idx] = new_pixel

          └─→ surface.present()
```

---

## 时序图

### 程序启动时序

```
主线程                    winit              PTY Reader          Shell
  │                        │                    │                  │
  │──main()───────────────▶│                    │                  │
  │                        │                    │                  │
  │◀─EventLoop────────────│                    │                  │
  │                        │                    │                  │
  │──run_app(app)─────────▶│                    │                  │
  │                        │                    │                  │
  │◀─resumed()────────────│                    │                  │
  │                        │                    │                  │
  │──create_window()─────▶│                    │                  │
  │◀─Window───────────────│                    │                  │
  │                        │                    │                  │
  │──FontRenderer::new()──│                    │                  │
  │◀─Font─────────────────│                    │                  │
  │                        │                    │                  │
  │──TerminalBuffer::new()─│                    │                  │
  │◀─Buffer───────────────│                    │                  │
  │                        │                  │                  │
  │──PtySession::new()─────┼───────────────────▶│                  │
  │                        │                  │                  │
  │                        │──openpty()───────▶│                  │
  │                        │◀─pty_pair────────│                  │
  │                        │                  │                  │
  │                        │──spawn_command()─┼─────────────────▶│
  │                        │◀─child──────────│                  │
  │                        │                  │                  │
  │                        │──thread::spawn()─┼──[Reader Loop]──▶│
  │◀─PtySession────────────│                  │                  │
  │                        │                  │                  │
  │──request_redraw()─────▶│                    │                  │
```

### 用户输入时序

```
用户键盘              winit           主线程              PTY              Shell
  │                    │                │                  │                │
  │──按键─────────────▶│                │                  │                │
  │                    │                │                  │                │
  │                    │──KeyEvent─────▶│                  │                │
  │                    │                │                  │                │
  │                    │                │──[过滤状态]──────│                │
  │                    │                │                  │                │
  │                    │                │──write()─────────┼───────────────▶│
  │                    │                │                  │                │
  │                    │                │                  │──处理命令──────│
  │                    │                │                  │                │
```

### PTY 输出时序

```
Shell              PTY           Reader线程          主线程          窗口
  │                 │                │               │               │
  │──输出───────────▶│                │               │               │
  │                 │                │               │               │
  │                 │──数据可用─────▶│               │               │
  │                 │                │               │               │
  │                 │                │──read()───────│               │
  │                 │                │               │               │
  │                 │                │──callback()───┼──NewOutput───▶│
  │                 │                │               │               │
  │                 │                │               │──request_redraw()──▶
  │                 │                │               │               │
  │                 │                │               │◀─RedrawRequested│
  │                 │                │               │               │
  │                 │                │               │──渲染内容─────▶│
  │                 │                │               │               │
  │                 │                │               │               │──显示──│
```

---

## 状态转换图

### ANSI 过滤器状态机

```
┌─────────────────────────────────────────────────────────────────┐
│                     AnsiFilter 状态机                           │
└─────────────────────────────────────────────────────────────────┘

    ┌──────────┐
    │  Normal  │  ← 初始状态
    └────┬─────┘
         │
         │ 接收 0x1b (ESC)
         ▼
    ┌──────────┐
    │  Escape  │
    └────┬─────┘
         │
         ├───── b'[' ────▶ ┌──────────┐
         │                  │    Csi   │
         │                  └────┬─────┘
         │                       │
         │                       │ 接收 0x40-0x7e (final byte)
         │                       │
         │ ┌─────────────────────┘
         │ │
         │ │ 其他字符
         ▼ ▼
    ┌──────────┐
    │  Normal  │
    └──────────┘

状态说明:
• Normal: 正常输出模式，可打印字符直接输出
• Escape: 接收到 ESC，等待下一个字符判断序列类型
• Csi: CSI 序列（Control Sequence Introducer），读取参数和结束符
```

### 线程同步状态

```
┌─────────────────────────────────────────────────────────────────┐
│                      线程同步状态                               │
└─────────────────────────────────────────────────────────────────┘

主线程状态:
┌───────────────────────────────────────────────────────────────┐
│  Idle ────────────────▶ Processing ──────────────▶ Idle       │
│                            │                                   │
│                            ▼                                   │
│                       ┌───────────┐                            │
│                       │ 处理事件  │                            │
│                       │ • 用户输入│                            │
│                       │ • 重绘请求│                            │
│                       └───────────┘                            │
└───────────────────────────────────────────────────────────────┘

Reader 线程状态:
┌───────────────────────────────────────────────────────────────┐
│  Reading ────────────▶ Processing ────────────▶ Reading       │
│     │                     │                              │      │
│     │ 阻塞在 read()       │ 执行回调                      │      │
│     │                     │ • ANSI 过滤                   │      │
│     │                     │ • 写入缓冲区                  │      │
│     │                     │ • 触发重绘                    │      │
│     │                     │                              │      │
│     │                     │ ◀────────────────────────────│      │
│     │                     │                              │      │
│     ▼                     ▼                              ▼      │
│  [EOF/Error ─────────────────────────▶ Exited]                   │
└───────────────────────────────────────────────────────────────┘
```

---

## 性能考虑

### CPU 使用

```
主线程:
  • 事件驱动，大部分时间空闲
  • 重绘时 CPU 密集 (字体光栅化)
  • 每帧 ~10ms (800x600 窗口)

Reader 线程:
  • 阻塞在 read()，无输出时不占用 CPU
  • 有数据时快速处理 (< 1ms)
  • 回调执行时间极短

瓶颈:
  • 字体光栅化 (软件渲染)
  • Alpha 混合 (逐像素计算)
  • 内存拷贝 (Arc/Mutex 开销)
```

### 内存使用

```
固定分配:
  • 窗口缓冲区: 800x600x4 = ~1.9MB
  • 字体数据: ~500KB (Roboto-Regular.ttf)
  • PTY 缓冲区: 8KB (reader buffer)

动态分配:
  • TerminalBuffer: 随输出增长
  • 渲染缓存: 每次重绘临时分配

共享内存:
  • Arc<Mutex<String>>: 内容缓冲区
  • Arc<Mutex<PtySession>>: PTY 访问
```

---

## 错误处理

### 错误传播路径

```
可恢复错误:
  • PTY read error → 记录日志，退出 reader 线程
  • PTY write error → 记录日志，继续运行
  • Surface resize error → panic (expect)

不可恢复错误:
  • PTY 创建失败 → panic (expect)
  • 窗口创建失败 → 退出事件循环
  • 字体加载失败 → panic (expect)

清理:
  • Drop trait 确保资源释放
  • 线程超时清理 (2 秒)
  • 子进程自动终止
```

---

## 总结

### 关键设计决策

1. **事件驱动架构**: winit 事件循环驱动所有操作
2. **线程分离**: Reader 线程独立处理 PTY 输出
3. **共享状态**: Arc<Mutex<>> 实现跨线程数据共享
4. **回调机制**: PTY 输出通过回调传递给主线程
5. **简化 ANSI**: 状态机过滤，不完整但够用
6. **软件渲染**: fontdue + softbuffer，避免 GPU 依赖

### 架构优点

✅ 模块化清晰，职责分离
✅ 线程安全，使用 Rust 类型系统保证
✅ 事件驱动，响应式设计
✅ 可扩展，易于添加新功能

### 架构缺点

⚠️ 性能瓶颈在软件渲染
⚠️ 无滚动缓冲区
⚠️ ANSI 支持不完整
⚠️ 单一字体，无样式支持

### 改进方向

1. **硬件加速**: 使用 GPU 渲染 (wgpu/vulkano)
2. **滚动缓冲**: 实现历史输出保存
3. **完整 ANSI**: 使用 vte crate 完整解析
4. **多样式**: 支持颜色、粗体、斜体
5. **光标渲染**: 显示和跟随光标
