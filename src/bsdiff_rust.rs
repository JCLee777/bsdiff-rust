use std::io::Cursor;
use std::path::Path;
use std::time::Instant;
use qbsdiff::{Bsdiff, Bspatch, ParallelScheme};
use qbsdiff::bsdiff::MAX_LENGTH;

/// Performance statistics.
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// Elapsed time in milliseconds.
    pub elapsed_ms: u64,
    /// Old file size in bytes.
    pub old_size: u64,
    /// New file size in bytes.
    pub new_size: u64,
    /// Patch file size in bytes.
    pub patch_size: u64,
    /// Compression ratio as a percentage.
    pub compression_ratio: f64,
}

/// Diff configuration options.
#[derive(Debug, Clone)]
pub struct DiffOptions {
    /// Compression level (0-9).
    pub compression_level: u32,
    /// Whether to enable parallel processing.
    pub enable_parallel: bool,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            compression_level: 6,
            enable_parallel: true,
        }
    }
}

pub struct BsdiffRust;

impl BsdiffRust {
    /// Generate a standard BSDIFF40 format patch file.
    pub fn diff(old_file: &str, new_file: &str, patch_file: &str) -> Result<(), Box<dyn std::error::Error>> {
        Self::diff_with_options(old_file, new_file, patch_file, &DiffOptions::default())
    }

    /// Generate a patch file with custom options.
    pub fn diff_with_options(
        old_file: &str, 
        new_file: &str, 
        patch_file: &str,
        options: &DiffOptions
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate input files
        if !Path::new(old_file).exists() {
            return Err(format!("Old file not found: {}", old_file).into());
        }
        if !Path::new(new_file).exists() {
            return Err(format!("New file not found: {}", new_file).into());
        }

        let old_data = std::fs::read(old_file)?;
        let new_data = std::fs::read(new_file)?;

        // Check file size limit
        if old_data.len() > MAX_LENGTH {
            return Err(format!(
                "Old file too large: {} bytes (max: {} bytes)", 
                old_data.len(), 
                MAX_LENGTH
            ).into());
        }

        let parallel_scheme = if options.enable_parallel {
            ParallelScheme::Auto
        } else {
            ParallelScheme::Never
        };

        let mut patch_data = Vec::new();
        Bsdiff::new(&old_data, &new_data)
            .compression_level(options.compression_level)
            .parallel_scheme(parallel_scheme)
            .compare(Cursor::new(&mut patch_data))?;

        std::fs::write(patch_file, patch_data)?;

        Ok(())
    }

    /// Generate a patch file and return performance statistics.
    pub fn diff_with_stats(
        old_file: &str, 
        new_file: &str, 
        patch_file: &str
    ) -> Result<PerformanceStats, Box<dyn std::error::Error>> {
        Self::diff_with_options_and_stats(old_file, new_file, patch_file, &DiffOptions::default())
    }

    /// Generate a patch file with custom options and return performance statistics.
    pub fn diff_with_options_and_stats(
        old_file: &str, 
        new_file: &str, 
        patch_file: &str,
        options: &DiffOptions
    ) -> Result<PerformanceStats, Box<dyn std::error::Error>> {
        let start = Instant::now();

        // Perform diff
        Self::diff_with_options(old_file, new_file, patch_file, options)?;

        let elapsed = start.elapsed();

        // Collect statistics
        let old_size = std::fs::metadata(old_file)?.len();
        let new_size = std::fs::metadata(new_file)?.len();
        let patch_size = std::fs::metadata(patch_file)?.len();

        let compression_ratio = if old_size + new_size > 0 {
            (patch_size as f64 / (old_size + new_size) as f64) * 100.0
        } else {
            0.0
        };

        Ok(PerformanceStats {
            elapsed_ms: elapsed.as_millis() as u64,
            old_size,
            new_size,
            patch_size,
            compression_ratio,
        })
    }

    /// Apply a standard BSDIFF40 format patch file.
    pub fn patch(old_file: &str, new_file: &str, patch_file: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Validate input files
        if !Path::new(old_file).exists() {
            return Err(format!("Old file not found: {}", old_file).into());
        }
        if !Path::new(patch_file).exists() {
            return Err(format!("Patch file not found: {}", patch_file).into());
        }

        // Read files
        let old_data = std::fs::read(old_file)?;
        let patch_data = std::fs::read(patch_file)?;

        // Apply patch with pre-allocated buffer for better performance
        let patcher = Bspatch::new(&patch_data)?;
        // Pre-allocate target size to reduce memory reallocations
        let mut new_data = Vec::with_capacity(patcher.hint_target_size() as usize);
        patcher.apply(&old_data, Cursor::new(&mut new_data))?;

        // Write output file
        std::fs::write(new_file, new_data)?;

        Ok(())
    }

