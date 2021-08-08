use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    WebGlProgram,
    WebGlRenderingContext,
    WebGlBuffer,
    WebGlShader,
    WebGlUniformLocation,
};

mod camera;
use camera::Camera;

struct ProgramInfo {
    program: WebGlProgram,
    attrib_locations: HashMap<String, u32>,
    uniform_locations: HashMap<String, WebGlUniformLocation>,
}

impl ProgramInfo {
    pub fn new(
        context: &WebGlRenderingContext,
        vert_shader: &WebGlShader,
        frag_shader: &WebGlShader,
    ) -> Result<Self, JsValue> {
        let program = init_shader_program(&context, &vert_shader, &frag_shader)?;

        let mut attrib_locations = HashMap::new();

        attrib_locations.insert(
            "vertex_position".to_owned(),
            context.get_attrib_location(
                &program,
                "a_vertex_position"
            ) as u32
        );

        let mut uniform_locations = HashMap::new();

        uniform_locations.insert(
            "colour".to_owned(),
            context.get_uniform_location(
                &program,
                "u_colour"
            ).unwrap()
        );
        uniform_locations.insert(
            "model_view_matrix".to_owned(),
            context.get_uniform_location(
                &program,
                "u_model_view_matrix"
            ).unwrap()
        );

        Ok(Self {
            program,
            attrib_locations,
            uniform_locations
        })
    }
}

fn draw_scene(
    context: &WebGlRenderingContext,
) {
    let vertex_count = 4;
    context.draw_arrays(
        WebGlRenderingContext::TRIANGLE_STRIP,
        0,
        vertex_count as i32,
    );
}

fn set_uniform(
    context: &WebGlRenderingContext,
    program_info: &ProgramInfo,
    uniform_name: &str,
    data: &[f32; 4]
) {
    let colour_location = program_info.uniform_locations.get(uniform_name);

    context.uniform4fv_with_f32_array(
        colour_location,
        data
    );
}

fn set_uniforms(
    context: &WebGlRenderingContext,
    program_info: &ProgramInfo,
) {
    set_uniform(
        context,
        program_info,
        "colour",
        &[0., 1.0, 0.6, 1.0,],
    );

    let model_view_matrix_position = program_info.uniform_locations.get("model_view_matrix");

    context.uniform_matrix4fv_with_f32_array(
        model_view_matrix_position,
        false,
        &[
            0.5, 0., 0., 0.,
            0., 1., 0., 0.,
            0., 0., 1., 0.,
            0., 0., 0., 1.,
        ],
    );
}

fn prepare_scene(
    context: &WebGlRenderingContext,
    program_info: &ProgramInfo,
    buffer: &WebGlBuffer,
    _camera: &Camera,
) {
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(buffer));
    let vertex_position = *program_info.attrib_locations.get("vertex_position").unwrap();

    context.use_program(Some(&program_info.program));

    context.vertex_attrib_pointer_with_i32(
        vertex_position,
        3,
        WebGlRenderingContext::FLOAT,
        false,
        0,
        0
    );
    context.enable_vertex_attrib_array(
        vertex_position
    );

    set_uniforms(
        context,
        program_info
    );
}

fn clear_scene(
    context: &WebGlRenderingContext,
) {
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
}

fn render_scene(
    context: &WebGlRenderingContext,
    program_info: &ProgramInfo,
    buffer: &WebGlBuffer,
    camera: &Camera,
) {
    clear_scene(context);
    prepare_scene(
        context,
        program_info,
        buffer,
        camera,
    );
    draw_scene(context);
}

fn init_buffers(
    context: &WebGlRenderingContext
) -> Result<WebGlBuffer, String> {
    let buffer = context.create_buffer().ok_or("failed to create buffer")?;
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

    let vertices: [f32; 12] = [
        -0.5,  0.5, 0.0,
        0.5,  0.5, 0.0,
        -0.5, -0.5, 0.0,
        0.5, -0.5, 0.0,
    ];

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);

        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGlRenderingContext::STATIC_DRAW,
        );

    }
    Ok(buffer)
}

pub fn init_shader_program(
    context: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

pub fn load_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(||
                String::from("Unknown error creating shader")
            ))
    }
}

fn init_context() -> Result<WebGlRenderingContext, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    Ok(context)
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let context = init_context()?;
    let vert_shader = load_shader(
        &context,
        WebGlRenderingContext::VERTEX_SHADER,
        r#"
        attribute vec4 a_vertex_position;

        uniform mat4 u_model_view_matrix;
        // uniform mat4 uProjectionMatrix;

        void main() {
            gl_Position = u_model_view_matrix * a_vertex_position;
            // gl_Position = uProjectionMatrix * u_model_view_matrix * a_vertex_position;
        }
    "#,
    )?;
    let frag_shader = load_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        r#"

        precision mediump float;
        uniform vec4 u_colour;

        void main() {
            gl_FragColor = u_colour;
        }
    "#,
    )?;
    let program_info = ProgramInfo::new(&context, &vert_shader, &frag_shader)?;
    let positions_buffer = init_buffers(&context)?;
    let camera = Camera::new();

    render_scene(
        &context,
        &program_info,
        &positions_buffer,
        &camera
    );
    Ok(())
}