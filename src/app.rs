use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver},
    thread::{current, spawn, yield_now, ThreadId},
};

use glfw::*;
use gom::*;

use crate::{debug, error, warn};
const GLFW: &str = id!(GLFW);
const APP: &str = id!(APP);
/// 窗口实例ID
pub const WINDOW: &str = id!(@GLFW.WINODW);
const EVENT_MS: &str = id!(@WINDOW.EVENT_MS);
const RENDER_MS: &str = id!(@WINDOW.RENDER_MS);
const CATON: &str = id!(@WINDOW.CATON);

const THREAD_NAMES: &str = id!(@APP.THREAD_NAMES);
type NameTable = HashMap<ThreadId, String>;

pub use glfw::{Action, CursorMode, Key, Modifiers};

/// 用于构建App实例
///
/// # 示例
///
/// ```
/// use rustcraft::AppBuilder;
///
/// let mut app = AppBuilder::new(800, 600, "RustCraft").build();
/// ```
pub struct AppBuilder {
    size: (i32, i32),
    title: String,
    render_init: Option<Box<dyn FnOnce() + 'static + Send>>,
    render_loop: Option<Box<dyn FnMut() + 'static + Send>>,
    event_init: Option<Box<dyn FnOnce() + 'static + Send>>,
    event_loop: Option<Box<dyn FnMut() + 'static + Send>>,
    window_size_callback: Option<Box<dyn FnMut(i32, i32) + 'static + Send>>,
    window_pos_callback: Option<Box<dyn FnMut(i32, i32) + 'static + Send>>,
    window_close_callback: Option<Box<dyn FnMut() + 'static + Send>>,
    key_callback: Option<Box<dyn FnMut(Key, i32, Action, Modifiers) + 'static + Send>>,
    mouse_button_callback: Option<Box<dyn FnMut(MouseButton, Action, Modifiers) + 'static + Send>>,
    cursor_pos_callback: Option<Box<dyn FnMut(f64, f64) + 'static + Send>>,
    scroll_callback: Option<Box<dyn FnMut(f64, f64) + 'static + Send>>,
}

impl AppBuilder {
    /// 创建一个新的`AppBuilder`实例
    ///
    /// # 参数
    /// + `width` - 窗口宽度
    /// + `height` - 窗口高度
    /// + `title` - 窗口标题
    ///
    /// # 返回值
    /// 返回一个新的`AppBuilder`实例
    pub fn new(width: i32, height: i32, title: &str) -> Self {
        Self {
            size: (width, height),
            title: title.to_string(),
            render_init: None,
            render_loop: None,
            event_init: None,
            event_loop: None,
            window_size_callback: None,
            window_pos_callback: None,
            window_close_callback: None,
            key_callback: None,
            mouse_button_callback: None,
            cursor_pos_callback: None,
            scroll_callback: None,
        }
    }

