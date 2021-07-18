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
            "vertexPosition".to_owned(),
            context.get_attrib_location(
                &program,
                "aVertexPosition"
            ) as u32
        );

        let mut uniform_locations = HashMap::new();
        uniform_locations.insert(
            "projectionMatrix".to_owned(),
            context.get_uniform_location(
                &program,
                "uProjectionMatrix"
            ).unwrap()
        );
        uniform_locations.insert(
            "modelViewMatrix".to_owned(),
            context.get_uniform_location(
                &program,
                "uModelViewMatrix"
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
    program_info: &ProgramInfo,
    buffer: &WebGlBuffer,
    camera: &Camera
) {
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(buffer));
    context.vertex_attrib_pointer_with_i32(
        *program_info.attrib_locations.get("vertexPosition").unwrap(),
        3,
        WebGlRenderingContext::FLOAT,
        false,
        0,
        0
    );
    context.enable_vertex_attrib_array(
        *program_info.attrib_locations.get("vertexPosition").unwrap()
    );

    context.use_program(Some(&program_info.program));
    context.uniform_matrix4fv_with_f32_array(
        program_info.uniform_locations.get("projectionMatrix"),
        false,
        &camera.projection()
    );
    context.uniform_matrix4fv_with_f32_array(
        program_info.uniform_locations.get("modelViewMatrix"),
        false,
        &camera.view()
    );


    {
        let vertex_count = 4;
        context.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            vertex_count as i32,
        );
    }
}

fn init_buffers(
    context: &WebGlRenderingContext
) -> Result<WebGlBuffer, String> {
    let buffer = context.create_buffer().ok_or("failed to create buffer")?;
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

    let vertices: [f32; 12] = [
        -1.0,  1.0, 0.0,
        1.0,  1.0, 0.0,
        -1.0, -1.0, 0.0,
        1.0, -1.0, 0.0,
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
        attribute vec4 aVertexPosition;

        uniform mat4 uModelViewMatrix;
        uniform mat4 uProjectionMatrix;

        void main() {
            // gl_Position = aVertexPosition;
            // gl_Position = uProjectionMatrix * aVertexPosition;
            gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
        }
    "#,
    )?;
    let frag_shader = load_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        r#"
        void main() {
            gl_FragColor = vec4(1.0, 1.0, 0.0, 1.0);
        }
    "#,
    )?;
    let program_info = ProgramInfo::new(&context, &vert_shader, &frag_shader)?;
    let positions_buffer = init_buffers(&context)?;
    let camera = Camera::new();

    draw_scene(&context, &program_info, &positions_buffer, &camera);
    Ok(())
}