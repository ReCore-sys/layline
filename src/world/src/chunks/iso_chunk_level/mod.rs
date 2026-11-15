#[derive(Debug, Clone, Default)]
pub struct IsoChunkLevel {
    pub level: i32,
    // The decomp just uses the actual chunk it belongs to instead of the coords,
    // but rust doesn't like self referencing structs, so we just store the coords instead
    pub chunk_coords: (i32, i32),
    pub rain_splash_frame: Vec<f32>
}

impl IsoChunkLevel {
    pub fn new(chunk_coords: (i32, i32), level: i32) -> Self {
        Self {
            level,
            chunk_coords,
            rain_splash_frame: vec![-1.0; 64],
        }
    }
}