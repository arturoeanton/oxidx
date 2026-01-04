//! # OxidX Renderer
//!
//! A batched 2D renderer that abstracts WGPU complexity from components.
//! Components call methods like `fill_rect()` and `draw_text()`, and the
//! renderer batches all primitives for efficient GPU rendering.
//!
//! ## Batching Algorithm
//!
//! 1. `begin_frame()`: Clear vertex/index arrays, prepare for new frame
//! 2. Components call `fill_rect()`, `draw_text()`, etc. - appends to arrays
//! 3. `end_frame()`: Upload all batched data and issue draw calls
//!
//! This approach minimizes GPU state changes and buffer uploads.

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

#[derive(Debug, Clone, Copy)]
struct DrawBatch {
    texture_id: TextureId,
    index_start: u32,
    index_count: u32,
}

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

/// Text command queued for rendering.
enum TextCommand {
    /// Simple string (legacy/easy API)
    Simple {
        text: String,
        position: Vec2,
        style: TextStyle,
        bounds_width: Option<f32>,
    },
    /// Rich text with attributes
    Rich {
        text: String,
        attrs: AttrsList,
        position: Vec2,
        bounds_width: Option<f32>,
        default_color: Color,
        line_height: f32,
    },
}

/// Initial capacity for vertex/index buffers.
const INITIAL_VERTEX_CAPACITY: usize = 1024;
const INITIAL_INDEX_CAPACITY: usize = 1536; // 1.5x vertices for quads

/// The main batched 2D renderer.
///
/// Collects draw commands from components and renders them efficiently.
/// Uses orthographic projection so components work in pixel coordinates.
pub struct Renderer {
    // GPU resources
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    _surface_format: wgpu::TextureFormat,

    // Global theme for rendering components
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

    // CPU-side batching arrays
    vertices: Vec<Vertex>,
    indices: Vec<u32>,

    // Text rendering
    pub text_brush: TextBrush,
    text_commands: Vec<TextCommand>,

    // Overlay layer (rendered on top, no clipping)
    overlay_vertices: Vec<Vertex>,
    overlay_indices: Vec<u32>,
    overlay_text_commands: Vec<TextCommand>,

    // Batches
    batches: Vec<DrawBatch>,
    texture_bind_groups: HashMap<TextureId, wgpu::BindGroup>,
    // Cache of loaded assets (path -> texture_id)
    assets: HashMap<String, TextureId>,
    white_texture: TextureId,
    next_texture_id: TextureId,
    texture_layout: wgpu::BindGroupLayout,

    // Screen size for projection
    screen_width: f32,
    screen_height: f32,

