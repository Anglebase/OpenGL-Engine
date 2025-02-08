use std::ffi::CString;
use gle::*;

const VS: &str = r#"
#version 330 core
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec3 aColor;

out vec3 ourColor;

void main()
{
    gl_Position = vec4(aPos, 1.0, 1.0);
    ourColor = aColor;
}
"#;

const FS: &str = r#"
#version 330 core
in vec3 ourColor;
out vec4 FragColor;

void main()
{
    FragColor = vec4(ourColor, 1.0);
}
"#;

const VET: [f32; 25] = [
    0.0, 0.5, 1.0, 0.0, 0.0, // 1
    0.5, -0.5, 0.0, 1.0, 0.0, // 2
    -0.5, -0.5, 0.0, 0.0, 1.0, // 3
    0.5, 0.5, 1.0, 1.0, 0.0, // 4
    -0.5, 0.5, 0.0, 1.0, 1.0, // 5
];

const VAO: &str = id!(VAO);

fn event_init() {
    debug!(self, "事件初始化函数执行...");
}

fn event_loop() {
    if Registry::apply(WINDOW, |w: &mut Window| w.get_key(Key::W) == Action::Press).unwrap_or(false)
    {
        println!("W key pressed");
    }

    if Registry::apply(WINDOW, |w: &mut Window| {
        w.get_key(Key::LeftAlt) == Action::Press
    })
    .unwrap_or(false)
    {
        debug!(
            "event_loop",
            "E_MS: {:>8.2}\tE_FPS: {:>8.2}\tR_MS: {:>8.2}\tR_FPS: {:>8.2}",
            App::event_ms(),
            App::event_fps(),
            App::render_ms(),
            App::render_fps()
        );
    }
}

fn render_init() {
    debug!(self, "渲染初始化函数执行...");
    unsafe {
        let mut vao = 0;
        let mut vbo = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (VET.len() * std::mem::size_of::<f32>()) as isize,
            VET.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            5 * std::mem::size_of::<f32>() as i32,
            0 as _,
        );
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            5 * std::mem::size_of::<f32>() as i32,
            (2 * std::mem::size_of::<f32>()) as _,
        );
        gl::BindVertexArray(0);
        Registry::register(VAO, vao).unwrap();

        let vs_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let fs_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let vs = CString::new(VS).unwrap();
        let fs = CString::new(FS).unwrap();
        gl::ShaderSource(vs_shader, 1, &vs.as_ptr(), std::ptr::null());
        gl::ShaderSource(fs_shader, 1, &fs.as_ptr(), std::ptr::null());
        gl::CompileShader(vs_shader);
        gl::CompileShader(fs_shader);
        let mut success = gl::FALSE as i32;
        gl::GetShaderiv(vs_shader, gl::COMPILE_STATUS, &mut success);
        if success == gl::FALSE as i32 {
            let mut len = 0;
            gl::GetShaderiv(vs_shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len(len as usize);
            gl::GetShaderInfoLog(
                vs_shader,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut _,
            );
            println!(
                "Vertex shader compile error: {}",
                std::str::from_utf8(&buf).unwrap()
            );
        }
        gl::GetShaderiv(fs_shader, gl::COMPILE_STATUS, &mut success);
        if success == gl::FALSE as i32 {
            let mut len = 0;
            gl::GetShaderiv(fs_shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len(len as usize);
            gl::GetShaderInfoLog(
                fs_shader,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut _,
            );
            println!(
                "Fragment shader compile error: {}",
                std::str::from_utf8(&buf).unwrap()
            );
        }
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs_shader);
        gl::AttachShader(program, fs_shader);
        gl::LinkProgram(program);
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == gl::FALSE as i32 {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len(len as usize);
            gl::GetProgramInfoLog(
                program,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut _,
            );
            println!("Program link error: {}", std::str::from_utf8(&buf).unwrap());
        }
        gl::UseProgram(program);
        // Registry::register(PROGRAM, program).unwrap();
    }
}

fn render_loop() {
    unsafe {
        gl::ClearColor(0.3, 0.4, 0.5, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        Registry::with(VAO, |vao: &u32| {
            gl::BindVertexArray(*vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::DrawArrays(gl::LINES, 3, 2);
            gl::BindVertexArray(0);
        });
    }
}

fn main() {
    Log::set_level(Level::Debug);
    let mut app = AppBuilder::new(800, 600, "RustCraft")
        .set_render_init(render_init)
        .set_render_loop(render_loop)
        .set_event_init(event_init)
        .set_event_loop(event_loop)
        .build();
    app.exec();
}