    /// 设置渲染线程的初始化函数
    ///
    /// # 参数
    /// + `f` - 一个函数，它将在渲染线程的OpenGL上下文初始化后，渲染循环开始前被调用
    ///
    /// # 返回值
    /// 返回`AppBuilder`实例本身
    pub fn set_render_init<F: 'static + FnOnce() + Send>(&mut self, f: F) -> &mut Self {
        self.render_init = Some(Box::new(f));
        self
    }

    /// 设置渲染线程的循环函数
    ///
    /// # 参数
    /// + `f` - 一个函数，它将在渲染线程的渲染循环中被循环调用
    ///
    /// # 返回值
    /// 返回`AppBuilder`实例本身
    pub fn set_render_loop<F: 'static + FnMut() + Send>(&mut self, f: F) -> &mut Self {
        self.render_loop = Some(Box::new(f));
        self
    }

    /// 设置事件线程的初始化函数
    ///
    /// # 参数
    /// + `f` - 一个函数，它将在事件线程的事件循环开始前被调用
    ///
    /// # 返回值
    /// 返回`AppBuilder`实例本身
    pub fn set_event_init<F: 'static + FnOnce() + Send>(&mut self, f: F) -> &mut Self {
        self.event_init = Some(Box::new(f));
        self
    }

    /// 设置事件线程的循环函数
    ///
    /// # 参数
    /// + `f` - 一个函数，它将在事件线程的事件循环中被循环调用
    ///
    /// # 返回值
    /// 返回`AppBuilder`实例本身
    ///
    /// # 注解
    ///
    /// 当窗口处于大小或位置变化过程中时，事件循环将被阻塞，直到窗口脱离此状态
    pub fn set_event_loop<F: 'static + FnMut() + Send>(&mut self, f: F) -> &mut Self {
        self.event_loop = Some(Box::new(f));
        self
    }

    /// 设置窗口大小变化回调函数
    ///
    /// # 参数
    /// + `f` - 一个函数，它将在窗口大小发生变化时被调用，该函数接受两个参数：`fn(width: i32, height: i32)`
    ///         + `width` - 窗口宽度
    ///         + `height` - 窗口高度
    ///
    /// # 返回值
    /// 返回`AppBuilder`实例本身
    pub fn set_window_size_callback<F: 'static + FnMut(i32, i32) + Send>(
        &mut self,
        f: F,
    ) -> &mut Self {
        self.window_size_callback = Some(Box::new(f));
        self
    }

    /// 设置窗口位置变化回调函数
    ///
    /// # 参数
    /// + `f` - 一个函数，它将在窗口位置发生变化时被调用，该函数接受两个参数：`fn(x: i32, y: i32)`
    ///         + `x` - 窗口左上角横坐标
    ///         + `y` - 窗口左上角纵坐标
    ///
    /// # 返回值
    /// 返回`AppBuilder`实例本身
    pub fn set_window_pos_callback<F: 'static + FnMut(i32, i32) + Send>(
        &mut self,
        f: F,
    ) -> &mut Self {
        self.window_pos_callback = Some(Box::new(f));
        self
    }

    /// 设置窗口关闭回调函数
    ///
    /// # 参数
    /// + `f` - 一个函数，它将在窗口关闭时被调用
    ///
    /// # 返回值
    /// 返回`AppBuilder`实例本身
    pub fn set_window_close_callback<F: 'static + FnMut() + Send>(&mut self, f: F) -> &mut Self {
        self.window_close_callback = Some(Box::new(f));
        self
    }

    /// 设置键盘按键回调函数
    ///
    /// # 参数
    /// + `f` - 一个函数，它将在用户按下按键时被调用，该函数接受四个参数：`fn(key: Key, scancode: i32, action: Action, modifiers: Modifiers)`
    ///         + `key` - 按下的键
    ///         + `scancode` - 按键的扫描码
    ///         + `action` - 按键动作
    ///         + `modifiers` - 按键修饰符
    ///
    /// # 返回值
    /// 返回`AppBuilder`实例本身
    pub fn set_key_callback<F: 'static + FnMut(Key, i32, Action, Modifiers) + Send>(
        &mut self,
        f: F,
    ) -> &mut Self {
        self.key_callback = Some(Box::new(f));
        self
    }

    /// 设置鼠标按键回调函数
    ///
    /// # 参数
    /// + `f` - 一个函数，它将在用户按下鼠标按键时被调用，该函数接受三个参数：`fn(button: MouseButton, action: Action, modifiers: Modifiers)`
    ///         + `button` - 按下的鼠标按键
    ///         + `action` - 鼠标按键动作
    ///         + `modifiers` - 鼠标按键修饰符
    ///
    /// # 返回值
    /// 返回`AppBuilder`实例本身
    pub fn set_mouse_button_callback<F: 'static + FnMut(MouseButton, Action, Modifiers) + Send>(
        &mut self,
        f: F,
    ) -> &mut Self {
        self.mouse_button_callback = Some(Box::new(f));
        self
    }

    /// 设置鼠标光标位置回调函数
    ///
    /// # 参数
    /// + `f` - 一个函数，它将在鼠标光标位置发生变化时被调用，该函数接受两个参数：`fn(x: f64, y: f64)`
    ///         + `x` - 鼠标光标横坐标
    ///         + `y` - 鼠标光标纵坐标
    ///
    /// # 返回值
    /// 返回`AppBuilder`实例本身
    pub fn set_cursor_pos_callback<F: 'static + FnMut(f64, f64) + Send>(
        &mut self,
        f: F,
    ) -> &mut Self {
        self.cursor_pos_callback = Some(Box::new(f));
        self
    }

    /// 设置滚轮回调函数
    ///
    /// # 参数
    /// + `f` - 一个函数，它将在滚轮滚动时被调用，该函数接受两个参数：`fn(x: f64, y: f64)`
    ///         + `x` - 滚轮滚动横向距离
    ///         + `y` - 滚轮滚动纵向距离
    ///
    /// # 返回值
    /// 返回`AppBuilder`实例本身
    pub fn set_scroll_callback<F: 'static + FnMut(f64, f64) + Send>(&mut self, f: F) -> &mut Self {
        self.scroll_callback = Some(Box::new(f));
        self
    }

    /// 构建`App`实例
    ///
    /// # 返回值
    /// 返回一个新的`App`实例
    pub fn build(&mut self) -> App {
        App::set_current_thread_name("MainThread");
        if Registry::<PWindow>::exists(WINDOW) {
            error!(Self, "已存在一个 App 实例");
            panic!("重复创建 App 实例");
        }
        // 初始化GLFW环境并创建窗口实例
        debug!(Self, "正在初始化 GLFW 环境...");
        let mut glfw = init(fail_on_errors).unwrap();
        glfw.window_hint(WindowHint::Visible(false));
        let (window, _) = glfw
            .create_window(
                self.size.0 as _,
                self.size.1 as _,
                &self.title,
                WindowMode::Windowed,
            )
            .unwrap();
        Registry::register(WINDOW, window).unwrap();
        // 注册窗口回调函数
        debug!(Self, "正在注册回调函数...");
        let mut window_size_callback = self.window_size_callback.take();
        let mut window_pos_callback = self.window_pos_callback.take();
        let mut window_close_callback = self.window_close_callback.take();
        let mut key_callback = self.key_callback.take();
        let mut mouse_button_callback = self.mouse_button_callback.take();
        let mut cursor_pos_callback = self.cursor_pos_callback.take();
        let mut scroll_callback = self.scroll_callback.take();
        Registry::apply(WINDOW, |w: &mut PWindow| {
            w.set_size_callback(move |_, width, height| {
                if let Some(f) = window_size_callback.as_mut() {
                    f(width, height);
                }
            });
            w.set_pos_callback(move |_, x: i32, y: i32| {
                if let Some(f) = window_pos_callback.as_mut() {
                    f(x, y);
                }
            });
            w.set_close_callback(move |_| {
                if let Some(f) = window_close_callback.as_mut() {
                    f();
                }
            });
            w.set_key_callback(move |_, k, s, a, m| {
                if let Some(f) = key_callback.as_mut() {
                    f(k, s, a, m);
                }
            });
            w.set_mouse_button_callback(move |_, mb, a, m| {
                if let Some(f) = mouse_button_callback.as_mut() {
                    f(mb, a, m);
                }
            });
            w.set_cursor_pos_callback(move |_, x, y| {
                if let Some(f) = cursor_pos_callback.as_mut() {
                    f(x, y);
                }
            });
            w.set_scroll_callback(move |_, x, y| {
                if let Some(f) = scroll_callback.as_mut() {
                    f(x, y);
                }
            });
        });
        // 启动渲染循环
        debug!(Self, "正在启动渲染线程...");
        let (show_window, render_initialized) = channel();
        let render_init = self.render_init.take().unwrap_or_else(|| Box::new(|| {}));
        let mut render_loop = self.render_loop.take().unwrap_or_else(|| Box::new(|| {}));
        let (event_loop_exit, render_thread_exit) = channel();
        spawn(move || {
            App::set_current_thread_name("RenderThread");
            Registry::apply(WINDOW, |w: &mut PWindow| w.make_current());
            gl::load_with(|s| {
                Registry::apply(WINDOW, |w: &mut PWindow| w.get_proc_address(s)).unwrap()
            });

            render_init();
            show_window.send(()).unwrap();
            let mut last_render_ms = chrono::Local::now().timestamp_micros() as f64 / 1000.0;
            while Registry::with(WINDOW, |w: &PWindow| !w.should_close()).unwrap_or(false) {
                let render_ms = chrono::Local::now().timestamp_micros() as f64 / 1000.0;
                let dt = render_ms - last_render_ms;
                last_render_ms = render_ms;
                let caton = Registry::with(CATON, |caton: &f64| *caton).unwrap_or(16.67);
                if dt > caton {
                    warn!(Self, "渲染时间 {:.2}ms 超过 {:.2}ms", dt, caton);
                }
                Registry::register(RENDER_MS, dt).unwrap();
                Registry::with(WINDOW, |window: &PWindow| {
                    let (w, h) = window.get_size();
                    unsafe { gl::Viewport(0, 0, w, h) };
                });

                render_loop();
                Registry::apply(WINDOW, |w: &mut PWindow| w.swap_buffers());
            }
            debug!(Self, "渲染线程退出");
            event_loop_exit.send(()).unwrap();
        });
        render_initialized.recv().unwrap();
        debug!(Self, "显示窗口");
        Registry::apply(WINDOW, |w: &mut PWindow| w.show());
        // 返回 App 实例
        App {
            glfw,
            event_init: self.event_init.take(),
            event_loop: self.event_loop.take(),
            render_thread_exit,
        }
    }
}

