use napi::bindgen_prelude::*;
use napi_derive::napi;

mod bsdiff_rust;
mod utils;
use bsdiff_rust::{BsdiffRust, DiffOptions};
use utils::{verify_patch as verify_patch_util, get_patch_info, get_file_size, check_file_access, get_compression_ratio};

fn call_bsdiff(
  old_str: &str,
  new_str: &str,
  patch: &str,
) -> Result<()> {
  BsdiffRust::diff(old_str, new_str, patch)
    .map_err(|e| Error::from_reason(e.to_string()))
}

fn call_bspatch(
  old_str: &str,
  new_str: &str,
  patch: &str,
) -> Result<()> {
  BsdiffRust::patch(old_str, new_str, patch)
    .map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
pub fn diff_sync(old_str: String, new_str: String, patch: String) -> Result<()> {
  call_bsdiff(&old_str, &new_str, &patch)
}

#[napi]
pub fn patch_sync(old_str: String, new_str: String, patch: String) -> Result<()> {
  call_bspatch(&old_str, &new_str, &patch)
}

/// 生成补丁文件并返回性能统计（同步）
#[napi]
pub fn diff_with_stats_sync(old_str: String, new_str: String, patch: String) -> Result<PerformanceStatsJs> {
  let stats = BsdiffRust::diff_with_stats(&old_str, &new_str, &patch)
    .map_err(|e| Error::from_reason(e.to_string()))?;
  
  Ok(PerformanceStatsJs {
    elapsed_ms: stats.elapsed_ms as f64,
    old_size: stats.old_size as f64,
    new_size: stats.new_size as f64,
    patch_size: stats.patch_size as f64,
    compression_ratio: stats.compression_ratio,
  })
}

/// 应用补丁文件并返回性能统计（同步）
#[napi]
pub fn patch_with_stats_sync(old_str: String, new_str: String, patch: String) -> Result<PerformanceStatsJs> {
  let stats = BsdiffRust::patch_with_stats(&old_str, &new_str, &patch)
    .map_err(|e| Error::from_reason(e.to_string()))?;
  
  Ok(PerformanceStatsJs {
    elapsed_ms: stats.elapsed_ms as f64,
    old_size: stats.old_size as f64,
    new_size: stats.new_size as f64,
    patch_size: stats.patch_size as f64,
    compression_ratio: stats.compression_ratio,
  })
}

/// 生成补丁文件，支持自定义选项（同步）
#[napi]
pub fn diff_with_options_sync(
  old_str: String, 
  new_str: String, 
  patch: String,
  options: DiffOptionsJs
) -> Result<()> {
  let opts = DiffOptions {
    compression_level: options.compression_level.unwrap_or(6),
    enable_parallel: options.enable_parallel.unwrap_or(true),
  };
  
  BsdiffRust::diff_with_options(&old_str, &new_str, &patch, &opts)
    .map_err(|e| Error::from_reason(e.to_string()))
}

/// 生成补丁文件，支持自定义选项并返回性能统计（同步）
#[napi]
pub fn diff_with_options_and_stats_sync(
  old_str: String, 
  new_str: String, 
  patch: String,
  options: DiffOptionsJs
) -> Result<PerformanceStatsJs> {
  let opts = DiffOptions {
    compression_level: options.compression_level.unwrap_or(6),
    enable_parallel: options.enable_parallel.unwrap_or(true),
  };
  
  let stats = BsdiffRust::diff_with_options_and_stats(&old_str, &new_str, &patch, &opts)
    .map_err(|e| Error::from_reason(e.to_string()))?;
  
  Ok(PerformanceStatsJs {
    elapsed_ms: stats.elapsed_ms as f64,
    old_size: stats.old_size as f64,
    new_size: stats.new_size as f64,
    patch_size: stats.patch_size as f64,
    compression_ratio: stats.compression_ratio,
  })
}

/// 验证补丁文件完整性
#[napi]
pub fn verify_patch_sync(old_str: String, new_str: String, patch: String) -> Result<bool> {
  verify_patch_util(&old_str, &new_str, &patch)
    .map_err(|e| Error::from_reason(e.to_string()))
}

/// 获取补丁文件信息
#[napi]
pub fn get_patch_info_sync(patch: String) -> Result<PatchInfoJs> {
  let info = get_patch_info(&patch)
    .map_err(|e| Error::from_reason(e.to_string()))?;
  
  Ok(PatchInfoJs {
    size: info.size as f64,
    compressed: info.compressed,
  })
}

/// 获取文件大小
#[napi]
pub fn get_file_size_sync(file_path: String) -> Result<f64> {
  get_file_size(&file_path)
    .map(|size| size as f64)
    .map_err(|e| Error::from_reason(e.to_string()))
}

/// 检查文件访问权限
#[napi]
pub fn check_file_access_sync(file_path: String) -> Result<()> {
  check_file_access(&file_path)
    .map_err(|e| Error::from_reason(e.to_string()))
}

/// 获取压缩比信息
#[napi]
pub fn get_compression_ratio_sync(old_str: String, new_str: String, patch: String) -> Result<CompressionRatioJs> {
  let ratio = get_compression_ratio(&old_str, &new_str, &patch)
    .map_err(|e| Error::from_reason(e.to_string()))?;
  
  Ok(CompressionRatioJs {
    old_size: ratio.old_size as f64,
    new_size: ratio.new_size as f64,
    patch_size: ratio.patch_size as f64,
    ratio: ratio.ratio,
  })
}

/// JavaScript 补丁信息结构
#[napi(object)]
pub struct PatchInfoJs {
  pub size: f64,
  pub compressed: bool,
}

/// JavaScript 压缩比信息结构
#[napi(object)]
pub struct CompressionRatioJs {
  pub old_size: f64,
  pub new_size: f64,
  pub patch_size: f64,
  pub ratio: f64,
}

/// JavaScript 性能统计结构
#[napi(object)]
pub struct PerformanceStatsJs {
  /// 操作耗时（毫秒）
  pub elapsed_ms: f64,
  /// 旧文件大小（字节）
  pub old_size: f64,
  /// 新文件大小（字节）
  pub new_size: f64,
  /// 补丁大小（字节）
  pub patch_size: f64,
  /// 压缩比（百分比）
  pub compression_ratio: f64,
}

/// JavaScript Diff 配置选项
#[napi(object)]
pub struct DiffOptionsJs {
  /// 压缩级别 (0-9, 默认 6)
  pub compression_level: Option<u32>,
  /// 是否启用并行处理（默认 true）
  pub enable_parallel: Option<bool>,
}

// 简化的异步版本，暂时不包含进度回调
pub struct DiffTask {
  old_str: String,
  new_str: String,
  patch: String,
}

#[napi]
impl Task for DiffTask {
  type Output = ();
  type JsValue = ();

  fn compute(&mut self) -> Result<Self::Output> {
    call_bsdiff(&self.old_str, &self.new_str, &self.patch)
  }

  fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
    Ok(())
  }
}

