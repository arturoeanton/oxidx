//! # Async Asset Loader
//!
//! Provides non-blocking asset loading (images, fonts) using a thread pool.
//! Loaded assets are cached to avoid duplicate loading.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::renderer::{Renderer, TextureId};
use rayon::ThreadPool;

/// Result of an async image load operation.
pub struct LoadedImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA bytes
}

/// Error type for asset loading.
#[derive(Debug, Clone)]
pub enum AssetError {
    /// Failed to read the file from disk.
    IoError(String),
    /// Failed to decode the image.
    DecodeError(String),
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetError::IoError(e) => write!(f, "IO Error: {}", e),
            AssetError::DecodeError(e) => write!(f, "Decode Error: {}", e),
        }
    }
}

impl std::error::Error for AssetError {}

/// A pending asset load result waiting to be processed on the main thread.
pub struct PendingAsset {
    pub path: String,
    pub result: Result<LoadedImage, AssetError>,
}

/// Async asset loader using a thread pool.
///
/// # Example
/// ```ignore
/// let loader = AssetLoader::new();
/// loader.load_image("icon.png");
///
/// // In update loop
/// for pending in loader.poll_completed() {
///     match pending.result {
///         Ok(img) => { /* Upload to GPU */ }
///         Err(e) => log::error!("Failed to load: {}", e),
///     }
/// }
/// ```
pub struct AssetLoader {
    thread_pool: ThreadPool,
    pending: Arc<Mutex<Vec<PendingAsset>>>,
    /// Cache of loaded textures (path -> texture handle).
    /// The actual wgpu::Texture is managed by the caller since
    /// GPU upload must happen on the main thread.
    texture_cache: HashMap<String, bool>, // path -> loaded flag
}

impl AssetLoader {
    /// Creates a new AssetLoader with a thread pool.
    pub fn new() -> Self {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .expect("Failed to create asset loader thread pool");

        Self {
            thread_pool,
            pending: Arc::new(Mutex::new(Vec::new())),
            texture_cache: HashMap::new(),
        }
    }

    /// Queues an image for async loading.
    ///
    /// # Arguments
    /// * `path` - Path to the image file.
    ///
    /// # Returns
    /// Returns `true` if loading was started, `false` if already cached.
    pub fn load_image(&mut self, path: impl Into<String>) -> bool {
        let path = path.into();

        // Check cache
        if self.texture_cache.contains_key(&path) {
            return false;
        }

        // Mark as loading
        self.texture_cache.insert(path.clone(), false);

        let pending = Arc::clone(&self.pending);
        let path_clone = path.clone();

        self.thread_pool.spawn(move || {
            let result = Self::load_image_sync(&path_clone);
            let mut lock = pending.lock().unwrap();
            lock.push(PendingAsset {
                path: path_clone,
                result,
            });
        });

        true
    }

    /// Synchronously loads an image from disk.
    fn load_image_sync(path: &str) -> Result<LoadedImage, AssetError> {
        let img = image::open(Path::new(path)).map_err(|e| AssetError::IoError(e.to_string()))?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        Ok(LoadedImage {
            width,
            height,
            data: rgba.into_raw(),
        })
    }

    /// Polls for completed asset loads.
    ///
    /// Call this once per frame on the main thread.
    /// Returns completed loads that need GPU upload.
    pub fn poll_completed(&mut self) -> Vec<PendingAsset> {
        let mut lock = self.pending.lock().unwrap();
        let completed: Vec<PendingAsset> = lock.drain(..).collect();

        // Update cache for successful loads
        for asset in &completed {
            if asset.result.is_ok() {
                self.texture_cache.insert(asset.path.clone(), true);
            } else {
                // Remove failed entries so they can be retried
                self.texture_cache.remove(&asset.path);
            }
        }

        completed
    }

    /// Checks if an image is already loaded (cached).
    pub fn is_loaded(&self, path: &str) -> bool {
        self.texture_cache.get(path).copied().unwrap_or(false)
    }

    /// Checks if an image is currently loading.
    pub fn is_loading(&self, path: &str) -> bool {
        self.texture_cache.get(path).copied() == Some(false)
    }
}

impl Default for AssetLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Asset Manager for synchronous loading and caching of textures.
pub struct AssetManager {
    textures: HashMap<String, TextureId>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    /// Loads an image from disk and creates a GPU texture.
    /// Returns the cached TextureId if already loaded.
    pub fn load_image(
        &mut self,
        renderer: &mut Renderer,
        path: &str,
    ) -> Result<TextureId, AssetError> {
        if let Some(&id) = self.textures.get(path) {
            return Ok(id);
        }

        let img = image::open(Path::new(path)).map_err(|e| AssetError::IoError(e.to_string()))?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let loaded = LoadedImage {
            width,
            height,
            data: rgba.into_raw(),
        };

        let id = renderer.create_texture(&loaded, Some(path));
        self.textures.insert(path.to_string(), id);
        Ok(id)
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}
