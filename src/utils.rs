use std::fs::File;
use std::io::Read;

/// Patch file information.
#[derive(Debug, Clone)]
pub struct PatchInfo {
    pub size: u64,
    pub compressed: bool,
}

/// Compression ratio information.
#[derive(Debug, Clone)]
pub struct CompressionRatio {
    pub old_size: u64,
    pub new_size: u64,
    pub patch_size: u64,
    pub ratio: f64, // percentage
}

/// Verify patch file integrity.
pub fn verify_patch(old_file: &str, new_file: &str, patch_file: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let new_data = std::fs::read(new_file)?;
    
    // Create a temporary file to apply the patch
    let temp_file = tempfile::NamedTempFile::new()?;
    let temp_path = temp_file.path().to_str().ok_or("Invalid temp path")?;
    
    // Apply the patch using BsdiffRust::patch
    crate::bsdiff_rust::BsdiffRust::patch(old_file, temp_path, patch_file)?;
    
    // Read the generated data and compare
    let patched_data = std::fs::read(temp_path)?;
    
    Ok(patched_data == new_data)
}

/// Get patch file information.
pub fn get_patch_info(patch_file: &str) -> Result<PatchInfo, Box<dyn std::error::Error>> {
    let metadata = std::fs::metadata(patch_file)?;
    
    // Check if the file is in BSDIFF40 format
    let mut file = File::open(patch_file)?;
    let mut header = [0u8; 8];
    file.read_exact(&mut header).ok();
    let is_bsdiff40 = &header == b"BSDIFF40";
    
    Ok(PatchInfo {
        size: metadata.len(),
        compressed: is_bsdiff40, // BSDIFF40 format uses bzip2 compression
    })
}

/// Get file size in bytes.
pub fn get_file_size(file_path: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let metadata = std::fs::metadata(file_path)?;
    Ok(metadata.len())
}

/// Check whether a file exists and is readable.
pub fn check_file_access(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path).into());
    }
    if !path.is_file() {
        return Err(format!("Path is not a file: {}", file_path).into());
    }
    // Try opening the file to verify readability
    File::open(file_path)?;
    Ok(())
}

/// Get compression ratio information.
pub fn get_compression_ratio(old_file: &str, new_file: &str, patch_file: &str) -> Result<CompressionRatio, Box<dyn std::error::Error>> {
    let old_size = get_file_size(old_file)?;
    let new_size = get_file_size(new_file)?;
    let patch_size = get_file_size(patch_file)?;
    
    let total_size = old_size + new_size;
    let ratio = if total_size > 0 {
        (patch_size as f64 / total_size as f64) * 100.0
    } else {
        0.0
    };
    
    Ok(CompressionRatio {
        old_size,
        new_size,
        patch_size,
        ratio,
    })
}
