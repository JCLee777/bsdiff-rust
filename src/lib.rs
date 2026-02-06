use napi::bindgen_prelude::*;
use napi_derive::napi;

mod bsdiff_rust;
mod utils;
use bsdiff_rust::{BsdiffRust, DiffOptions};
use utils::{verify_patch as verify_patch_util, get_patch_info, get_file_size, check_file_access, get_compression_ratio};

// ============================================================
// Common type conversions and helper functions
// ============================================================

/// Convert a `Box<dyn Error>` into a `napi::Error`.
fn to_napi_err(e: Box<dyn std::error::Error>) -> Error {
  Error::from_reason(e.to_string())
}

/// Convert `Result<T, Box<dyn Error>>` into `napi::Result<T>`.
fn into_napi<T>(result: std::result::Result<T, Box<dyn std::error::Error>>) -> Result<T> {
  result.map_err(to_napi_err)
}

// ============================================================
// JS ↔ Rust struct definitions and type conversions
// ============================================================

/// Patch file information exposed to JavaScript.
#[napi(object)]
pub struct PatchInfoJs {
  pub size: f64,
  pub compressed: bool,
}

/// Compression ratio information exposed to JavaScript.
#[napi(object)]
pub struct CompressionRatioJs {
  pub old_size: f64,
  pub new_size: f64,
  pub patch_size: f64,
  pub ratio: f64,
}

/// Performance statistics exposed to JavaScript.
#[napi(object)]
pub struct PerformanceStatsJs {
  /// Elapsed time in milliseconds.
  pub elapsed_ms: f64,
  /// Old file size in bytes.
  pub old_size: f64,
  /// New file size in bytes.
  pub new_size: f64,
  /// Patch file size in bytes.
  pub patch_size: f64,
  /// Compression ratio as a percentage.
  pub compression_ratio: f64,
}

impl From<bsdiff_rust::PerformanceStats> for PerformanceStatsJs {
  fn from(s: bsdiff_rust::PerformanceStats) -> Self {
    Self {
      elapsed_ms: s.elapsed_ms as f64,
      old_size: s.old_size as f64,
      new_size: s.new_size as f64,
      patch_size: s.patch_size as f64,
      compression_ratio: s.compression_ratio,
    }
  }
}

/// Diff configuration options exposed to JavaScript.
#[napi(object)]
pub struct DiffOptionsJs {
  /// Compression level (0-9, default 6).
  pub compression_level: Option<u32>,
  /// Enable parallel processing (default true).
  pub enable_parallel: Option<bool>,
}

impl From<DiffOptionsJs> for DiffOptions {
  fn from(js: DiffOptionsJs) -> Self {
    Self {
      compression_level: js.compression_level.unwrap_or(6),
      enable_parallel: js.enable_parallel.unwrap_or(true),
    }
  }
}

// ============================================================
// Synchronous API
// ============================================================

#[napi]
pub fn diff_sync(old_str: String, new_str: String, patch: String) -> Result<()> {
  into_napi(BsdiffRust::diff(&old_str, &new_str, &patch))
}

#[napi]
pub fn patch_sync(old_str: String, new_str: String, patch: String) -> Result<()> {
  into_napi(BsdiffRust::patch(&old_str, &new_str, &patch))
}

/// Generate a patch file and return performance statistics (sync).
#[napi]
pub fn diff_with_stats_sync(old_str: String, new_str: String, patch: String) -> Result<PerformanceStatsJs> {
  into_napi(BsdiffRust::diff_with_stats(&old_str, &new_str, &patch)).map(Into::into)
}

/// Apply a patch file and return performance statistics (sync).
#[napi]
pub fn patch_with_stats_sync(old_str: String, new_str: String, patch: String) -> Result<PerformanceStatsJs> {
  into_napi(BsdiffRust::patch_with_stats(&old_str, &new_str, &patch)).map(Into::into)
}

