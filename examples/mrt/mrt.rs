//------------------------------------------------------------------------------
//  mrt.rs
//
//  Rendering with multiple-rendertargets, and reallocate render targets
//  on window resize events.
//
//  NOTE: the rotation direction will appear different on the different
//  backend 3D APIs. This is because of the different image origin conventions
//  in GL vs D3D vs Metal. We don't care about those differences in this sample
//  (using the sokol shader compiler allows to easily 'normalize' those differences.
//------------------------------------------------------------------------------
mod math;
mod shader;

use math as m;
use sokol::{app as sapp, gfx as sg};

const OFFSCREEN_SAMPLE_COUNT: usize = 1;

struct Offscreen {
    pub pass_action: sg::PassAction,
    pub pass_desc: sg::PassDesc,
    pub pass: sg::Pass,
    pub pip: sg::Pipeline,
    pub bind: sg::Bindings,
}

struct Fsq {
    pub pip: sg::Pipeline,
    pub bind: sg::Bindings,
}

struct Dbg {
    pub pip: sg::Pipeline,
    pub bind: sg::Bindings,
}

struct Dflt {
    pub pass_action: sg::PassAction,
}

struct State {
    pub offscreen: Offscreen,
    pub fsq: Fsq,
    pub dbg: Dbg,
    pub dflt: Dflt,
    pub rx: f32,
    pub ry: f32,
    pub view: m::Mat4,
}

static mut STATE: State = State {
    offscreen: Offscreen {
        pass_action: sg::PassAction::new(),
        pass_desc: sg::PassDesc::new(),
        pass: sg::Pass::new(),
        pip: sg::Pipeline::new(),
        bind: sg::Bindings::new(),
    },
    fsq: Fsq { pip: sg::Pipeline::new(), bind: sg::Bindings::new() },
    dbg: Dbg { pip: sg::Pipeline::new(), bind: sg::Bindings::new() },
    dflt: Dflt { pass_action: sg::PassAction::new() },
    rx: 0.0,
    ry: 0.0,
    view: [[0.0; 4]; 4],
};