    // Scissor clipping stack
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
/// Wraps cosmic-text and glyphon.
pub struct TextBrush {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub text_renderer: GlyphonTextRenderer,
    pub text_atlas: glyphon::TextAtlas,
    // Cache for simple text commands to avoid reallocation every frame
    // In a real engine we'd use a proper LRU cache or handle IDs
    // For now we just use it for the current frame's batch
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
    /// Creates a new Renderer.
    ///
    /// # Arguments
    /// * `device` - WGPU device
    /// * `queue` - WGPU queue
    /// * `surface_format` - Format of the render target
    /// * `width` - Initial screen width in pixels
    /// * `height` - Initial screen height in pixels
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("OxidX Batched Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_batched.wgsl").into()),
        });

        // Create uniform buffer for projection matrix
        let projection = Self::create_orthographic_projection(width as f32, height as f32);
        let uniforms = Uniforms {
            projection: projection.to_cols_array_2d(),
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout and bind group
        // Create bind group layout and bind group (Uniforms)
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

        // Create texture bind group layout (Group 1)
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                // Texture
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
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create default white texture for non-textured rendering
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

        // Create render pipeline
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

        // Create initial vertex/index buffers
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

        // Initialize TextBrush
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
            vertices: Vec::with_capacity(INITIAL_VERTEX_CAPACITY),
            indices: Vec::with_capacity(INITIAL_INDEX_CAPACITY),
            text_brush,
            text_commands: Vec::new(),
            overlay_vertices: Vec::new(),
            overlay_indices: Vec::new(),
            overlay_text_commands: Vec::new(),

            batches: Vec::new(),
            texture_bind_groups,
            assets: HashMap::new(),
            white_texture: white_id,
            next_texture_id: 1,
            texture_layout,

            screen_width: width as f32,
            screen_height: height as f32,
            clip_stack: ClipStack::new(),
        }
    }

    /// Creates an orthographic projection matrix.
    /// Maps pixel coordinates to clip space:
    /// - (0, 0) at top-left
    /// - (width, height) at bottom-right
    fn create_orthographic_projection(width: f32, height: f32) -> Mat4 {
        // left=0, right=width, bottom=height, top=0, near=-1, far=1
        Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0)
    }

    /// Updates the projection matrix when the window is resized.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.resize_with_scale(width, height, 1.0);
    }

    /// Updates the projection matrix with DPI scaling.
    ///
    /// - `width`, `height`: Physical pixel size of the surface
    /// - `scale_factor`: Display scale factor (1.0 = normal, 2.0 = Retina)
    ///
    /// The projection matrix uses logical coordinates (physical / scale_factor)
    /// so components can draw in consistent logical units regardless of DPI.
    /// The viewport uses physical pixels for crisp rendering.
    pub fn resize_with_scale(&mut self, width: u32, height: u32, scale_factor: f64) {
        // Store logical size for component layout
        self.screen_width = width as f32 / scale_factor as f32;
        self.screen_height = height as f32 / scale_factor as f32;

        // Projection uses logical size so components draw in logical coordinates
        let projection =
            Self::create_orthographic_projection(self.screen_width, self.screen_height);
        let uniforms = Uniforms {
            projection: projection.to_cols_array_2d(),
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Returns the current screen size (in logical points).
    pub fn screen_size(&self) -> Vec2 {
        Vec2::new(self.screen_width, self.screen_height)
    }

    /// Begins a new frame. Clears all batched data.
    pub fn begin_frame(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.text_commands.clear();
        self.overlay_vertices.clear();
        self.overlay_indices.clear();
        self.overlay_text_commands.clear();
        self.overlay_indices.clear();
        self.overlay_text_commands.clear();
        self.clip_stack.clear();
        self.batches.clear();
    }

    /// Pushes a clip rectangle onto the clipping stack.
    ///
    /// Content rendered after this call will be clipped to the intersection
    /// of all active clip rectangles. Call `pop_clip()` to restore the previous
    /// clipping state.
    ///
    /// # Arguments
    /// * `rect` - The clipping rectangle in pixel coordinates
    pub fn push_clip(&mut self, rect: Rect) {
        self.clip_stack.push(rect);
    }

    /// Pops the most recent clip rectangle from the stack.
    ///
    /// Restores the previous clipping state. If the stack is empty,
    /// this is a no-op.
    pub fn pop_clip(&mut self) {
        if self.clip_stack.pop().is_none() {
            log::warn!("Clip stack underflow! pop_clip() called on empty stack.");
        }
    }

    /// Returns the current clip rectangle, if any.
    pub fn current_clip(&self) -> Option<Rect> {
        self.clip_stack.current()
    }

    /// Switches rendering batch if necessary.
    fn ensure_batch(&mut self, texture_id: TextureId) {
        if let Some(current) = self.batches.last() {
            if current.texture_id == texture_id {
                return;
            }
        }

        // Start new batch
        self.batches.push(DrawBatch {
            texture_id,
            index_start: self.indices.len() as u32,
            index_count: 0,
        });
    }

    /// Creates a texture from an image.
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

    /// Loads an image from disk and creates a GPU texture.
    /// Returns the cached TextureId if already loaded.
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

    /// Draws a textured rectangle (Image).
    pub fn draw_image(&mut self, rect: Rect, texture_id: TextureId) {
        self.ensure_batch(texture_id);

        let base_index = self.vertices.len() as u32;

        self.vertices.push(Vertex {
            position: [rect.x, rect.y],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [0.0, 0.0],
        });
        self.vertices.push(Vertex {
            position: [rect.x + rect.width, rect.y],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [1.0, 0.0],
        });
        self.vertices.push(Vertex {
            position: [rect.x + rect.width, rect.y + rect.height],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [1.0, 1.0],
        });
        self.vertices.push(Vertex {
            position: [rect.x, rect.y + rect.height],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [0.0, 1.0],
        });

        self.indices.push(base_index);
        self.indices.push(base_index + 1);
        self.indices.push(base_index + 2);
        self.indices.push(base_index);
        self.indices.push(base_index + 2);
        self.indices.push(base_index + 3);

        if let Some(batch) = self.batches.last_mut() {
            batch.index_count += 6;
        }
    }

    /// Draws a filled rectangle.
    ///
    /// # Arguments
    /// * `rect` - Rectangle bounds in pixels
    /// * `color` - Fill color
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.ensure_batch(self.white_texture);
        let base_index = self.vertices.len() as u32;

        // Four corners of the rectangle
        self.vertices.push(Vertex::new(rect.x, rect.y, color)); // top-left
        self.vertices
            .push(Vertex::new(rect.x + rect.width, rect.y, color)); // top-right
        self.vertices.push(Vertex::new(
            rect.x + rect.width,
            rect.y + rect.height,
            color,
        )); // bottom-right
        self.vertices
            .push(Vertex::new(rect.x, rect.y + rect.height, color)); // bottom-left

        // Two triangles: 0-1-2 and 0-2-3
        self.indices.push(base_index);
        self.indices.push(base_index + 1);
        self.indices.push(base_index + 2);
        self.indices.push(base_index);
        self.indices.push(base_index + 2);
        self.indices.push(base_index + 3);

        if let Some(batch) = self.batches.last_mut() {
            batch.index_count += 6;
        }
    }

    /// Draws a stroked (outlined) rectangle.
    ///
    /// # Arguments
    /// * `rect` - Rectangle bounds in pixels
    /// * `color` - Stroke color
    /// * `thickness` - Line thickness in pixels
    pub fn stroke_rect(&mut self, rect: Rect, color: Color, thickness: f32) {
        let half = thickness / 2.0;

        // Top edge
        self.fill_rect(
            Rect::new(
                rect.x - half,
                rect.y - half,
                rect.width + thickness,
                thickness,
            ),
            color,
        );
        // Bottom edge
        self.fill_rect(
            Rect::new(
                rect.x - half,
                rect.y + rect.height - half,
                rect.width + thickness,
                thickness,
            ),
            color,
        );
        // Left edge
        self.fill_rect(
            Rect::new(
                rect.x - half,
                rect.y + half,
                thickness,
                rect.height - thickness,
            ),
            color,
        );
        // Right edge
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

    /// Draws a line between two points.
    pub fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color, width: f32) {
        self.ensure_batch(self.white_texture);
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

        let base_index = self.vertices.len() as u32;
        self.vertices.push(Vertex::new(p1.x, p1.y, color));
        self.vertices.push(Vertex::new(p2.x, p2.y, color));
        self.vertices.push(Vertex::new(p3.x, p3.y, color));
        self.vertices.push(Vertex::new(p4.x, p4.y, color));

        self.indices.push(base_index);
        self.indices.push(base_index + 1);
        self.indices.push(base_index + 2);
        self.indices.push(base_index);
        self.indices.push(base_index + 2);
        self.indices.push(base_index + 3);

        if let Some(batch) = self.batches.last_mut() {
            batch.index_count += 6;
        }
    }

    /// Draws a rounded rectangle.
    ///
    /// # Arguments
    /// * `rect` - Rectangle bounds in pixels
    /// * `color` - Fill color
    /// * `radius` - Corner radius in pixels
    /// * `border_color` - Optional border color
    /// * `border_width` - Optional border width
    pub fn draw_rounded_rect(
        &mut self,
        rect: Rect,
        color: Color,
        _radius: f32,
        border_color: Option<Color>,
        border_width: Option<f32>,
    ) {
        // TODO: Implement actual rounded corners using UVs or geometry
        // For now, render as a standard rectangle
        self.fill_rect(rect, color);
        if let Some(bc) = border_color {
            if let Some(bw) = border_width {
                self.stroke_rect(rect, bc, bw);
            }
        }
    }

    /// Draws a styled rectangle (with background, border, shadow).
    ///
    /// # Arguments
    /// * `rect` - Rectangle bounds in pixels
    /// * `style` - Visual style configuration
    pub fn draw_style_rect(&mut self, rect: Rect, style: &Style) {
        // 1. Draw Shadow (Simulated)
        if let Some(shadow) = &style.shadow {
            // Render a rect offset by shadow.offset
            // For Phase 1 simulation, we just draw a semi-transparent rect
            let shadow_rect = Rect::new(
                rect.x + shadow.offset.x,
                rect.y + shadow.offset.y,
                rect.width,
                rect.height,
            );
            self.fill_rect(shadow_rect, shadow.color);
        }

        // 2. Draw Border (Simulated as stroke or larger rect behind)
        if let Some(border) = &style.border {
            if border.width > 0.0 {
                // Draw a larger rect behind
                let border_rect = Rect::new(
                    rect.x - border.width,
                    rect.y - border.width,
                    rect.width + border.width * 2.0,
                    rect.height + border.width * 2.0,
                );
                self.fill_rect(border_rect, border.color);
            }
        }

        // 3. Draw Background
        match style.background {
            Background::Solid(color) => {
                self.fill_rect(rect, color);
            }
            Background::LinearGradient { start, end, .. } => {
                // Phase 1 Simulation: Average color
                let r = (start.r + end.r) / 2.0;
                let g = (start.g + end.g) / 2.0;
                let b = (start.b + end.b) / 2.0;
                let a = (start.a + end.a) / 2.0;
                self.fill_rect(rect, Color::new(r, g, b, a));
            }
        }
    }

    /// Queues simple text for rendering.
    pub fn draw_text(&mut self, text: impl Into<String>, position: Vec2, style: TextStyle) {
        self.text_commands.push(TextCommand::Simple {
            text: text.into(),
            position,
            style,
            bounds_width: None,
        });
    }

    /// Measures the pixel width of a text string with the given style.
    ///
    /// # Arguments
    /// * `text` - The text to measure
    /// * `font_size` - Font size in pixels
    ///
    /// # Returns
    /// The width in pixels
    pub fn measure_text(&mut self, text: &str, font_size: f32) -> f32 {
        use glyphon::cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping};

        // Create a temporary buffer for measurement
        let mut buffer = Buffer::new(
            &mut self.text_brush.font_system,
            Metrics::new(font_size, font_size * 1.2),
        );

        // Set a large width so text doesn't wrap
        buffer.set_size(&mut self.text_brush.font_system, f32::MAX, font_size * 2.0);

        // Set the text
        buffer.set_text(
            &mut self.text_brush.font_system,
            text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );

        // Shape the text
        buffer.shape_until_scroll(&mut self.text_brush.font_system);

        // Calculate width from layout runs
        let mut width = 0.0f32;
        for line in buffer.layout_runs() {
            width = width.max(line.line_w);
        }

        width
    }

    /// Queues rich text (using cosmic-text AttrsList) for rendering.
    pub fn draw_rich_text(
        &mut self,
        text: String,
        attrs: AttrsList,
        position: Vec2,
        bounds_width: Option<f32>,
        default_color: Color,
        line_height: f32,
    ) {
        self.text_commands.push(TextCommand::Rich {
            text,
            attrs,
            position,
            bounds_width,
            default_color,
            line_height,
        });
    }

    /// Queues text for rendering with bounded width.
    ///
    /// # Arguments
    /// * `text` - The text to render
    /// * `position` - Position in pixels
    /// * `max_width` - Maximum width for text wrapping
    /// * `style` - Text style
    pub fn draw_text_bounded(
        &mut self,
        text: impl Into<String>,
        position: Vec2,
        max_width: f32,
        style: TextStyle,
    ) {
        self.text_commands.push(TextCommand::Simple {
            text: text.into(),
            position,
            style,
            bounds_width: Some(max_width),
        });
    }

    // ========================================================================
    // OVERLAY LAYER (rendered on top, no clipping)
    // ========================================================================

    /// Draws a filled rectangle on the overlay layer.
    /// Overlay content is rendered after the main pass with no scissor clipping.
    pub fn draw_overlay_rect(&mut self, rect: Rect, color: Color) {
        let base_index = self.overlay_vertices.len() as u32;

        self.overlay_vertices
            .push(Vertex::new(rect.x, rect.y, color));
        self.overlay_vertices
            .push(Vertex::new(rect.x + rect.width, rect.y, color));
        self.overlay_vertices.push(Vertex::new(
            rect.x + rect.width,
            rect.y + rect.height,
            color,
        ));
        self.overlay_vertices
            .push(Vertex::new(rect.x, rect.y + rect.height, color));

        self.overlay_indices.push(base_index);
        self.overlay_indices.push(base_index + 1);
        self.overlay_indices.push(base_index + 2);
        self.overlay_indices.push(base_index);
        self.overlay_indices.push(base_index + 2);
        self.overlay_indices.push(base_index + 3);
    }

    /// Draws text on the overlay layer.
    /// Overlay content is rendered after the main pass with no scissor clipping.
    pub fn draw_overlay_text(&mut self, text: impl Into<String>, position: Vec2, style: TextStyle) {
        self.overlay_text_commands.push(TextCommand::Simple {
            text: text.into(),
            position,
            style,
            bounds_width: None,
        });
    }

    /// Draws a styled rectangle on the overlay layer.
    pub fn draw_overlay_style_rect(&mut self, rect: Rect, style: &Style) {
        // Shadow
        if let Some(shadow) = &style.shadow {
            let shadow_rect = Rect::new(
                rect.x + shadow.offset.x,
                rect.y + shadow.offset.y,
                rect.width,
                rect.height,
            );
            self.draw_overlay_rect(shadow_rect, shadow.color);
        }
        // Border
        if let Some(border) = &style.border {
            if border.width > 0.0 {
                let border_rect = Rect::new(
                    rect.x - border.width,
                    rect.y - border.width,
                    rect.width + border.width * 2.0,
                    rect.height + border.width * 2.0,
                );
                self.draw_overlay_rect(border_rect, border.color);
            }
        }
        // Background
        match style.background {
            Background::Solid(color) => {
                self.draw_overlay_rect(rect, color);
            }
            Background::LinearGradient { start, end, .. } => {
                let r = (start.r + end.r) / 2.0;
                let g = (start.g + end.g) / 2.0;
                let b = (start.b + end.b) / 2.0;
                let a = (start.a + end.a) / 2.0;
                self.draw_overlay_rect(rect, Color::new(r, g, b, a));
            }
        }
    }

    /// Ends the frame and renders all batched content.
    ///
    /// # Arguments
    /// * `encoder` - Command encoder to record draw calls
    /// * `view` - Texture view to render to
    /// * `clear_color` - Background clear color
    pub fn end_frame(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        clear_color: Color,
    ) {
        // Ensure buffers are large enough
        self.ensure_buffer_capacity();

        // Upload main layer vertex data
        if !self.vertices.is_empty() {
            self.queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        }

        // Upload main layer index data
        if !self.indices.is_empty() {
            self.queue
                .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));
        }

        // Prepare main layer text for rendering
        self.prepare_text();

        // Prepare overlay text for rendering (before render pass to avoid borrow issues)
        self.prepare_overlay_text();

        // Begin render pass (main layer + overlay layer in same pass)
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("OxidX Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color.r as f64,
                            g: clear_color.g as f64,
                            b: clear_color.b as f64,
                            a: clear_color.a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // =============================================
            // MAIN PASS (subject to scissor clipping)
            // =============================================
            if !self.batches.is_empty() {
                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                for batch in &self.batches {
                    if let Some(bind_group) = self.texture_bind_groups.get(&batch.texture_id) {
                        render_pass.set_bind_group(1, bind_group, &[]);
                        render_pass.draw_indexed(
                            batch.index_start..(batch.index_start + batch.index_count),
                            0,
                            0..1,
                        );
                    }
                }
            }

            // Render main layer text
            let _ = self
                .text_brush
                .text_renderer
                .render(&self.text_brush.text_atlas, &mut render_pass);

            // =============================================
            // OVERLAY PASS (no scissor clipping)
            // =============================================
            if !self.overlay_indices.is_empty() {
                // Upload overlay vertex/index data
                // Note: We reuse the same buffers for simplicity, overwriting main data
                // A production engine would use separate buffers or offset
                self.queue.write_buffer(
                    &self.vertex_buffer,
                    0,
                    bytemuck::cast_slice(&self.overlay_vertices),
                );
                self.queue.write_buffer(
                    &self.index_buffer,
                    0,
                    bytemuck::cast_slice(&self.overlay_indices),
                );

                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.overlay_indices.len() as u32, 0, 0..1);
            }

            // Note: Overlay text was prepared before render pass, just render it
            let _ = self
                .text_brush
                .text_renderer
                .render(&self.text_brush.text_atlas, &mut render_pass);
        }

        // Trim the text atlas periodically
        self.text_brush.text_atlas.trim();
    }

    /// Prepares text commands for rendering.
    fn prepare_text(&mut self) {
        if self.text_commands.is_empty() {
            return;
        }

        let mut buffers = Vec::new();

        // 1. Create buffers for all commands
        // Note: In a real implementation we would cache these buffers in TextBrush
        // using IDs or frames, but for simplicity we recreate them.
        for cmd in &self.text_commands {
            match cmd {
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
                    buffer.set_size(&mut self.text_brush.font_system, width, self.screen_height);

                    buffer.set_text(
                        &mut self.text_brush.font_system,
                        text,
                        Attrs::new().family(Family::SansSerif),
                        Shaping::Advanced,
                    );

                    buffer.shape_until_scroll(&mut self.text_brush.font_system);
                    buffers.push((buffer, *position, style.color));
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
                    buffer.set_size(&mut self.text_brush.font_system, width, self.screen_height);

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
                    buffers.push((buffer, *position, *default_color));
                }
            }
        }

        // 2. Prepare text areas
        let text_areas: Vec<TextArea> = buffers
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

        // 3. Submit to glyphon
        let _ = self.text_brush.text_renderer.prepare(
            &self.device,
            &self.queue,
            &mut self.text_brush.font_system,
            &mut self.text_brush.text_atlas,
            Resolution {
                width: self.screen_width as u32,
                height: self.screen_height as u32,
            },
            text_areas,
            &mut self.text_brush.swash_cache,
        );
    }

    /// Prepares overlay text commands for rendering.
    fn prepare_overlay_text(&mut self) {
        if self.overlay_text_commands.is_empty() {
            return;
        }

        let mut buffers = Vec::new();

        for cmd in &self.overlay_text_commands {
            match cmd {
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
                    buffer.set_size(&mut self.text_brush.font_system, width, self.screen_height);

                    buffer.set_text(
                        &mut self.text_brush.font_system,
                        text,
                        Attrs::new().family(Family::SansSerif),
                        Shaping::Advanced,
                    );

                    buffer.shape_until_scroll(&mut self.text_brush.font_system);
                    buffers.push((buffer, *position, style.color));
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
                    buffer.set_size(&mut self.text_brush.font_system, width, self.screen_height);

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
                    buffers.push((buffer, *position, *default_color));
                }
            }
        }

        let text_areas: Vec<TextArea> = buffers
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

        let _ = self.text_brush.text_renderer.prepare(
            &self.device,
            &self.queue,
            &mut self.text_brush.font_system,
            &mut self.text_brush.text_atlas,
            Resolution {
                width: self.screen_width as u32,
                height: self.screen_height as u32,
            },
            text_areas,
            &mut self.text_brush.swash_cache,
        );
    }

    /// Ensures vertex and index buffers are large enough.
    fn ensure_buffer_capacity(&mut self) {
        // Check if vertex buffer needs to grow
        if self.vertices.len() > self.vertex_capacity {
            self.vertex_capacity = self.vertices.len().next_power_of_two();
            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: (self.vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // Check if index buffer needs to grow
        if self.indices.len() > self.index_capacity {
            self.index_capacity = self.indices.len().next_power_of_two();
            self.index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Index Buffer"),
                size: (self.index_capacity * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
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

        // Push Rect A (0,0, 100,100)
        let rect_a = Rect::new(0.0, 0.0, 100.0, 100.0);
        stack.push(rect_a);
        assert_eq!(stack.current(), Some(rect_a));

        // Push Rect B (50,50, 100,100) -> Intersection should be (50,50, 50,50)
        let rect_b = Rect::new(50.0, 50.0, 100.0, 100.0);
        stack.push(rect_b);
        let expected = Rect::new(50.0, 50.0, 50.0, 50.0);
        assert_eq!(stack.current(), Some(expected));

        // Pop
        stack.pop();
        assert_eq!(stack.current(), Some(rect_a));

        // Pop empty
        stack.pop();
        assert!(stack.is_empty());
        assert_eq!(stack.pop(), None); // Safe: should handle underflow without panic
    }
}
