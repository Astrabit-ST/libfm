#![warn(rust_2018_idioms, clippy::all)]
#![feature(buf_read_has_data_left)]

mod message_thread;

use glow::{NativeProgram, NativeVertexArray};
use glutin::{
    config::ConfigTemplateBuilder,
    context::ContextAttributesBuilder,
    display::GetGlDisplay,
    prelude::*,
    surface::{SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use image::DynamicImage;
use raw_window_handle::HasRawWindowHandle;
use std::{ffi::CString, mem::size_of, num::NonZeroU32, sync::Arc};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use bytemuck::cast_slice;

#[cfg(target_os = "windows")]
use winit::platform::windows::WindowBuilderExtWindows;

fn main() {
    let event_loop = EventLoop::new();

    let window_builder = WindowBuilder::new()
        .with_transparent(true)
        .with_always_on_top(true)
        .with_decorations(false)
        .with_visible(false)
        .with_position(PhysicalPosition::new(0, 0))
        .with_inner_size(PhysicalSize::new(480, 480))
        .with_resizable(false);

    #[cfg(target_os = "windows")]
    let window_builder = window_builder.with_skip_taskbar(true);

    let template = ConfigTemplateBuilder::new().with_alpha_size(8);

    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(&event_loop, template, |configs| {
            configs
                .reduce(|accum, config| {
                    let transparency_check = config.supports_transparency().unwrap_or(false)
                        & !accum.supports_transparency().unwrap_or(false);

                    if transparency_check || config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .unwrap();

    let window = Arc::new(window.unwrap());

    let surface = unsafe {
        {
            let (width, height): (u32, u32) = window.inner_size().into();
            let raw_window_handle = window.raw_window_handle();
            let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
                raw_window_handle,
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            );

            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        }
    };

    let gl_display = gl_config.display();

    let (glutin_context, gl) = unsafe {
        let context = gl_display
            .create_context(
                &gl_config,
                &ContextAttributesBuilder::new().build(Some(window.raw_window_handle())),
            )
            .unwrap();

        let context = context.make_current(&surface).unwrap();

        let gl = glow::Context::from_loader_function(|sym| {
            let sym = CString::new(sym).unwrap();
            gl_display.get_proc_address(&sym)
        });

        (context, gl)
    };

    let mut renderer = unsafe { Renderer::new(gl) };

    let window_clone = window.clone();
    let (image_sender, image_reciever) = crossbeam_channel::bounded(1);
    std::thread::spawn(move || {
        let window = window_clone;

        message_thread::message_thread(window, image_sender);
    });

    event_loop.run(move |event, _, _| {
        if let Ok(image) = image_reciever.try_recv() {
            let size = PhysicalSize::new(image.width(), image.height());
            window.set_inner_size(size);

            surface.resize(
                &glutin_context,
                NonZeroU32::new(image.width()).unwrap(),
                NonZeroU32::new(image.height()).unwrap(),
            );

            unsafe {
                renderer.load_texture(image);
            }
        }

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } => {
                if window_id == window.id() {
                    match event {
                        WindowEvent::Resized(_physical_size) => {}
                        WindowEvent::ScaleFactorChanged { .. } => {}
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => unsafe {
                renderer.redraw();

                surface.swap_buffers(&glutin_context).unwrap();
            },
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    })
}

#[allow(dead_code)]
struct Renderer {
    gl: glow::Context,
    program: NativeProgram,
    vao: NativeVertexArray,
    texture: Option<glow::NativeTexture>,
}

impl Renderer {
    unsafe fn new(gl: glow::Context) -> Self {
        use glow::HasContext;

        const VERTICIES: &[f32] = &[
            // vertex     | texture
            1.0, 1.0, 0.0, 1.0, 1.0, // top right
            1.0, -1.0, 0.0, 1.0, 0.0, // bottom right
            -1.0, -1.0, 0.0, 0.0, 0.0, // bottom left
            -1.0, 1.0, 0.0, 0.0, 1.0, // top left
        ];
        const INDICES: &[u32] = &[
            0, 1, 3, // first triangle
            1, 2, 3, // second triangle
        ];

        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, cast_slice(VERTICIES), glow::STATIC_DRAW);

        let ebo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            cast_slice(INDICES),
            glow::STATIC_DRAW,
        );

        let vert_contents = "
        #version 330 core
        layout (location = 0) in vec3 aPos;
        layout (location = 1) in vec2 aTexCoord;

        out vec2 TexCoord;
        
        void main()
        {
            gl_Position = vec4(aPos, 1.0);
            TexCoord = aTexCoord;
        }";
        let vert = gl.create_shader(glow::VERTEX_SHADER).unwrap();
        gl.shader_source(vert, vert_contents);
        gl.compile_shader(vert);

        let frag_contents = "
        #version 330 core
        out vec4 FragColor;
          
        in vec2 TexCoord;
        
        uniform sampler2D ourTexture;
        
        void main()
        {
            FragColor = texture(ourTexture, TexCoord);
        }";
        let frag = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
        gl.shader_source(frag, frag_contents);
        gl.compile_shader(frag);

        let program = gl.create_program().unwrap();
        gl.attach_shader(program, vert);
        gl.attach_shader(program, frag);
        gl.link_program(program);

        gl.delete_shader(vert);
        gl.delete_shader(frag);

        gl.use_program(Some(program));

        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, (size_of::<f32>() * 5) as _, 0);
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(
            1,
            2,
            glow::FLOAT,
            false,
            (size_of::<f32>() * 5) as _,
            (size_of::<f32>() * 3) as _,
        );
        gl.enable_vertex_attrib_array(1);

        Self {
            gl,
            program,
            vao,
            texture: None,
        }
    }

    unsafe fn redraw(&mut self) {
        use glow::HasContext;
        let Self { program, vao, .. } = *self;
        let gl = &mut self.gl;

        gl.clear_color(0.1, 0.1, 0.1, 0.1);
        gl.clear(glow::COLOR_BUFFER_BIT);

        gl.use_program(Some(program));
        gl.bind_vertex_array(Some(vao));

        if let Some(texture) = self.texture {
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        }

        gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);
    }

    unsafe fn load_texture(&mut self, image: DynamicImage) {
        use glow::HasContext;
        let gl = &mut self.gl;

        if let Some(tex) = self.texture.take() {
            gl.delete_texture(tex);
        }

        let image = image.flipv();
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as _,
            image.width() as _,
            image.height() as _,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(image.into_rgba8().as_ref()),
        );
        gl.generate_mipmap(glow::TEXTURE_2D);

        self.texture = Some(texture);
    }
}
