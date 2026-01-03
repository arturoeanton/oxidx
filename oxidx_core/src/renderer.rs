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

use crate::primitives::{Color, Rect, TextStyle};
use crate::style::{Background, Style};
use glam::{Mat4, Vec2};
use glyphon::{
    Attrs, Buffer, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextBounds, TextRenderer as GlyphonTextRenderer,
};
use std::sync::Arc;
use wgpu::util::DeviceExt;

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
struct TextCommand {
    text: String,
    position: Vec2,
    style: TextStyle,
    bounds_width: Option<f32>,
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
    surface_format: wgpu::TextureFormat,

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

    // Text rendering (glyphon)
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_renderer: GlyphonTextRenderer,
    text_atlas: glyphon::TextAtlas,
    text_commands: Vec<TextCommand>,

    // Screen size for projection
    screen_width: f32,
    screen_height: f32,

    // Scissor clipping stack
    clip_stack: Vec<Rect>,
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

        // Create render pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
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

        // Initialize glyphon for text rendering
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let mut text_atlas = glyphon::TextAtlas::new(&device, &queue, surface_format);
        let text_renderer = GlyphonTextRenderer::new(
            &mut text_atlas,
            &device,
            wgpu::MultisampleState::default(),
            None,
        );

        Self {
            device,
            queue,
            surface_format,
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            vertex_buffer,
            index_buffer,
            vertex_capacity: INITIAL_VERTEX_CAPACITY,
            index_capacity: INITIAL_INDEX_CAPACITY,
            vertices: Vec::with_capacity(INITIAL_VERTEX_CAPACITY),
            indices: Vec::with_capacity(INITIAL_INDEX_CAPACITY),
            font_system,
            swash_cache,
            text_renderer,
            text_atlas,
            text_commands: Vec::new(),
            screen_width: width as f32,
            screen_height: height as f32,
            clip_stack: Vec::new(),
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
        self.screen_width = width as f32;
        self.screen_height = height as f32;

        let projection =
            Self::create_orthographic_projection(self.screen_width, self.screen_height);
        let uniforms = Uniforms {
            projection: projection.to_cols_array_2d(),
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Returns the current screen size.
    pub fn screen_size(&self) -> Vec2 {
        Vec2::new(self.screen_width, self.screen_height)
    }

    /// Begins a new frame. Clears all batched data.
    pub fn begin_frame(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.text_commands.clear();
        self.clip_stack.clear();
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
        // Intersect with current clip if there is one
        let clipped = if let Some(current) = self.clip_stack.last() {
            current.intersect(&rect)
        } else {
            rect
        };
        self.clip_stack.push(clipped);
    }

    /// Pops the most recent clip rectangle from the stack.
    ///
    /// Restores the previous clipping state. If the stack is empty,
    /// this is a no-op.
    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    /// Returns the current clip rectangle, if any.
    pub fn current_clip(&self) -> Option<Rect> {
        self.clip_stack.last().copied()
    }

    /// Draws a filled rectangle.
    ///
    /// # Arguments
    /// * `rect` - Rectangle bounds in pixels
    /// * `color` - Fill color
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
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

    /// Queues text for rendering.
    ///
    /// # Arguments
    /// * `text` - The text to render
    /// * `position` - Position in pixels (top-left of text bounds)
    /// * `style` - Text style (font size, color, alignment)
    pub fn draw_text(&mut self, text: impl Into<String>, position: Vec2, style: TextStyle) {
        self.text_commands.push(TextCommand {
            text: text.into(),
            position,
            style,
            bounds_width: None,
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
        self.text_commands.push(TextCommand {
            text: text.into(),
            position,
            style,
            bounds_width: Some(max_width),
        });
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

        // Upload vertex data
        if !self.vertices.is_empty() {
            self.queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        }

        // Upload index data
        if !self.indices.is_empty() {
            self.queue
                .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));
        }

        // Prepare text for rendering
        self.prepare_text();

        // Begin render pass
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

            // Draw all batched shapes
            if !self.indices.is_empty() {
                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
            }

            // Render text (glyphon 0.5 renders from what was prepared in the atlas)
            let _ = self
                .text_renderer
                .render(&self.text_atlas, &mut render_pass);
        }

        // Trim the text atlas periodically
        self.text_atlas.trim();
    }

    /// Prepares text commands for rendering with glyphon.
    fn prepare_text(&mut self) {
        if self.text_commands.is_empty() {
            return;
        }

        for cmd in &self.text_commands {
            // Create a text buffer for this command
            let mut buffer = Buffer::new(
                &mut self.font_system,
                Metrics::new(cmd.style.font_size, cmd.style.font_size * 1.2),
            );

            let bounds_width = cmd
                .bounds_width
                .unwrap_or(self.screen_width - cmd.position.x);
            buffer.set_size(&mut self.font_system, bounds_width, self.screen_height);

            buffer.set_text(
                &mut self.font_system,
                &cmd.text,
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );

            buffer.shape_until_scroll(&mut self.font_system);

            // Prepare the buffer for rendering
            let _ = self.text_renderer.prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.text_atlas,
                Resolution {
                    width: self.screen_width as u32,
                    height: self.screen_height as u32,
                },
                [TextArea {
                    buffer: &buffer,
                    left: cmd.position.x,
                    top: cmd.position.y,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: cmd.position.x as i32,
                        top: cmd.position.y as i32,
                        right: (cmd.position.x + bounds_width) as i32,
                        bottom: self.screen_height as i32,
                    },
                    default_color: glyphon::Color::rgba(
                        (cmd.style.color.r * 255.0) as u8,
                        (cmd.style.color.g * 255.0) as u8,
                        (cmd.style.color.b * 255.0) as u8,
                        (cmd.style.color.a * 255.0) as u8,
                    ),
                }],
                &mut self.swash_cache,
            );
        }
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
