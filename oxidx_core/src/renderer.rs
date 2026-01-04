//! # OxidX Renderer
//!
//! A batched 2D renderer that abstracts WGPU complexity from components.
//! Components call methods like `fill_rect()` and `draw_text()`, and the
//! renderer batches all primitives for efficient GPU rendering.
//!
//! ## Batching Algorithm (Updated for Z-Index)
//!
//! 1. `begin_frame()`: Clear render queue.
//! 2. Components call `fill_rect()`, `draw_text()`. Commands are pushed to `commands` queue with current `z_index` and `submission_order`.
//! 3. `end_frame()`:
//!    - Sort commands by Z-Index (primary) and Order (secondary).
//!    - Iterate through commands.
//!    - Sequential `Rect` commands are batched into vertex buffer.
//!    - `Text` commands flush the current batch, draw text, and resume batching.
//!
//! This approach allows perfect interleaving of text and geometry ("Painter's Algorithm")
//! while maintaining batching efficiency for the geometry parts.

use crate::assets::LoadedImage;
use crate::primitives::{Color, Rect, TextStyle};
use crate::style::{Background, Style};
use crate::theme::Theme;
use glam::{Mat4, Vec2};
use glyphon::cosmic_text::{
    Attrs, AttrsList, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache,
};
use glyphon::{Resolution, TextArea, TextBounds, TextRenderer as GlyphonTextRenderer};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub type TextureId = u32;

/// Vertex structure for the batched renderer.
/// Each vertex has position (pixels), color (RGBA), and UV coordinates.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

impl Vertex {
    /// Creates a new vertex with default UV (0,0).
    pub fn new(x: f32, y: f32, color: Color) -> Self {
        Self {
            position: [x, y],
            color: color.to_array(),
            uv: [0.0, 0.0],
        }
    }

    /// Vertex buffer layout for WGPU.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position at location 0
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color at location 1
                wgpu::VertexAttribute {
                    offset: 8, // 2 * sizeof(f32)
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // UV at location 2
                wgpu::VertexAttribute {
                    offset: 24, // 2 * sizeof(f32) + 4 * sizeof(f32)
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/// Uniform data for the shader (projection matrix).
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    projection: [[f32; 4]; 4],
}

/// Operation to be rendered.
#[derive(Clone, Debug)]
enum RenderOp {
    /// A set of vertices/indices representing a geometric shape (Rect, Image)
    Geometry {
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        texture_id: TextureId,
    },
    /// Text to be rendered
    Text(TextCommand),
    /// A clipping push op
    PushClip(Rect),
    /// A clipping pop op
    PopClip,
}

/// Text command details.
#[derive(Clone, Debug)]
enum TextCommand {
    Simple {
        text: String,
        position: Vec2,
        style: TextStyle,
        bounds_width: Option<f32>,
    },
    Rich {
        text: String,
        attrs: AttrsList,
        position: Vec2,
        bounds_width: Option<f32>,
        default_color: Color,
        line_height: f32,
    },
}

/// A command in the render queue.
#[derive(Clone, Debug)]
struct RenderCommand {
    /// Z-Index for explicit layering (-100 to 100, default 0)
    z_index: i32,
    /// Submission order for stable sorting (Painter's Algorithm)
    order: u64,
    /// The operation to perform
    op: RenderOp,
}

/// Initial capacity for vertex/index buffers.
const INITIAL_VERTEX_CAPACITY: usize = 4096;
const INITIAL_INDEX_CAPACITY: usize = 6144;

/// The main batched 2D renderer.
pub struct Renderer {
    // GPU resources
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    _surface_format: wgpu::TextureFormat,

    // Global theme
    pub theme: Theme,

    // Render pipeline
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    // Dynamic vertex/index buffers
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_capacity: usize,

    // Text rendering
    pub text_brush: TextBrush,

    // Unified Render Command Queue
    commands: Vec<RenderCommand>,

    // State
    current_z_index: i32,
    next_order: u64,

    // Resources
    texture_bind_groups: HashMap<TextureId, wgpu::BindGroup>,
    assets: HashMap<String, TextureId>,
    white_texture: TextureId,
    next_texture_id: TextureId,
    texture_layout: wgpu::BindGroupLayout,

    // Screen info
    screen_width: f32,
    screen_height: f32,
    physical_width: u32,
    physical_height: u32,
    scale_factor: f64,

    // Scissor clipping
    clip_stack: ClipStack,
}

/// Manages the clipping rectangles stack.
#[derive(Debug, Clone, Default)]
pub struct ClipStack {
    stack: Vec<Rect>,
}

impl ClipStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self, rect: Rect) {
        let clipped = if let Some(current) = self.stack.last() {
            current.intersect(&rect)
        } else {
            rect
        };
        self.stack.push(clipped);
    }