extern "C" fn init() {
    let state = unsafe { &mut STATE };

    state.view = m::lookat_mat4(m::vec3(0.0, 1.5, 6.0), m::Vec3::ZERO, m::vec3(0.0, 1.0, 0.0));

    sg::setup(&sg::Desc {
        context: sokol::glue::context(),
        logger: sg::Logger { func: Some(sokol::log::slog_func), ..Default::default() },
        ..Default::default()
    });

    // setup pass action for default render pass
    state.dflt.pass_action.colors[0] =
        sg::ColorAttachmentAction { action: sg::Action::Dontcare, ..Default::default() };
    state.dflt.pass_action.depth =
        sg::DepthAttachmentAction { action: sg::Action::Dontcare, ..Default::default() };
    state.dflt.pass_action.stencil =
        sg::StencilAttachmentAction { action: sg::Action::Dontcare, ..Default::default() };

    // set pass action for offscreen render pass
    state.offscreen.pass_action.colors[0] = sg::ColorAttachmentAction {
        action: sg::Action::Clear,
        value: sg::Color { r: 0.25, g: 0.0, b: 0.0, a: 1.0 },
    };
    state.offscreen.pass_action.colors[1] = sg::ColorAttachmentAction {
        action: sg::Action::Clear,
        value: sg::Color { r: 0.0, g: 0.25, b: 0.0, a: 1.0 },
    };
    state.offscreen.pass_action.colors[2] = sg::ColorAttachmentAction {
        action: sg::Action::Clear,
        value: sg::Color { r: 0.0, g: 0.0, b: 0.25, a: 1.0 },
    };

    // setup the offscreen render pass and render target images,
    // this will also be called when the window resizes
    create_offscreen_pass(sapp::width(), sapp::height());

    #[rustfmt::skip]
    const VERTICES: &[f32] = &[
        // positions        brightness
        -1.0, -1.0, -1.0,   1.0,
         1.0, -1.0, -1.0,   1.0,
         1.0,  1.0, -1.0,   1.0,
        -1.0,  1.0, -1.0,   1.0,

        -1.0, -1.0,  1.0,   0.8,
         1.0, -1.0,  1.0,   0.8,
         1.0,  1.0,  1.0,   0.8,
        -1.0,  1.0,  1.0,   0.8,

        -1.0, -1.0, -1.0,   0.6,
        -1.0,  1.0, -1.0,   0.6,
        -1.0,  1.0,  1.0,   0.6,
        -1.0, -1.0,  1.0,   0.6,

         1.0, -1.0, -1.0,   0.0,
         1.0,  1.0, -1.0,   0.0,
         1.0,  1.0,  1.0,   0.0,
         1.0, -1.0,  1.0,   0.0,

        -1.0, -1.0, -1.0,   0.5,
        -1.0, -1.0,  1.0,   0.5,
         1.0, -1.0,  1.0,   0.5,
         1.0, -1.0, -1.0,   0.5,

        -1.0,  1.0, -1.0,   0.7,
        -1.0,  1.0,  1.0,   0.7,
         1.0,  1.0,  1.0,   0.7,
         1.0,  1.0, -1.0,   0.7,
    ];

    // create vertex buffer for a cube
    let cube_vbuf =
        sg::make_buffer(&sg::BufferDesc { data: sg::slice_as_range(VERTICES), ..Default::default() });

    #[rustfmt::skip]
    const INDICES: &[u16] = &[
        0,  1,  2,   0,  2,  3,
        6,  5,  4,   7,  6,  4,
        8,  9,  10,  8,  10, 11,
        14, 13, 12,  15, 14, 12,
        16, 17, 18,  16, 18, 19,
        22, 21, 20,  23, 22, 20,
    ];

    // index buffer for a cube
    let cube_ibuf = sg::make_buffer(&sg::BufferDesc {
        _type: sg::BufferType::Indexbuffer,
        data: sg::slice_as_range(INDICES),
        ..Default::default()
    });

    // resource bindings for offscreen rendering
    state.offscreen.bind.vertex_buffers[0] = cube_vbuf;
    state.offscreen.bind.index_buffer = cube_ibuf;

    // shader and pipeline state object for rendering cube into MRT render targets
    let mut offscreen_pip_desc = sg::PipelineDesc {
        shader: sg::make_shader(&shader::offscreen_shader_desc(sg::query_backend())),
        index_type: sg::IndexType::Uint16,
        cull_mode: sg::CullMode::Back,
        sample_count: OFFSCREEN_SAMPLE_COUNT as _,
        depth: sg::DepthState {
            pixel_format: sg::PixelFormat::Depth,
            compare: sg::CompareFunc::LessEqual,
            write_enabled: true,
            ..Default::default()
        },
        color_count: 3,
        ..Default::default()
    };
    offscreen_pip_desc.layout.attrs[shader::ATTR_VS_OFFSCREEN_POS].format = sg::VertexFormat::Float3;
    offscreen_pip_desc.layout.attrs[shader::ATTR_VS_OFFSCREEN_BRIGHT0].format = sg::VertexFormat::Float;
    state.offscreen.pip = sg::make_pipeline(&offscreen_pip_desc);

    const QUAD_VERTICES: &[f32] = &[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    // a vertex buffer to render a fullscreen quad
    let quad_vbuf =
        sg::make_buffer(&sg::BufferDesc { data: sg::slice_as_range(QUAD_VERTICES), ..Default::default() });

    // shader and pipeline object to render a fullscreen quad which composes
    // the 3 offscreen render targets into the default framebuffer
    let mut fsq_pip_desc = sg::PipelineDesc {
        shader: sg::make_shader(&shader::fsq_shader_desc(sg::query_backend())),
        primitive_type: sg::PrimitiveType::TriangleStrip,
        ..Default::default()
    };
    fsq_pip_desc.layout.attrs[shader::ATTR_VS_FSQ_POS].format = sg::VertexFormat::Float2;
    state.fsq.pip = sg::make_pipeline(&fsq_pip_desc);

    // resource bindings to render the fullscreen quad (composed from the
    // offscreen render target textures
    state.fsq.bind.vertex_buffers[0] = quad_vbuf;
    for i in 0..=2 {
        state.fsq.bind.fs_images[i] = state.offscreen.pass_desc.color_attachments[i].image;
    }

    // shader, pipeline and resource bindings to render debug visualization quads
    let mut dbg_pip_desc = sg::PipelineDesc {
        shader: sg::make_shader(&shader::dbg_shader_desc(sg::query_backend())),
        primitive_type: sg::PrimitiveType::TriangleStrip,
        ..Default::default()
    };
    dbg_pip_desc.layout.attrs[shader::ATTR_VS_DBG_POS].format = sg::VertexFormat::Float2;
    state.dbg.pip = sg::make_pipeline(&dbg_pip_desc);

    // resource bindings to render the debug visualization
    // (the required images will be filled in during rendering)
    state.dbg.bind.vertex_buffers[0] = quad_vbuf;
}

extern "C" fn frame() {
    let state = unsafe { &mut STATE };

    let dt = (sapp::frame_duration() * 60.0) as f32;
    state.rx += 1.0 * dt;
    state.ry += 2.0 * dt;

    // compute shader uniform data
    let offscreen_params = shader::OffscreenParams { mvp: compute_mvp(state.rx, state.ry) };
    let fsq_params = shader::FsqParams {
        offset: m::vec2(f32::sin(state.rx * 0.01) * 0.1, f32::cos(state.ry * 0.01) * 0.1),
        _pad_8: [0; 8],
    };

    // render cube into MRT offscreen render targets
    sg::begin_pass(state.offscreen.pass, &state.offscreen.pass_action);
    sg::apply_pipeline(state.offscreen.pip);
    sg::apply_bindings(&state.offscreen.bind);
    sg::apply_uniforms(
        sg::ShaderStage::Vs,
        shader::SLOT_OFFSCREEN_PARAMS,
        &sg::value_as_range(&offscreen_params),
    );
    sg::draw(0, 36, 1);
    sg::end_pass();

    // render fullscreen quad with the composed offscreen-render images,
    // 3 a small debug view quads at the bottom of the screen
    sg::begin_default_pass(&state.dflt.pass_action, sapp::width(), sapp::height());
    sg::apply_pipeline(state.fsq.pip);
    sg::apply_bindings(&state.fsq.bind);
    sg::apply_uniforms(sg::ShaderStage::Vs, shader::SLOT_FSQ_PARAMS, &sg::value_as_range(&fsq_params));
    sg::draw(0, 4, 1);
    sg::apply_pipeline(state.dbg.pip);
    for i in 0..=2 {
        sg::apply_viewport(i * 100, 0, 100, 100, false);
        state.dbg.bind.fs_images[0] = state.offscreen.pass_desc.color_attachments[i as usize].image;
        sg::apply_bindings(&state.dbg.bind);
        sg::draw(0, 4, 1);
    }
    sg::end_pass();
    sg::commit();
}

extern "C" fn event(event: *const sapp::Event) {
    let event = unsafe { &*event };

    if event._type == sapp::EventType::Resized {
        create_offscreen_pass(event.framebuffer_width, event.framebuffer_height);
    }
}

// helper function to create or re-create render target images and pass object for offscreen rendering
fn create_offscreen_pass(width: i32, height: i32) {
    let state = unsafe { &mut STATE };

    // destroy previous resources (can be called with invalid ids)
    sg::destroy_pass(state.offscreen.pass);
    for att in state.offscreen.pass_desc.color_attachments {
        sg::destroy_image(att.image);
    }
    sg::destroy_image(state.offscreen.pass_desc.depth_stencil_attachment.image);

    // create offscreen render target images and pass
    let color_img_desc = sg::ImageDesc {
        render_target: true,
        width,
        height,
        min_filter: sg::Filter::Linear,
        mag_filter: sg::Filter::Linear,
        wrap_u: sg::Wrap::ClampToEdge,
        wrap_v: sg::Wrap::ClampToEdge,
        sample_count: OFFSCREEN_SAMPLE_COUNT as _,

        ..Default::default()
    };

    let mut depth_img_desc = color_img_desc;
    depth_img_desc.pixel_format = sg::PixelFormat::Depth;

    for i in 0..=2 {
        state.offscreen.pass_desc.color_attachments[i].image = sg::make_image(&color_img_desc);
    }
    state.offscreen.pass_desc.depth_stencil_attachment.image = sg::make_image(&depth_img_desc);
    state.offscreen.pass = sg::make_pass(&state.offscreen.pass_desc);

    // update the fullscreen-quad texture bindings
    for i in 0..=2 {
        state.fsq.bind.fs_images[i] = state.offscreen.pass_desc.color_attachments[i].image;
    }
}

pub fn compute_mvp(rx: f32, ry: f32) -> [[f32; 4]; 4] {
    let proj = m::persp_mat4(60.0, sapp::widthf() / sapp::heightf(), 0.01, 10.0);
    let view = m::lookat_mat4(m::vec3(0.0, 1.5, 6.0), m::Vec3::ZERO, m::Vec3::UP);
    let view_proj = m::mul_mat4(proj, view);
    let rxm = m::rotate_mat4(rx, m::vec3(1.0, 0.0, 0.0));
    let rym = m::rotate_mat4(ry, m::vec3(0.0, 1.0, 0.0));
    let model = m::mul_mat4(rxm, rym);

    m::mul_mat4(view_proj, model)
}

extern "C" fn cleanup() {
    sg::shutdown()
}

fn main() {
    let window_title = b"mrt\0".as_ptr() as _;

    sapp::run(&sapp::Desc {
        init_cb: Some(init),
        frame_cb: Some(frame),
        cleanup_cb: Some(cleanup),
        event_cb: Some(event),
        width: 800,
        height: 600,
        sample_count: 1,
        window_title,
        logger: sapp::Logger { func: Some(sokol::log::slog_func), ..Default::default() },
        icon: sapp::IconDesc { sokol_default: true, ..Default::default() },

        ..Default::default()
    });
}