/// 用于运行App实例
///
/// # 示例
///
/// ```
/// use rustcraft::AppBuilder;
///
/// let mut app = AppBuilder::new(800, 600, "RustCraft").build();
/// app.exec();
/// ```
pub struct App {
    glfw: Glfw,
    event_init: Option<Box<dyn FnOnce() + 'static + Send>>,
    event_loop: Option<Box<dyn FnMut() + 'static + Send>>,
    render_thread_exit: Receiver<()>,
}

impl App {
    /// 运行事件循环
    pub fn exec(&mut self) {
        debug!(Self, "正在启动事件循环...");
        let event_init = self.event_init.take().unwrap_or_else(|| Box::new(|| {}));
        let mut event_loop = self.event_loop.take().unwrap_or_else(|| Box::new(|| {}));
        event_init();
        let mut last_event_ms = chrono::Local::now().timestamp_micros() as f64 / 1000.0;
        loop {
            if let Ok(_) = self.render_thread_exit.try_recv() {
                break;
            }
            yield_now();

            let event_ms = chrono::Local::now().timestamp_micros() as f64 / 1000.0;
            let dt = event_ms - last_event_ms;
            last_event_ms = event_ms;
            Registry::register(EVENT_MS, dt).unwrap();

            event_loop();
            self.glfw.poll_events();
        }
        debug!(Self, "事件循环退出");
    }