    pub fn pop(&mut self) -> Option<Rect> {
        self.stack.pop()
    }

    pub fn current(&self) -> Option<Rect> {
        self.stack.last().copied()
    }

    pub fn clear(&mut self) {
        self.stack.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

/// Manages text resources and rendering.
pub struct TextBrush {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub text_renderer: GlyphonTextRenderer,
    pub text_atlas: glyphon::TextAtlas,
}

impl TextBrush {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let mut text_atlas = glyphon::TextAtlas::new(device, queue, format);
        let text_renderer = GlyphonTextRenderer::new(
            &mut text_atlas,
            device,
            wgpu::MultisampleState::default(),
            None,
        );

        Self {
            font_system,
            swash_cache,
            text_renderer,
            text_atlas,
        }
    }
}

impl Renderer {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("OxidX Batched Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_batched.wgsl").into()),
        });

        let projection = Self::create_orthographic_projection(width as f32, height as f32);
        let uniforms = Uniforms {
            projection: projection.to_cols_array_2d(),
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Default white texture
        let white_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let white_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("White Texture"),
            size: white_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &white_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: None,
            },
            white_size,
        );
        let white_view = white_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let white_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let white_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("White Texture Bind Group"),
            layout: &texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&white_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&white_sampler),
                },
            ],
        });

        let mut texture_bind_groups = HashMap::new();
        let white_id = 0;
        texture_bind_groups.insert(white_id, white_bind_group);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, &texture_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("OxidX Batched Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (INITIAL_VERTEX_CAPACITY * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: (INITIAL_INDEX_CAPACITY * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let text_brush = TextBrush::new(&device, &queue, surface_format);

        Self {
            device,
            queue,
            _surface_format: surface_format,
            theme: Theme::default(),
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            vertex_buffer,
            index_buffer,
            vertex_capacity: INITIAL_VERTEX_CAPACITY,
            index_capacity: INITIAL_INDEX_CAPACITY,
            text_brush,

            commands: Vec::new(),
            current_z_index: 0,
            next_order: 0,

            texture_bind_groups,
            assets: HashMap::new(),
            white_texture: white_id,
            next_texture_id: 1,
            texture_layout,

            screen_width: width as f32,
            screen_height: height as f32,
            physical_width: width,
            physical_height: height,
            scale_factor: 1.0,
            clip_stack: ClipStack::new(),
        }
    }

    fn create_orthographic_projection(width: f32, height: f32) -> Mat4 {
        Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.resize_with_scale(width, height, 1.0);
    }

    pub fn resize_with_scale(&mut self, width: u32, height: u32, scale_factor: f64) {
        self.physical_width = width;
        self.physical_height = height;
        self.scale_factor = scale_factor;
        self.screen_width = width as f32 / scale_factor as f32;
        self.screen_height = height as f32 / scale_factor as f32;

        let projection =
            Self::create_orthographic_projection(self.screen_width, self.screen_height);
        let uniforms = Uniforms {
            projection: projection.to_cols_array_2d(),
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    pub fn screen_size(&self) -> Vec2 {
        Vec2::new(self.screen_width, self.screen_height)
    }

    /// Set the current Z-Index for subsequent draw calls.
    pub fn set_z_index(&mut self, z_index: i32) {
        self.current_z_index = z_index;
    }

    /// Get current Z-Index.
    pub fn z_index(&self) -> i32 {
        self.current_z_index
    }

    pub fn begin_frame(&mut self) {
        self.commands.clear();
        self.next_order = 0;
        self.current_z_index = 0;
        self.clip_stack.clear();
    }

    fn push_command(&mut self, op: RenderOp) {
        self.commands.push(RenderCommand {
            z_index: self.current_z_index,
            order: self.next_order,
            op,
        });
        self.next_order += 1;
    }

    // Clip Stack
    pub fn push_clip(&mut self, rect: Rect) {
        self.clip_stack.push(rect);
        // We push an Op so we can replay it during render
        self.push_command(RenderOp::PushClip(rect));
    }

    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
        self.push_command(RenderOp::PopClip);
    }

    pub fn current_clip(&self) -> Option<Rect> {
        self.clip_stack.current()
    }

    pub fn clear_clip(&mut self) {
        // Pop all clips
        while self.clip_stack.current().is_some() {
            self.pop_clip();
        }
    }

    // --- Drawing primitives ---

    pub fn create_texture(&mut self, image: &LoadedImage, label: Option<&str>) -> TextureId {
        let size = wgpu::Extent3d {
            width: image.width,
            height: image.height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.width),
                rows_per_image: None,
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &self.texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let id = self.next_texture_id;
        self.next_texture_id += 1;
        self.texture_bind_groups.insert(id, bind_group);
        id
    }

    pub fn load_image(&mut self, path: &str) -> Result<TextureId, String> {
        if let Some(&id) = self.assets.get(path) {
            return Ok(id);
        }

        let img = image::open(std::path::Path::new(path)).map_err(|e| e.to_string())?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let loaded_image = LoadedImage {
            width,
            height,
            data: rgba.into_raw(),
        };

        let id = self.create_texture(&loaded_image, Some(path));
        self.assets.insert(path.to_string(), id);
        Ok(id)
    }

    pub fn draw_image(&mut self, rect: Rect, texture_id: TextureId) {
        let mut vertices = Vec::with_capacity(4);
        let mut indices = Vec::with_capacity(6);

        vertices.push(Vertex {
            position: [rect.x, rect.y],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [0.0, 0.0],
        });
        vertices.push(Vertex {
            position: [rect.x + rect.width, rect.y],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [1.0, 0.0],
        });
        vertices.push(Vertex {
            position: [rect.x + rect.width, rect.y + rect.height],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [1.0, 1.0],
        });
        vertices.push(Vertex {
            position: [rect.x, rect.y + rect.height],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [0.0, 1.0],
        });

        indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);

        self.push_command(RenderOp::Geometry {
            vertices,
            indices,
            texture_id,
        });
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let mut vertices = Vec::with_capacity(4);
        let mut indices = Vec::with_capacity(6);

        vertices.push(Vertex::new(rect.x, rect.y, color));
        vertices.push(Vertex::new(rect.x + rect.width, rect.y, color));
        vertices.push(Vertex::new(
            rect.x + rect.width,
            rect.y + rect.height,
            color,
        ));
        vertices.push(Vertex::new(rect.x, rect.y + rect.height, color));

        indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);

        self.push_command(RenderOp::Geometry {
            vertices,
            indices,
            texture_id: self.white_texture,
        });
    }

    pub fn stroke_rect(&mut self, rect: Rect, color: Color, thickness: f32) {
        let half = thickness / 2.0;

        self.fill_rect(
            Rect::new(
                rect.x - half,
                rect.y - half,
                rect.width + thickness,
                thickness,
            ),
            color,
        );
        self.fill_rect(
            Rect::new(
                rect.x - half,
                rect.y + rect.height - half,
                rect.width + thickness,
                thickness,
            ),
            color,
        );
        self.fill_rect(
            Rect::new(
                rect.x - half,
                rect.y + half,
                thickness,
                rect.height - thickness,
            ),
            color,
        );
        self.fill_rect(
            Rect::new(
                rect.x + rect.width - half,
                rect.y + half,
                thickness,
                rect.height - thickness,
            ),
            color,
        );
    }

    pub fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color, width: f32) {
        let diff = end - start;
        let len = diff.length();
        if len < 0.001 {
            return;
        }

        let normal = Vec2::new(-diff.y, diff.x).normalize() * (width / 2.0);
        let p1 = start + normal;
        let p2 = start - normal;
        let p3 = end - normal;
        let p4 = end + normal;

        let mut vertices = Vec::with_capacity(4);
        vertices.push(Vertex::new(p1.x, p1.y, color));
        vertices.push(Vertex::new(p2.x, p2.y, color));
        vertices.push(Vertex::new(p3.x, p3.y, color));
        vertices.push(Vertex::new(p4.x, p4.y, color));

        let indices = vec![0, 1, 2, 0, 2, 3];

        self.push_command(RenderOp::Geometry {
            vertices,
            indices,
            texture_id: self.white_texture,
        });
    }

    pub fn draw_rounded_rect(
        &mut self,
        rect: Rect,
        color: Color,
        _radius: f32,
        border_color: Option<Color>,
        border_width: Option<f32>,
    ) {
        self.fill_rect(rect, color);
        if let Some(bc) = border_color {
            if let Some(bw) = border_width {
                self.stroke_rect(rect, bc, bw);
            }
        }
    }

    pub fn draw_style_rect(&mut self, rect: Rect, style: &Style) {
        if let Some(shadow) = &style.shadow {
            let shadow_rect = Rect::new(
                rect.x + shadow.offset.x,
                rect.y + shadow.offset.y,
                rect.width,
                rect.height,
            );
            self.fill_rect(shadow_rect, shadow.color);
        }

        if let Some(border) = &style.border {
            if border.width > 0.0 {
                let border_rect = Rect::new(
                    rect.x - border.width,
                    rect.y - border.width,
                    rect.width + border.width * 2.0,
                    rect.height + border.width * 2.0,
                );
                self.fill_rect(border_rect, border.color);
            }
        }

        match style.background {
            Background::Solid(color) => {
                self.fill_rect(rect, color);
            }
            Background::LinearGradient { start, end, .. } => {
                let r = (start.r + end.r) / 2.0;
                let g = (start.g + end.g) / 2.0;
                let b = (start.b + end.b) / 2.0;
                let a = (start.a + end.a) / 2.0;
                self.fill_rect(rect, Color::new(r, g, b, a));
            }
        }
    }

    // --- Text Rendering ---

    pub fn draw_text(&mut self, text: impl Into<String>, position: Vec2, style: TextStyle) {
        self.push_command(RenderOp::Text(TextCommand::Simple {
            text: text.into(),
            position,
            style,
            bounds_width: None,
        }));
    }

    pub fn measure_text(&mut self, text: &str, font_size: f32) -> f32 {
        let mut buffer = Buffer::new(
            &mut self.text_brush.font_system,
            Metrics::new(font_size, font_size * 1.2),
        );
        buffer.set_size(&mut self.text_brush.font_system, f32::MAX, font_size * 2.0);
        buffer.set_text(
            &mut self.text_brush.font_system,
            text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.text_brush.font_system);

        let mut width = 0.0f32;
        for line in buffer.layout_runs() {
            width = width.max(line.line_w);
        }
        width
    }

    pub fn draw_rich_text(
        &mut self,
        text: String,
        attrs: AttrsList,
        position: Vec2,
        bounds_width: Option<f32>,
        default_color: Color,
        line_height: f32,
    ) {
        self.push_command(RenderOp::Text(TextCommand::Rich {
            text,
            attrs,
            position,
            bounds_width,
            default_color,
            line_height,
        }));
    }

    pub fn draw_text_bounded(
        &mut self,
        text: impl Into<String>,
        position: Vec2,
        max_width: f32,
        style: TextStyle,
    ) {
        self.push_command(RenderOp::Text(TextCommand::Simple {
            text: text.into(),
            position,
            style,
            bounds_width: Some(max_width),
        }));
    }

    // --- Overlay Methods (Wrapper for Z-Index) ---

    pub fn draw_overlay_rect(&mut self, rect: Rect, color: Color) {
        let old_z = self.current_z_index;
        self.current_z_index = 1000;
        self.fill_rect(rect, color);
        self.current_z_index = old_z;
    }

    pub fn draw_overlay_text(&mut self, text: impl Into<String>, position: Vec2, style: TextStyle) {
        let old_z = self.current_z_index;
        self.current_z_index = 1000;
        self.draw_text(text, position, style);
        self.current_z_index = old_z;
    }

    pub fn draw_overlay_text_bounded(
        &mut self,
        text: impl Into<String>,
        position: Vec2,
        max_width: f32,
        style: TextStyle,
    ) {
        let old_z = self.current_z_index;
        self.current_z_index = 1000;
        self.draw_text_bounded(text, position, max_width, style);
        self.current_z_index = old_z;
    }

    pub fn draw_overlay_style_rect(&mut self, rect: Rect, style: &Style) {
        let old_z = self.current_z_index;
        self.current_z_index = 1000;
        self.draw_style_rect(rect, style);
        self.current_z_index = old_z;
    }

    // --- END FRAME EXECUTION ---

    pub fn end_frame(&mut self, view: &wgpu::TextureView, clear_color: Color) {
        // 1. Sort commands
        self.commands
            .sort_by(|a, b| a.z_index.cmp(&b.z_index).then(a.order.cmp(&b.order)));

        // 2. Prepare CPU buffers
        let mut cpu_vertices: Vec<Vertex> = Vec::with_capacity(INITIAL_VERTEX_CAPACITY);
        let mut cpu_indices: Vec<u32> = Vec::with_capacity(INITIAL_INDEX_CAPACITY);

        let mut current_texture = self.white_texture;

        enum ExecStep {
            DrawGeometry {
                index_range: std::ops::Range<u32>,
                texture_id: TextureId,
            },
            DrawText(Vec<(Buffer, Vec2, Color)>),
            SetClip(Rect),
            ClearClip,
        }

        let mut steps = Vec::new();
        let mut text_accum: Vec<(Buffer, Vec2, Color)> = Vec::new();
        let mut geo_start_index = 0u32;

        for cmd in &self.commands {
            match &cmd.op {
                RenderOp::Geometry {
                    vertices,
                    indices,
                    texture_id,
                } => {
                    if !text_accum.is_empty() {
                        steps.push(ExecStep::DrawText(std::mem::take(&mut text_accum)));
                    }
                    if *texture_id != current_texture {
                        let current_indices_len = cpu_indices.len() as u32;
                        if current_indices_len > geo_start_index {
                            steps.push(ExecStep::DrawGeometry {
                                index_range: geo_start_index..current_indices_len,
                                texture_id: current_texture,
                            });
                            geo_start_index = current_indices_len;
                        }
                        current_texture = *texture_id;
                    }

                    let v_offset = cpu_vertices.len() as u32;
                    cpu_vertices.extend_from_slice(vertices);
                    for &idx in indices {
                        cpu_indices.push(v_offset + idx);
                    }
                }
                RenderOp::Text(text_cmd) => {
                    let current_indices_len = cpu_indices.len() as u32;
                    if current_indices_len > geo_start_index {
                        steps.push(ExecStep::DrawGeometry {
                            index_range: geo_start_index..current_indices_len,
                            texture_id: current_texture,
                        });
                        geo_start_index = current_indices_len;
                    }

                    match text_cmd {
                        TextCommand::Simple {
                            text,
                            position,
                            style,
                            bounds_width,
                        } => {
                            let mut buffer = Buffer::new(
                                &mut self.text_brush.font_system,
                                Metrics::new(style.font_size, style.font_size * 1.2),
                            );
                            let width = bounds_width.unwrap_or(self.screen_width - position.x);
                            buffer.set_size(
                                &mut self.text_brush.font_system,
                                width,
                                self.screen_height,
                            );
                            buffer.set_text(
                                &mut self.text_brush.font_system,
                                text,
                                Attrs::new().family(Family::SansSerif),
                                Shaping::Advanced,
                            );
                            buffer.shape_until_scroll(&mut self.text_brush.font_system);
                            text_accum.push((buffer, *position, style.color));
                        }
                        TextCommand::Rich {
                            text,
                            attrs,
                            position,
                            bounds_width,
                            default_color,
                            line_height,
                        } => {
                            let mut buffer = Buffer::new(
                                &mut self.text_brush.font_system,
                                Metrics::new(*line_height, *line_height * 1.2),
                            );
                            let width = bounds_width.unwrap_or(self.screen_width - position.x);
                            buffer.set_size(
                                &mut self.text_brush.font_system,
                                width,
                                self.screen_height,
                            );
                            buffer.set_text(
                                &mut self.text_brush.font_system,
                                text,
                                Attrs::new().family(Family::SansSerif),
                                Shaping::Advanced,
                            );
                            if !buffer.lines.is_empty() {
                                for line in &mut buffer.lines {
                                    line.set_attrs_list(attrs.clone());
                                }
                            }
                            buffer.shape_until_scroll(&mut self.text_brush.font_system);
                            text_accum.push((buffer, *position, *default_color));
                        }
                    }
                }
                RenderOp::PushClip(rect) => {
                    let current_indices_len = cpu_indices.len() as u32;
                    if current_indices_len > geo_start_index {
                        steps.push(ExecStep::DrawGeometry {
                            index_range: geo_start_index..current_indices_len,
                            texture_id: current_texture,
                        });
                        geo_start_index = current_indices_len;
                    }
                    if !text_accum.is_empty() {
                        steps.push(ExecStep::DrawText(std::mem::take(&mut text_accum)));
                    }
                    steps.push(ExecStep::SetClip(*rect));
                }
                RenderOp::PopClip => {
                    let current_indices_len = cpu_indices.len() as u32;
                    if current_indices_len > geo_start_index {
                        steps.push(ExecStep::DrawGeometry {
                            index_range: geo_start_index..current_indices_len,
                            texture_id: current_texture,
                        });
                        geo_start_index = current_indices_len;
                    }
                    if !text_accum.is_empty() {
                        steps.push(ExecStep::DrawText(std::mem::take(&mut text_accum)));
                    }
                    steps.push(ExecStep::ClearClip);
                }
            }
        }

        let current_indices_len = cpu_indices.len() as u32;
        if current_indices_len > geo_start_index {
            steps.push(ExecStep::DrawGeometry {
                index_range: geo_start_index..current_indices_len,
                texture_id: current_texture,
            });
        }
        if !text_accum.is_empty() {
            steps.push(ExecStep::DrawText(std::mem::take(&mut text_accum)));
        }

        // 3. Upload Buffers
        if cpu_vertices.len() > self.vertex_capacity {
            self.vertex_capacity = cpu_vertices.len().next_power_of_two();
            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: (self.vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if cpu_indices.len() > self.index_capacity {
            self.index_capacity = cpu_indices.len().next_power_of_two();
            self.index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Index Buffer"),
                size: (self.index_capacity * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        if !cpu_vertices.is_empty() {
            self.queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&cpu_vertices));
        }
        if !cpu_indices.is_empty() {
            self.queue
                .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&cpu_indices));
        }

        // 5. Execute Steps (Chunked Submission)
        {
            // Destructure outside loop
            let TextBrush {
                font_system,
                swash_cache,
                text_renderer,
                text_atlas,
            } = &mut self.text_brush;

            let mut step_iter = steps.into_iter().peekable();
            let mut runtime_clip_stack: Vec<Rect> = Vec::new();
            let mut first_pass = true;

            while let Some(peek_step) = step_iter.peek() {
                // Determine step type without consuming
                let is_text = matches!(peek_step, ExecStep::DrawText(_));

                let mut encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("OxidX Chunk Encoder"),
                        });

                if is_text {
                    // Handle Text Step (Submission)
                    if let Some(ExecStep::DrawText(list)) = step_iter.next() {
                        let step_areas: Vec<TextArea> = list
                            .iter()
                            .map(|(buffer, pos, color)| TextArea {
                                buffer,
                                left: pos.x,
                                top: pos.y,
                                scale: 1.0,
                                bounds: TextBounds {
                                    left: pos.x as i32,
                                    top: pos.y as i32,
                                    right: (pos.x + buffer.size().0) as i32,
                                    bottom: self.screen_height as i32,
                                },
                                default_color: glyphon::Color::rgba(
                                    (color.r * 255.0) as u8,
                                    (color.g * 255.0) as u8,
                                    (color.b * 255.0) as u8,
                                    (color.a * 255.0) as u8,
                                ),
                            })
                            .collect();

                        let _ = text_renderer.prepare(
                            &self.device,
                            &self.queue,
                            font_system,
                            text_atlas,
                            Resolution {
                                width: self.screen_width as u32,
                                height: self.screen_height as u32,
                            },
                            step_areas,
                            swash_cache,
                        );

                        // Start Text Pass
                        let load_op = if first_pass {
                            wgpu::LoadOp::Clear(wgpu::Color {
                                r: clear_color.r as f64,
                                g: clear_color.g as f64,
                                b: clear_color.b as f64,
                                a: clear_color.a as f64,
                            })
                        } else {
                            wgpu::LoadOp::Load
                        };
                        first_pass = false;

                        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("OxidX Text Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: load_op,
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            occlusion_query_set: None,
                            timestamp_writes: None,
                        });

                        // Apply Scissor
                        if let Some(rect) = runtime_clip_stack.last() {
                            let sf = self.scale_factor as f32;
                            let x = (rect.x * sf).max(0.0) as u32;
                            let y = (rect.y * sf).max(0.0) as u32;
                            let w = (rect.width * sf).max(1.0) as u32;
                            let h = (rect.height * sf).max(1.0) as u32;
                            pass.set_scissor_rect(x, y, w, h);
                        } else {
                            pass.set_scissor_rect(0, 0, self.physical_width, self.physical_height);
                        }

                        let _ = text_renderer.render(text_atlas, &mut pass);
                    }
                } else {
                    // Handle Geometry/Clip Batch (Submission)
                    let load_op = if first_pass {
                        wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color.r as f64,
                            g: clear_color.g as f64,
                            b: clear_color.b as f64,
                            a: clear_color.a as f64,
                        })
                    } else {
                        wgpu::LoadOp::Load
                    };
                    first_pass = false;

                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("OxidX Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: load_op,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    // Set global state
                    pass.set_pipeline(&self.pipeline);
                    pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                    pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                    // Apply Scissor (Initial)
                    if let Some(rect) = runtime_clip_stack.last() {
                        let sf = self.scale_factor as f32;
                        let x = (rect.x * sf).max(0.0) as u32;
                        let y = (rect.y * sf).max(0.0) as u32;
                        let w = (rect.width * sf).max(1.0) as u32;
                        let h = (rect.height * sf).max(1.0) as u32;
                        pass.set_scissor_rect(x, y, w, h);
                    } else {
                        pass.set_scissor_rect(0, 0, self.physical_width, self.physical_height);
                    }

                    // Execute batch until next Text
                    while let Some(peek_step) = step_iter.peek() {
                        if matches!(peek_step, ExecStep::DrawText(_)) {
                            break;
                        }
                        // Consume non-text step
                        let step = step_iter.next().unwrap();
                        match step {
                            ExecStep::DrawGeometry {
                                index_range,
                                texture_id,
                            } => {
                                if let Some(bind_group) = self.texture_bind_groups.get(&texture_id)
                                {
                                    pass.set_bind_group(1, bind_group, &[]);
                                    pass.draw_indexed(index_range, 0, 0..1);
                                }
                            }
                            ExecStep::SetClip(rect) => {
                                runtime_clip_stack.push(rect);
                                let sf = self.scale_factor as f32;
                                let x = (rect.x * sf).max(0.0) as u32;
                                let y = (rect.y * sf).max(0.0) as u32;
                                let w = (rect.width * sf).max(1.0) as u32;
                                let h = (rect.height * sf).max(1.0) as u32;
                                pass.set_scissor_rect(x, y, w, h);
                            }
                            ExecStep::ClearClip => {
                                runtime_clip_stack.pop();
                                if let Some(rect) = runtime_clip_stack.last() {
                                    let sf = self.scale_factor as f32;
                                    let x = (rect.x * sf).max(0.0) as u32;
                                    let y = (rect.y * sf).max(0.0) as u32;
                                    let w = (rect.width * sf).max(1.0) as u32;
                                    let h = (rect.height * sf).max(1.0) as u32;
                                    pass.set_scissor_rect(x, y, w, h);
                                } else {
                                    pass.set_scissor_rect(
                                        0,
                                        0,
                                        self.physical_width,
                                        self.physical_height,
                                    );
                                }
                            }
                            _ => {} // Text handled outside
                        }
                    }
                }

                // Submit this chunk
                self.queue.submit(std::iter::once(encoder.finish()));
            }
        }

        self.text_brush.text_atlas.trim();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Rect;

    #[test]
    fn test_clip_stack() {
        let mut stack = ClipStack::new();
        assert!(stack.is_empty());
        let rect_a = Rect::new(0.0, 0.0, 100.0, 100.0);
        stack.push(rect_a);
        assert_eq!(stack.current(), Some(rect_a));
        stack.pop();
        assert!(stack.is_empty());
    }
}
