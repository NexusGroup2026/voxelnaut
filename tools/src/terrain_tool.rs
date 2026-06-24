//! Terrain generation tool for VoxelNaut
//!
//! Standalone tool for generating and inspecting terrain.

use std::path::PathBuf;

/// Terrain generation tool
pub struct TerrainTool {
    seed: u64,
    output_dir: PathBuf,
}

impl TerrainTool {
    pub fn new(seed: u64, output_dir: PathBuf) -> Self {
        Self { seed, output_dir }
    }

    /// Generate terrain and save as image
    pub fn generate_heightmap(&self, width: usize, height: usize) -> Vec<f32> {
        use noise::{Perlin, Seedable};
        
        let perlin = Perlin::new(self.seed as u32);
        let scale = 0.01;
        
        let mut heights = Vec::with_capacity(width * height);
        
        for y in 0..height {
            for x in 0..width {
                let noise = perlin.get([
                    x as f64 * scale,
                    y as f64 * scale,
                ]);
                heights.push((noise + 1.0) / 2.0);
            }
        }
        
        heights
    }

    /// Save heightmap as PPM image
    pub fn save_as_ppm(&self, heights: &[f32], width: usize, height: usize, path: &PathBuf) -> std::io::Result<()> {
        use std::io::Write;
        
        let mut file = std::fs::File::create(path)?;
        writeln!(file, "P6 {} {} 255", width, height)?;
        
        for &h in heights {
            let value = (h * 255.0) as u8;
            file.write_all(&[value, value, value])?;
        }
        
        Ok(())
    }
}