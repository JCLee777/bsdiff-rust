use std::io::Cursor;
use std::path::Path;
use std::time::Instant;
use qbsdiff::{Bsdiff, Bspatch, ParallelScheme};
use qbsdiff::bsdiff::MAX_LENGTH;

/// 性能统计信息
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// 操作耗时（毫秒）
    pub elapsed_ms: u64,
    /// 旧文件大小（字节）
    pub old_size: u64,
    /// 新文件大小（字节）
    pub new_size: u64,
    /// 补丁大小（字节）
    pub patch_size: u64,
    /// 压缩比（百分比）
    pub compression_ratio: f64,
}

/// Diff 配置选项
#[derive(Debug, Clone)]
pub struct DiffOptions {
    /// 压缩级别 (0-9)
    pub compression_level: u32,
    /// 是否启用并行处理
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
    /// 生成标准 BSDIFF40 格式的补丁文件
    pub fn diff(old_file: &str, new_file: &str, patch_file: &str) -> Result<(), Box<dyn std::error::Error>> {
        Self::diff_with_options(old_file, new_file, patch_file, &DiffOptions::default())
    }

    /// 生成补丁文件，支持自定义选项
    pub fn diff_with_options(
        old_file: &str, 
        new_file: &str, 
        patch_file: &str,
        options: &DiffOptions
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 验证输入文件
        if !Path::new(old_file).exists() {
            return Err(format!("Old file not found: {}", old_file).into());
        }
        if !Path::new(new_file).exists() {
            return Err(format!("New file not found: {}", new_file).into());
        }

        let old_data = std::fs::read(old_file)?;
        let new_data = std::fs::read(new_file)?;

        // 检查文件大小限制
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

    /// 生成补丁文件并返回性能统计
    pub fn diff_with_stats(
        old_file: &str, 
        new_file: &str, 
        patch_file: &str
    ) -> Result<PerformanceStats, Box<dyn std::error::Error>> {
        Self::diff_with_options_and_stats(old_file, new_file, patch_file, &DiffOptions::default())
    }

    /// 生成补丁文件，支持自定义选项并返回性能统计
    pub fn diff_with_options_and_stats(
        old_file: &str, 
        new_file: &str, 
        patch_file: &str,
        options: &DiffOptions
    ) -> Result<PerformanceStats, Box<dyn std::error::Error>> {
        let start = Instant::now();

        // 执行 diff
        Self::diff_with_options(old_file, new_file, patch_file, options)?;

        let elapsed = start.elapsed();

        // 收集统计信息
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

    /// 应用标准 BSDIFF40 格式的补丁文件
    pub fn patch(old_file: &str, new_file: &str, patch_file: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 验证输入文件
        if !Path::new(old_file).exists() {
            return Err(format!("Old file not found: {}", old_file).into());
        }
        if !Path::new(patch_file).exists() {
            return Err(format!("Patch file not found: {}", patch_file).into());
        }

        // 读取文件
        let old_data = std::fs::read(old_file)?;
        let patch_data = std::fs::read(patch_file)?;

        // 应用补丁，使用内存预分配优化
        let patcher = Bspatch::new(&patch_data)?;
        // 预分配目标文件大小，减少内存重分配，提升性能
        let mut new_data = Vec::with_capacity(patcher.hint_target_size() as usize);
        patcher.apply(&old_data, Cursor::new(&mut new_data))?;

        // 写入新文件
        std::fs::write(new_file, new_data)?;

        Ok(())
    }

    /// 应用补丁文件并返回性能统计
    pub fn patch_with_stats(
        old_file: &str, 
        new_file: &str, 
        patch_file: &str
    ) -> Result<PerformanceStats, Box<dyn std::error::Error>> {
        let start = Instant::now();

        // 执行 patch
        Self::patch(old_file, new_file, patch_file)?;

        let elapsed = start.elapsed();

        // 收集统计信息
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
        
        // 生成 BSDIFF40 格式的补丁
        BsdiffRust::diff(
            old_file.path().to_str().unwrap(),
            new_file.path().to_str().unwrap(),
            patch_file.path().to_str().unwrap(),
        ).unwrap();
        
        // 验证补丁文件头部
        let patch_data = fs::read(patch_file.path()).unwrap();
        assert_eq!(&patch_data[0..8], b"BSDIFF40", "Patch should have BSDIFF40 header");
        
        // 应用补丁
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
        
        // 生成补丁并获取统计
        let stats = BsdiffRust::diff_with_stats(
            old_file.path().to_str().unwrap(),
            new_file.path().to_str().unwrap(),
            patch_file.path().to_str().unwrap(),
        ).unwrap();
        
        assert!(stats.elapsed_ms >= 0);
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
        
        // 验证补丁可以正确应用
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
}