/// Generate a patch file with custom options (sync).
#[napi]
pub fn diff_with_options_sync(
  old_str: String,
  new_str: String,
  patch: String,
  options: DiffOptionsJs,
) -> Result<()> {
  let opts: DiffOptions = options.into();
  into_napi(BsdiffRust::diff_with_options(&old_str, &new_str, &patch, &opts))
}

/// Generate a patch file with custom options and return performance statistics (sync).
#[napi]
pub fn diff_with_options_and_stats_sync(
  old_str: String,
  new_str: String,
  patch: String,
  options: DiffOptionsJs,
) -> Result<PerformanceStatsJs> {
  let opts: DiffOptions = options.into();
  into_napi(BsdiffRust::diff_with_options_and_stats(&old_str, &new_str, &patch, &opts)).map(Into::into)
}

/// Verify patch file integrity.
#[napi]
pub fn verify_patch_sync(old_str: String, new_str: String, patch: String) -> Result<bool> {
  into_napi(verify_patch_util(&old_str, &new_str, &patch))
}

/// Get patch file information.
#[napi]
pub fn get_patch_info_sync(patch: String) -> Result<PatchInfoJs> {
  let info = into_napi(get_patch_info(&patch))?;
  Ok(PatchInfoJs {
    size: info.size as f64,
    compressed: info.compressed,
  })
}

/// Get file size.
#[napi]
pub fn get_file_size_sync(file_path: String) -> Result<f64> {
  into_napi(get_file_size(&file_path)).map(|s| s as f64)
}

/// Check file access permissions.
#[napi]
pub fn check_file_access_sync(file_path: String) -> Result<()> {
  into_napi(check_file_access(&file_path))
}

/// Get compression ratio information.
#[napi]
pub fn get_compression_ratio_sync(old_str: String, new_str: String, patch: String) -> Result<CompressionRatioJs> {
  let ratio = into_napi(get_compression_ratio(&old_str, &new_str, &patch))?;
  Ok(CompressionRatioJs {
    old_size: ratio.old_size as f64,
    new_size: ratio.new_size as f64,
    patch_size: ratio.patch_size as f64,
    ratio: ratio.ratio,
  })
}

// ============================================================
// Async Task definitions
// ============================================================

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
    into_napi(BsdiffRust::diff(&self.old_str, &self.new_str, &self.patch))
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
    into_napi(BsdiffRust::patch(&self.old_str, &self.new_str, &self.patch))
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
    into_napi(verify_patch_util(&self.old_str, &self.new_str, &self.patch))
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

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
    into_napi(BsdiffRust::diff_with_stats(&self.old_str, &self.new_str, &self.patch))
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output.into())
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
    into_napi(BsdiffRust::patch_with_stats(&self.old_str, &self.new_str, &self.patch))
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output.into())
  }
}

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
    into_napi(BsdiffRust::diff_with_options(&self.old_str, &self.new_str, &self.patch, &self.options))
  }

  fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
    Ok(())
  }
}

// ============================================================
// Async API exports
// ============================================================

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

/// Generate a patch file and return performance statistics (async).
#[napi]
pub fn diff_with_stats(
  old_str: String,
  new_str: String,
  patch: String,
) -> Result<AsyncTask<DiffWithStatsTask>> {
  Ok(AsyncTask::new(DiffWithStatsTask { old_str, new_str, patch }))
}

/// Apply a patch file and return performance statistics (async).
#[napi]
pub fn patch_with_stats(
  old_str: String,
  new_str: String,
  patch: String,
) -> Result<AsyncTask<PatchWithStatsTask>> {
  Ok(AsyncTask::new(PatchWithStatsTask { old_str, new_str, patch }))
}

/// Generate a patch file with custom options (async).
#[napi]
pub fn diff_with_options(
  old_str: String,
  new_str: String,
  patch: String,
  options: DiffOptionsJs,
) -> Result<AsyncTask<DiffWithOptionsTask>> {
  let opts: DiffOptions = options.into();
  Ok(AsyncTask::new(DiffWithOptionsTask {
    old_str,
    new_str,
    patch,
    options: opts,
  }))
}