    /// Apply a patch file and return performance statistics.
    pub fn patch_with_stats(
        old_file: &str, 
        new_file: &str, 
        patch_file: &str
    ) -> Result<PerformanceStats, Box<dyn std::error::Error>> {
        let start = Instant::now();

        // Perform patch
        Self::patch(old_file, new_file, patch_file)?;

        let elapsed = start.elapsed();

        // Collect statistics
        let old_size = std::fs::metadata(old_file)?.len();
        let new_size = std::fs::metadata(new_file)?.len();
        let patch_size = std::fs::metadata(patch_file)?.len();

        let compression_ratio = if old_size + new_size > 0 {
            (patch_size as f64 / (old_size + new_size) as f64) * 100.0
        } else {
            0.0
        };

        Ok(PerformanceStats {
            elapsed_ms: elapsed.as_millis() as u64,
            old_size,
            new_size,
            patch_size,
            compression_ratio,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_qbsdiff_diff_patch() {
        let old_content = b"Hello World! This is the old version with some content.";
        let new_content = b"Hello World! This is the new version with more content and changes.";
        
        let old_file = NamedTempFile::new().unwrap();
        let new_file = NamedTempFile::new().unwrap();
        let patch_file = NamedTempFile::new().unwrap();
        
        fs::write(&old_file, old_content).unwrap();
        fs::write(&new_file, new_content).unwrap();
        
        // Generate BSDIFF40 format patch
        BsdiffRust::diff(
            old_file.path().to_str().unwrap(),
            new_file.path().to_str().unwrap(),
            patch_file.path().to_str().unwrap(),
        ).unwrap();
        
        // Verify patch file header
        let patch_data = fs::read(patch_file.path()).unwrap();
        assert_eq!(&patch_data[0..8], b"BSDIFF40", "Patch should have BSDIFF40 header");
        
        // Apply patch
        let generated_file = NamedTempFile::new().unwrap();
        BsdiffRust::patch(
            old_file.path().to_str().unwrap(),
            generated_file.path().to_str().unwrap(),
            patch_file.path().to_str().unwrap(),
        ).unwrap();
        
        let generated_content = fs::read(generated_file.path()).unwrap();
        assert_eq!(generated_content, new_content, "Patched content should match new content");
    }

    #[test]
    fn test_diff_with_stats() {
        let old_content = b"Hello World! This is the old version.";
        let new_content = b"Hello World! This is the new version with more data.";
        
        let old_file = NamedTempFile::new().unwrap();
        let new_file = NamedTempFile::new().unwrap();
        let patch_file = NamedTempFile::new().unwrap();
        
        fs::write(&old_file, old_content).unwrap();
        fs::write(&new_file, new_content).unwrap();
        
        // Generate patch and collect statistics
        let stats = BsdiffRust::diff_with_stats(
            old_file.path().to_str().unwrap(),
            new_file.path().to_str().unwrap(),
            patch_file.path().to_str().unwrap(),
        ).unwrap();
        
        // elapsed_ms is u64, always >= 0; just verify the field is accessible
        let _ = stats.elapsed_ms;
        assert_eq!(stats.old_size, old_content.len() as u64);
        assert_eq!(stats.new_size, new_content.len() as u64);
        assert!(stats.patch_size > 0);
        assert!(stats.compression_ratio >= 0.0);
    }

    #[test]
    fn test_diff_with_options() {
        let old_content = b"Test data for parallel option";
        let new_content = b"Test data for parallel option modified";
        
        let old_file = NamedTempFile::new().unwrap();
        let new_file = NamedTempFile::new().unwrap();
        let patch_file = NamedTempFile::new().unwrap();
        
        fs::write(&old_file, old_content).unwrap();
        fs::write(&new_file, new_content).unwrap();
        
        let options = DiffOptions {
            compression_level: 9,
            enable_parallel: false,
        };
        
        BsdiffRust::diff_with_options(
            old_file.path().to_str().unwrap(),
            new_file.path().to_str().unwrap(),
            patch_file.path().to_str().unwrap(),
            &options,
        ).unwrap();
        
        // Verify patch can be applied correctly
        let generated_file = NamedTempFile::new().unwrap();
        BsdiffRust::patch(
            old_file.path().to_str().unwrap(),
            generated_file.path().to_str().unwrap(),
            patch_file.path().to_str().unwrap(),
        ).unwrap();
        
        let generated_content = fs::read(generated_file.path()).unwrap();
        assert_eq!(generated_content, new_content);
    }

    #[test]
    fn test_file_not_found_errors() {
        let temp = NamedTempFile::new().unwrap();
        
        // Test diff with non-existent old file
        let result = BsdiffRust::diff(
            "/nonexistent/old.bin",
            temp.path().to_str().unwrap(),
            temp.path().to_str().unwrap(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Old file not found"));

        // Test patch with non-existent patch file
        let result = BsdiffRust::patch(
            temp.path().to_str().unwrap(),
            temp.path().to_str().unwrap(),
            "/nonexistent/patch.bin",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Patch file not found"));
    }

    #[test]
    fn test_identical_files() {
        let content = b"Identical content for both files";

        let old_file = NamedTempFile::new().unwrap();
        let new_file = NamedTempFile::new().unwrap();
        let patch_file = NamedTempFile::new().unwrap();

        fs::write(&old_file, content).unwrap();
        fs::write(&new_file, content).unwrap();

        // Diff two identical files should succeed
        BsdiffRust::diff(
            old_file.path().to_str().unwrap(),
            new_file.path().to_str().unwrap(),
            patch_file.path().to_str().unwrap(),
        ).unwrap();

        // Patch should restore the exact same content
        let generated_file = NamedTempFile::new().unwrap();
        BsdiffRust::patch(
            old_file.path().to_str().unwrap(),
            generated_file.path().to_str().unwrap(),
            patch_file.path().to_str().unwrap(),
        ).unwrap();

        let generated_content = fs::read(generated_file.path()).unwrap();
        assert_eq!(generated_content, content, "Patched content should match original for identical files");

        // Patch for identical files should exist and have a valid BSDIFF40 header
        let patch_data = fs::read(patch_file.path()).unwrap();
        assert!(patch_data.len() >= 8, "Patch file should contain at least the BSDIFF40 header");
        assert_eq!(&patch_data[0..8], b"BSDIFF40", "Patch should have BSDIFF40 header");
    }

    #[test]
    fn test_empty_files() {
        let old_file = NamedTempFile::new().unwrap();
        let new_file = NamedTempFile::new().unwrap();
        let patch_file = NamedTempFile::new().unwrap();

        fs::write(&old_file, b"").unwrap();
        fs::write(&new_file, b"").unwrap();

        // Diff two empty files should succeed
        BsdiffRust::diff(
            old_file.path().to_str().unwrap(),
            new_file.path().to_str().unwrap(),
            patch_file.path().to_str().unwrap(),
        ).unwrap();

        // Patch should produce an empty file
        let generated_file = NamedTempFile::new().unwrap();
        BsdiffRust::patch(
            old_file.path().to_str().unwrap(),
            generated_file.path().to_str().unwrap(),
            patch_file.path().to_str().unwrap(),
        ).unwrap();

        let generated_content = fs::read(generated_file.path()).unwrap();
        assert!(generated_content.is_empty(), "Patched empty files should produce empty output");

        // Stats should handle zero-size files without panicking
        let stats = BsdiffRust::diff_with_stats(
            old_file.path().to_str().unwrap(),
            new_file.path().to_str().unwrap(),
            patch_file.path().to_str().unwrap(),
        ).unwrap();
        assert_eq!(stats.old_size, 0);
        assert_eq!(stats.new_size, 0);
        assert_eq!(stats.compression_ratio, 0.0, "Compression ratio should be 0 when both files are empty");
    }

    #[test]
    fn test_corrupted_patch() {
        let old_file = NamedTempFile::new().unwrap();
        let patch_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&old_file, b"some original content").unwrap();
        fs::write(&patch_file, b"this is not a valid bsdiff patch").unwrap();

        // Applying a corrupted patch should return an error, not panic
        let result = BsdiffRust::patch(
            old_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            patch_file.path().to_str().unwrap(),
        );
        assert!(result.is_err(), "Corrupted patch should produce an error");
    }
}
