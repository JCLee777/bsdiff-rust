#!/usr/bin/env node

import * as fs from 'fs'
import * as path from 'path'
import bsdiff from '../index'
import type { DiffOptionsJs } from '../index'

// Test resources directory (relative to project root)
const RESOURCES_DIR = path.resolve(process.cwd(), 'test/resources')
const TEMP_DIR = path.resolve(process.cwd(), 'temp')

// 定义类型
interface BenchmarkResult {
  name: string
  time: number
  size: number
  throughput: number
  compressionRatio: number
}

interface FileSize {
  name: string
  size: number
}

interface ChangeRatio {
  name: string
  ratio: number
}

// 生成测试数据
function generateTestData(size: number): Buffer {
  const data = Buffer.alloc(size)
  for (let i = 0; i < size; i++) {
    data[i] = i % 256
  }
  return data
}

// 生成差异化的测试数据
function generateDiffData(baseData: Buffer, changeRatio: number): Buffer {
  const newData = Buffer.from(baseData)
  const changeCount = Math.floor(baseData.length * changeRatio)

  for (let i = 0; i < changeCount; i++) {
    const index = i % baseData.length
    newData[index] = (newData[index] + 1) % 256
  }

  return newData
}

// 格式化文件大小
function formatFileSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB']
  let size = bytes
  let unitIndex = 0

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex++
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`
}

// 格式化时间
function formatTime(ms: number): string {
  if (ms < 1) {
    return `${(ms * 1000).toFixed(2)}μs`
  } else if (ms < 1000) {
    return `${ms.toFixed(2)}ms`
  } else {
    return `${(ms / 1000).toFixed(2)}s`
  }
}

// 创建临时文件并返回清理函数
function createTempFiles(
  oldData: Buffer,
  newData: Buffer,
  prefix: string,
): {
  oldFile: string
  newFile: string
  patchFile: string
  cleanup: () => void
} {
  const oldFile = path.join(TEMP_DIR, `old_${prefix}.bin`)
  const newFile = path.join(TEMP_DIR, `new_${prefix}.bin`)
  const patchFile = path.join(TEMP_DIR, `patch_${prefix}.bin`)

  // 确保临时目录存在
  const tempDir = path.dirname(oldFile)
  if (!fs.existsSync(tempDir)) {
    fs.mkdirSync(tempDir, { recursive: true })
  }

  fs.writeFileSync(oldFile, oldData)
  fs.writeFileSync(newFile, newData)

  const cleanup = () => {
    try {
      if (fs.existsSync(oldFile)) fs.unlinkSync(oldFile)
      if (fs.existsSync(newFile)) fs.unlinkSync(newFile)
      if (fs.existsSync(patchFile)) fs.unlinkSync(patchFile)
    } catch (error) {
      // 忽略清理错误
    }
  }

  return { oldFile, newFile, patchFile, cleanup }
}

// 测试不同配置选项的性能
async function benchmarkConfigurations(): Promise<void> {
  console.log('\n⚙️  配置选项性能对比')
  console.log('='.repeat(70))

  const size = 2 * 1024 * 1024 // 2MB
  const oldData = generateTestData(size)
  const newData = generateDiffData(oldData, 0.1)

  const configs: Array<{ name: string; options: DiffOptionsJs }> = [
    { name: 'Fast (level 1, parallel)', options: { compressionLevel: 1, enableParallel: true } },
    { name: 'Default (level 6, parallel)', options: { compressionLevel: 6, enableParallel: true } },
    { name: 'Best (level 9, parallel)', options: { compressionLevel: 9, enableParallel: true } },
    { name: 'Sequential (level 6)', options: { compressionLevel: 6, enableParallel: false } },
  ]

  const results: BenchmarkResult[] = []

  for (const config of configs) {
    const { oldFile, newFile, patchFile, cleanup } = createTempFiles(oldData, newData, `config_${config.name.replace(/\s/g, '_')}`)

    // 使用性能统计 API
    const stats = bsdiff.diffWithOptionsAndStatsSync(oldFile, newFile, patchFile, config.options)
    
    const throughput = (stats.oldSize + stats.newSize) / 1024 / 1024 / (stats.elapsedMs / 1000)
    
    results.push({
      name: config.name,
      time: stats.elapsedMs,
      size: stats.patchSize,
      throughput,
      compressionRatio: stats.compressionRatio,
    })

    console.log(`\n📊 ${config.name}`)
    console.log(`   ⏱️  Time: ${formatTime(stats.elapsedMs)}`)
    console.log(`   📦 Patch Size: ${formatFileSize(stats.patchSize)}`)
    console.log(`   📊 Compression: ${stats.compressionRatio.toFixed(2)}%`)
    console.log(`   🚀 Throughput: ${throughput.toFixed(2)} MB/s`)

    cleanup()
  }

  // 性能对比
  console.log('\n📈 Performance Comparison:')
  const baseline = results.find(r => r.name.includes('Default'))!
  console.table(results.map(r => ({
    Configuration: r.name,
    'Time (ms)': r.time.toFixed(0),
    'Size (KB)': (r.size / 1024).toFixed(2),
    'Throughput (MB/s)': r.throughput.toFixed(2),
    'Speedup': (baseline.time / r.time).toFixed(2) + 'x',
    'Size vs Default': ((r.size / baseline.size - 1) * 100).toFixed(1) + '%',
  })))
}

// 测试不同文件大小
async function benchmarkDifferentSizes(): Promise<void> {
  console.log('\n📏 不同文件大小性能测试')
  console.log('='.repeat(70))

  const sizes: FileSize[] = [
    { name: '100KB', size: 100 * 1024 },
    { name: '500KB', size: 500 * 1024 },
    { name: '1MB', size: 1024 * 1024 },
    { name: '2MB', size: 2 * 1024 * 1024 },
    { name: '5MB', size: 5 * 1024 * 1024 },
    { name: '10MB', size: 10 * 1024 * 1024 },
  ]

  const results: BenchmarkResult[] = []

  for (const { name, size } of sizes) {
    console.log(`\n🧪 Testing: ${name} (${formatFileSize(size)})`)

    const oldData = generateTestData(size)
    const newData = generateDiffData(oldData, 0.1)
    const { oldFile, newFile, patchFile, cleanup } = createTempFiles(oldData, newData, name)

    // 使用性能统计 API
    const diffStats = bsdiff.diffWithStatsSync(oldFile, newFile, patchFile)
    const throughput = (diffStats.oldSize + diffStats.newSize) / 1024 / 1024 / (diffStats.elapsedMs / 1000)

    console.log(`   ⏱️  Diff Time: ${formatTime(diffStats.elapsedMs)}`)
    console.log(`   📦 Patch Size: ${formatFileSize(diffStats.patchSize)}`)
    console.log(`   🚀 Throughput: ${throughput.toFixed(2)} MB/s`)

    // 测试 Patch 性能
    const appliedFile = path.join(TEMP_DIR, `applied_${name}.bin`)
    const patchStats = bsdiff.patchWithStatsSync(oldFile, appliedFile, patchFile)
    const patchThroughput = (patchStats.oldSize + patchStats.patchSize) / 1024 / 1024 / (patchStats.elapsedMs / 1000)

    console.log(`   ⏱️  Patch Time: ${formatTime(patchStats.elapsedMs)}`)
    console.log(`   🚀 Patch Throughput: ${patchThroughput.toFixed(2)} MB/s`)
    console.log(`   📊 Diff vs Patch: ${(diffStats.elapsedMs / patchStats.elapsedMs).toFixed(2)}x`)

    // 验证
    const isValid = fs.readFileSync(appliedFile).equals(newData)
    console.log(`   ✅ Validation: ${isValid ? 'PASSED' : 'FAILED'}`)

    results.push({
      name,
      time: diffStats.elapsedMs,
      size: diffStats.patchSize,
      throughput,
      compressionRatio: diffStats.compressionRatio,
    })

    if (fs.existsSync(appliedFile)) fs.unlinkSync(appliedFile)
    cleanup()
  }

  // 总结表格
  console.log('\n📊 Size Performance Summary:')
  console.table(results.map(r => ({
    'File Size': r.name,
    'Time (ms)': r.time.toFixed(0),
    'Patch Size (KB)': (r.size / 1024).toFixed(2),
    'Throughput (MB/s)': r.throughput.toFixed(2),
    'Compression (%)': r.compressionRatio.toFixed(2),
  })))
}

// 测试不同的变化比例
async function benchmarkChangeRatios(): Promise<void> {
  console.log('\n📊 不同变化率性能测试')
  console.log('='.repeat(70))

  const ratios: ChangeRatio[] = [
    { name: '1%', ratio: 0.01 },
    { name: '5%', ratio: 0.05 },
    { name: '10%', ratio: 0.1 },
    { name: '25%', ratio: 0.25 },
    { name: '50%', ratio: 0.5 },
  ]

  const size = 2 * 1024 * 1024 // 2MB
  const results: BenchmarkResult[] = []

  for (const { name, ratio } of ratios) {
    const oldData = generateTestData(size)
    const newData = generateDiffData(oldData, ratio)
    const { oldFile, newFile, patchFile, cleanup } = createTempFiles(oldData, newData, `ratio_${name}`)

    const stats = bsdiff.diffWithStatsSync(oldFile, newFile, patchFile)
    const throughput = (stats.oldSize + stats.newSize) / 1024 / 1024 / (stats.elapsedMs / 1000)

    console.log(`\n🧪 Change Ratio: ${name}`)
    console.log(`   ⏱️  Time: ${formatTime(stats.elapsedMs)}`)
    console.log(`   📦 Patch Size: ${formatFileSize(stats.patchSize)}`)
    console.log(`   📊 Compression: ${stats.compressionRatio.toFixed(2)}%`)

    results.push({
      name,
      time: stats.elapsedMs,
      size: stats.patchSize,
      throughput,
      compressionRatio: stats.compressionRatio,
    })

    cleanup()
  }

  console.log('\n📊 Change Ratio Summary:')
  console.table(results.map(r => ({
    'Change Ratio': r.name,
    'Time (ms)': r.time.toFixed(0),
    'Patch Size (KB)': (r.size / 1024).toFixed(2),
    'Compression (%)': r.compressionRatio.toFixed(2),
  })))
}

// 测试实际文件（React 库）
async function benchmarkRealFiles(): Promise<void> {
  console.log('\n📦 真实文件性能测试 (React)')
  console.log('='.repeat(70))

  const oldFile = path.join(RESOURCES_DIR, 'react-0.3-stable.zip')
  const newFile = path.join(RESOURCES_DIR, 'react-0.4-stable.zip')

  if (!fs.existsSync(oldFile) || !fs.existsSync(newFile)) {
    console.log('⚠️  React test files not found, skipping...')
    return
  }

  const oldSize = fs.statSync(oldFile).size
  const newSize = fs.statSync(newFile).size
  console.log(`\n📁 Files:`)
  console.log(`   Old: ${formatFileSize(oldSize)}`)
  console.log(`   New: ${formatFileSize(newSize)}`)

  const configs: Array<{ name: string; options: DiffOptionsJs }> = [
    { name: 'Fast', options: { compressionLevel: 1, enableParallel: true } },
    { name: 'Default', options: { compressionLevel: 6, enableParallel: true } },
    { name: 'Best', options: { compressionLevel: 9, enableParallel: true } },
  ]

  for (const config of configs) {
    const patchFile = path.join(TEMP_DIR, `react_patch_${config.name.toLowerCase()}.bin`)
    
    console.log(`\n🔧 Configuration: ${config.name}`)
    const stats = bsdiff.diffWithOptionsAndStatsSync(oldFile, newFile, patchFile, config.options)
    
    const throughput = (stats.oldSize + stats.newSize) / 1024 / 1024 / (stats.elapsedMs / 1000)
    
    console.log(`   ⏱️  Time: ${formatTime(stats.elapsedMs)}`)
    console.log(`   📦 Patch Size: ${formatFileSize(stats.patchSize)}`)
    console.log(`   📊 Compression: ${stats.compressionRatio.toFixed(2)}%`)
    console.log(`   🚀 Throughput: ${throughput.toFixed(2)} MB/s`)

    // 测试 Patch 应用
    const appliedFile = path.join(TEMP_DIR, `react_applied_${config.name.toLowerCase()}.bin`)
    const patchStats = bsdiff.patchWithStatsSync(oldFile, appliedFile, patchFile)
    const patchThroughput = (patchStats.oldSize + patchStats.patchSize) / 1024 / 1024 / (patchStats.elapsedMs / 1000)
    
    console.log(`   ⏱️  Patch Time: ${formatTime(patchStats.elapsedMs)}`)
    console.log(`   🚀 Patch Throughput: ${patchThroughput.toFixed(2)} MB/s`)

    // 验证
    const isValid = fs.readFileSync(appliedFile).equals(fs.readFileSync(newFile))
    console.log(`   ✅ Validation: ${isValid ? 'PASSED' : 'FAILED'}`)

    // 清理
    if (fs.existsSync(patchFile)) fs.unlinkSync(patchFile)
    if (fs.existsSync(appliedFile)) fs.unlinkSync(appliedFile)
  }
}

// 并行 vs 顺序对比测试
async function benchmarkParallelVsSequential(): Promise<void> {
  console.log('\n🔀 并行 vs 顺序处理对比')
  console.log('='.repeat(70))

  const sizes = [
    { name: '500KB', size: 500 * 1024 },
    { name: '1MB', size: 1024 * 1024 },
    { name: '2MB', size: 2 * 1024 * 1024 },
    { name: '5MB', size: 5 * 1024 * 1024 },
    { name: '10MB', size: 10 * 1024 * 1024 },
  ]

  for (const { name, size } of sizes) {
    console.log(`\n📏 File Size: ${name}`)
    
    const oldData = generateTestData(size)
    const newData = generateDiffData(oldData, 0.1)
    
    const { oldFile: oldFileP, newFile: newFileP, patchFile: patchFileP, cleanup: cleanupP } = 
      createTempFiles(oldData, newData, `parallel_${name}`)
    const { oldFile: oldFileS, newFile: newFileS, patchFile: patchFileS, cleanup: cleanupS } = 
      createTempFiles(oldData, newData, `sequential_${name}`)

    // 并行处理
    const parallelStats = bsdiff.diffWithOptionsAndStatsSync(oldFileP, newFileP, patchFileP, {
      compressionLevel: 6,
      enableParallel: true,
    })

    // 顺序处理
    const sequentialStats = bsdiff.diffWithOptionsAndStatsSync(oldFileS, newFileS, patchFileS, {
      compressionLevel: 6,
      enableParallel: false,
    })

    const speedup = sequentialStats.elapsedMs / parallelStats.elapsedMs
    const improvement = ((sequentialStats.elapsedMs - parallelStats.elapsedMs) / sequentialStats.elapsedMs * 100)

    console.log(`   🚀 Parallel:   ${formatTime(parallelStats.elapsedMs)}`)
    console.log(`   🐢 Sequential: ${formatTime(sequentialStats.elapsedMs)}`)
    console.log(`   📈 Speedup:    ${speedup.toFixed(2)}x (${improvement.toFixed(1)}% faster)`)

    cleanupP()
    cleanupS()
  }
}

// 主函数
async function main(): Promise<void> {
  console.log('🚀 bsdiff-rust Performance Benchmark Suite')
  console.log('━'.repeat(70))
  
  // 确保临时目录存在
  if (!fs.existsSync(TEMP_DIR)) {
    fs.mkdirSync(TEMP_DIR, { recursive: true })
  }

  try {
    // 1. 配置选项对比
    await benchmarkConfigurations()

    // 2. 不同文件大小
    await benchmarkDifferentSizes()

    // 3. 不同变化率
    await benchmarkChangeRatios()

    // 4. 并行 vs 顺序
    await benchmarkParallelVsSequential()

    // 5. 真实文件测试
    await benchmarkRealFiles()

    console.log('\n✅ All benchmarks completed!')
    console.log('━'.repeat(70))
  } catch (error) {
    console.error('\n❌ Benchmark failed:', error)
    process.exit(1)
  } finally {
    // 清理临时目录
    try {
      fs.rmSync(TEMP_DIR, { recursive: true, force: true })
    } catch {
      // 忽略清理错误
    }
  }
}

// 运行
main().catch((error) => {
  console.error('Fatal error:', error)
  process.exit(1)
})