pub struct PatchTask {
  old_str: String,
  new_str: String,
  patch: String,
}

#[napi]
impl Task for PatchTask {
  type Output = ();
  type JsValue = ();

  fn compute(&mut self) -> Result<Self::Output> {
    call_bspatch(&self.old_str, &self.new_str, &self.patch)
  }

  fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
    Ok(())
  }
}

pub struct VerifyPatchTask {
  old_str: String,
  new_str: String,
  patch: String,
}

#[napi]
impl Task for VerifyPatchTask {
  type Output = bool;
  type JsValue = bool;

  fn compute(&mut self) -> Result<Self::Output> {
    verify_patch_util(&self.old_str, &self.new_str, &self.patch)
      .map_err(|e| Error::from_reason(e.to_string()))
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

// 带性能统计的异步任务
pub struct DiffWithStatsTask {
  old_str: String,
  new_str: String,
  patch: String,
}

#[napi]
impl Task for DiffWithStatsTask {
  type Output = bsdiff_rust::PerformanceStats;
  type JsValue = PerformanceStatsJs;

  fn compute(&mut self) -> Result<Self::Output> {
    BsdiffRust::diff_with_stats(&self.old_str, &self.new_str, &self.patch)
      .map_err(|e| Error::from_reason(e.to_string()))
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(PerformanceStatsJs {
      elapsed_ms: output.elapsed_ms as f64,
      old_size: output.old_size as f64,
      new_size: output.new_size as f64,
      patch_size: output.patch_size as f64,
      compression_ratio: output.compression_ratio,
    })
  }
}

pub struct PatchWithStatsTask {
  old_str: String,
  new_str: String,
  patch: String,
}

#[napi]
impl Task for PatchWithStatsTask {
  type Output = bsdiff_rust::PerformanceStats;
  type JsValue = PerformanceStatsJs;

  fn compute(&mut self) -> Result<Self::Output> {
    BsdiffRust::patch_with_stats(&self.old_str, &self.new_str, &self.patch)
      .map_err(|e| Error::from_reason(e.to_string()))
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(PerformanceStatsJs {
      elapsed_ms: output.elapsed_ms as f64,
      old_size: output.old_size as f64,
      new_size: output.new_size as f64,
      patch_size: output.patch_size as f64,
      compression_ratio: output.compression_ratio,
    })
  }
}

// 带配置选项的异步任务
pub struct DiffWithOptionsTask {
  old_str: String,
  new_str: String,
  patch: String,
  options: DiffOptions,
}

#[napi]
impl Task for DiffWithOptionsTask {
  type Output = ();
  type JsValue = ();

  fn compute(&mut self) -> Result<Self::Output> {
    BsdiffRust::diff_with_options(&self.old_str, &self.new_str, &self.patch, &self.options)
      .map_err(|e| Error::from_reason(e.to_string()))
  }

  fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
    Ok(())
  }
}

#[napi]
pub fn diff(
  old_str: String,
  new_str: String,
  patch: String,
) -> Result<AsyncTask<DiffTask>> {
  Ok(AsyncTask::new(DiffTask { old_str, new_str, patch }))
}

#[napi]
pub fn patch(
  old_str: String,
  new_str: String,
  patch: String,
) -> Result<AsyncTask<PatchTask>> {
  Ok(AsyncTask::new(PatchTask { old_str, new_str, patch }))
}

#[napi]
pub fn verify_patch(
  old_str: String,
  new_str: String,
  patch: String,
) -> Result<AsyncTask<VerifyPatchTask>> {
  Ok(AsyncTask::new(VerifyPatchTask { old_str, new_str, patch }))
}

/// 生成补丁文件并返回性能统计（异步）
#[napi]
pub fn diff_with_stats(
  old_str: String,
  new_str: String,
  patch: String,
) -> Result<AsyncTask<DiffWithStatsTask>> {
  Ok(AsyncTask::new(DiffWithStatsTask { old_str, new_str, patch }))
}

/// 应用补丁文件并返回性能统计（异步）
#[napi]
pub fn patch_with_stats(
  old_str: String,
  new_str: String,
  patch: String,
) -> Result<AsyncTask<PatchWithStatsTask>> {
  Ok(AsyncTask::new(PatchWithStatsTask { old_str, new_str, patch }))
}

/// 生成补丁文件，支持自定义选项（异步）
#[napi]
pub fn diff_with_options(
  old_str: String,
  new_str: String,
  patch: String,
  options: DiffOptionsJs,
) -> Result<AsyncTask<DiffWithOptionsTask>> {
  let opts = DiffOptions {
    compression_level: options.compression_level.unwrap_or(6),
    enable_parallel: options.enable_parallel.unwrap_or(true),
  };
  Ok(AsyncTask::new(DiffWithOptionsTask { 
    old_str, 
    new_str, 
    patch,
    options: opts,
  }))
}