    /// 退出程序
    pub fn exit() {
        Registry::apply(WINDOW, |w: &mut PWindow| {
            w.set_should_close(true);
        });
    }

    /// 获取窗口大小
    ///
    /// # 返回值
    /// 返回窗口的宽度和高度
    pub fn window_size() -> (i32, i32) {
        Registry::with(WINDOW, |w: &PWindow| w.get_size()).unwrap()
    }

    /// 获取事件循环最近一帧的运行时间
    ///
    /// # 返回值
    /// 返回事件循环最近一帧的运行时间，单位为毫秒
    pub fn event_ms() -> f64 {
        Registry::with(EVENT_MS, |ms: &f64| *ms).unwrap_or(0.0)
    }

    /// 获取渲染循环最近一帧的运行时间
    ///
    /// # 返回值
    /// 返回渲染循环最近一帧的运行时间，单位为毫秒
    pub fn render_ms() -> f64 {
        Registry::with(RENDER_MS, |ms: &f64| *ms).unwrap_or(0.0)
    }

    /// 获取事件循环的帧率
    ///
    /// # 返回值
    /// 返回事件循环的帧率
    pub fn event_fps() -> f64 {
        1000.0 / App::event_ms()
    }

    /// 获取渲染循环的帧率
    ///
    /// # 返回值
    /// 返回渲染循环的帧率
    pub fn render_fps() -> f64 {
        1000.0 / App::render_ms()
    }

    /// 设置鼠标光标模式
    ///
    /// # 参数
    /// + `mode` - 鼠标光标模式，可以是:
    ///   + `CursorMode::Normal` - 正常模式
    ///   + `CursorMode::Hidden` - 隐藏模式
    ///   + `CursorMode::Disabled` - 禁用模式
    pub fn set_cursor_mode(mode: CursorMode) {
        Registry::apply(WINDOW, |w: &mut PWindow| w.set_cursor_mode(mode));
    }

    fn _lazy_init_thread_names() {
        if !Registry::<NameTable>::exists(THREAD_NAMES) {
            Registry::<NameTable>::register(THREAD_NAMES, HashMap::new()).unwrap();
        }
    }

    /// 为当前线程设置名称
    ///
    /// # 参数
    /// + `name` - 线程名称
    pub fn set_current_thread_name(name: &str) {
        Self::_lazy_init_thread_names();
        let thread_id = current().id();
        Registry::apply(THREAD_NAMES, |map: &mut NameTable| {
            map.insert(thread_id, String::from(name));
        });
    }

    fn _get_thread_name() -> Option<String> {
        Self::_lazy_init_thread_names();
        let thread_id = current().id();
        Registry::with(THREAD_NAMES, |map: &NameTable| {
            map.get(&thread_id).cloned()
        })?
    }

    /// 获取当前线程的名称
    ///
    /// # 返回值
    /// 返回当前线程的名称，如果没有设置名称，则返回线程的ID
    pub fn current_thread_name() -> String {
        Self::_get_thread_name().unwrap_or_else(|| format!("Thread-{:?}", current().id()))
    }

    /// 设置渲染卡顿判定的临界时长
    ///
    /// # 参数
    /// + `caton` - 临界时长，单位为毫秒(默认值为16.67)
    pub fn set_caton(caton: f64) {
        Registry::register(CATON, caton).unwrap();
    }
}
