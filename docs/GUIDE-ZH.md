# 📖 bsdiff-rust 完整指南

## 目录

- [API 参考](#api-参考)
  - [核心 API](#核心-api)
  - [性能统计 API](#性能统计-api)
  - [高级配置 API](#高级配置-api)
  - [验证工具 API](#验证工具-api)
- [测试](#测试)
- [性能基准](#性能基准)

---

## API 参考

### 核心 API

**同步方法**

```typescript
diffSync(oldFile: string, newFile: string, patchFile: string): void
patchSync(oldFile: string, newFile: string, patchFile: string): void
```

**异步方法**

```typescript
diff(oldFile: string, newFile: string, patchFile: string): Promise<void>
patch(oldFile: string, newFile: string, patchFile: string): Promise<void>
```

**示例**

```javascript
const bsdiff = require('@bsdiff-rust/node')

// 生成和应用补丁
bsdiff.diffSync('old.zip', 'new.zip', 'patch.bin')
bsdiff.patchSync('old.zip', 'result.zip', 'patch.bin')
```

### 性能统计 API

返回 `PerformanceStatsJs` 对象：

```typescript
interface PerformanceStatsJs {
  elapsedMs: number           // 操作耗时（毫秒）
  oldSize: number             // 旧文件大小（字节）
  newSize: number             // 新文件大小（字节）
  patchSize: number           // 补丁大小（字节）
  compressionRatio: number    // 压缩比（百分比）
}
```

**方法**

```typescript
diffWithStatsSync(oldFile: string, newFile: string, patchFile: string): PerformanceStatsJs
diffWithStats(oldFile: string, newFile: string, patchFile: string): Promise<PerformanceStatsJs>
patchWithStatsSync(oldFile: string, newFile: string, patchFile: string): PerformanceStatsJs
patchWithStats(oldFile: string, newFile: string, patchFile: string): Promise<PerformanceStatsJs>
```

**示例**

```javascript
const stats = bsdiff.diffWithStatsSync('old.zip', 'new.zip', 'patch.bin')
console.log(`耗时: ${stats.elapsedMs}ms`)
console.log(`补丁: ${(stats.patchSize / 1024).toFixed(2)} KB`)
console.log(`压缩比: ${stats.compressionRatio.toFixed(2)}%`)
console.log(`吞吐量: ${((stats.oldSize + stats.newSize) / 1024 / 1024 / (stats.elapsedMs / 1000)).toFixed(2)} MB/s`)
```

### 高级配置 API

配置选项：

```typescript
interface DiffOptionsJs {
  compressionLevel?: number  // 压缩级别 (0-9, 默认: 6)
  enableParallel?: boolean   // 启用并行处理（默认: true）
}
```

**方法**

```typescript
diffWithOptionsSync(oldFile: string, newFile: string, patchFile: string, options: DiffOptionsJs): void
diffWithOptions(oldFile: string, newFile: string, patchFile: string, options: DiffOptionsJs): Promise<void>
diffWithOptionsAndStatsSync(oldFile: string, newFile: string, patchFile: string, options: DiffOptionsJs): PerformanceStatsJs
```

**示例**

```javascript
// 快速压缩（开发环境）
await bsdiff.diffWithOptions('old.zip', 'new.zip', 'patch.bin', {
  compressionLevel: 1,
  enableParallel: true
})

// 最佳压缩（生产环境）
await bsdiff.diffWithOptions('old.zip', 'new.zip', 'patch.bin', {
  compressionLevel: 9,
  enableParallel: true
})

// 带性能统计
const stats = bsdiff.diffWithOptionsAndStatsSync('old.zip', 'new.zip', 'patch.bin', {
  compressionLevel: 6,
  enableParallel: true
})
```

### 验证工具 API

**补丁验证**

```typescript
verifyPatchSync(oldFile: string, newFile: string, patchFile: string): boolean
verifyPatch(oldFile: string, newFile: string, patchFile: string): Promise<boolean>
```

**补丁信息**

```typescript
getPatchInfoSync(patchFile: string): PatchInfoJs

interface PatchInfoJs {
  size: number       // 补丁大小（字节）
  compressed: boolean // 是否压缩（BSDIFF40 始终为 true）
}
```

**压缩比分析**

```typescript
getCompressionRatioSync(oldFile: string, newFile: string, patchFile: string): CompressionRatioJs

interface CompressionRatioJs {
  oldSize: number    // 旧文件大小（字节）
  newSize: number    // 新文件大小（字节）
  patchSize: number  // 补丁大小（字节）
  ratio: number      // 压缩比（百分比）
}
```

**工具方法**

```typescript
getFileSizeSync(filePath: string): number
checkFileAccessSync(filePath: string): void
```

### 使用场景

**场景 1: 性能监控**

```javascript
const stats = bsdiff.diffWithStatsSync('old.zip', 'new.zip', 'patch.bin')
console.log(`生成补丁耗时: ${stats.elapsedMs}ms`)
console.log(`补丁大小: ${(stats.patchSize / 1024 / 1024).toFixed(2)} MB`)
console.log(`压缩率: ${stats.compressionRatio.toFixed(2)}%`)

// 计算节省的带宽
const savingsPercent = (1 - stats.patchSize / stats.newSize) * 100
console.log(`相比完整更新节省: ${savingsPercent.toFixed(1)}% 带宽`)
```

**场景 2: 开发环境快速迭代**

```javascript
await bsdiff.diffWithOptions('old.app', 'new.app', 'dev.patch', {
  compressionLevel: 1,  // 最快压缩
  enableParallel: true
})
```

**场景 3: 生产环境优化**

```javascript
const stats = await bsdiff.diffWithOptionsAndStats('v1.zip', 'v2.zip', 'update.patch', {
  compressionLevel: 9,  // 最佳压缩
  enableParallel: true
})

logger.info('Patch generated', {
  time: stats.elapsedMs,
  size: stats.patchSize,
  ratio: stats.compressionRatio
})
```

**场景 4: 完整性验证**

```javascript
bsdiff.diffSync('old.zip', 'new.zip', 'patch.bin')

const isValid = bsdiff.verifyPatchSync('old.zip', 'new.zip', 'patch.bin')
if (isValid) {
  console.log('✅ 补丁验证通过')
} else {
  console.error('❌ 补丁验证失败')
}
```

### 错误处理

```javascript
try {
  bsdiff.diffSync('old.zip', 'new.zip', 'patch.bin')
  console.log('✅ 补丁生成成功')
} catch (error) {
  console.error('❌ 生成失败:', error.message)
}
```

---

## 测试

### 运行测试

```bash
pnpm test              # 功能测试
pnpm run bench         # 性能基准测试
pnpm run test:compatibility  # 兼容性测试
```

### 测试覆盖

- **功能测试**: 同步/异步 API、文件读写、补丁生成和应用
- **错误处理**: 文件不存在、权限错误、无效参数、损坏补丁
- **性能测试**: 不同文件大小、压缩级别、并行处理
- **兼容性**: 与 bsdiff-node 的兼容性、跨平台补丁文件
- **数据完整性**: MD5 校验、文件大小验证
- **工具方法**: 文件操作、压缩比计算、补丁信息

### 编写测试示例

```typescript
import { diffSync, patchSync, diffWithStatsSync } from '@bsdiff-rust/node'
import * as assert from 'assert'
import * as fs from 'fs'

describe('bsdiff test', () => {
  it('should generate and apply patch', () => {
    diffSync('old.zip', 'new.zip', 'test.patch')
    assert.ok(fs.existsSync('test.patch'))
    
    patchSync('old.zip', 'result.zip', 'test.patch')
    
    const expected = fs.readFileSync('new.zip')
    const actual = fs.readFileSync('result.zip')
    assert.deepStrictEqual(actual, expected)
  })
  
  it('should track performance', () => {
    const stats = diffWithStatsSync('old.zip', 'new.zip', 'test.patch')
    assert.ok(stats.elapsedMs < 5000)
    assert.ok(stats.compressionRatio < 100)
  })
  
  it('should throw on missing file', () => {
    assert.throws(() => {
      diffSync('nonexistent.zip', 'new.zip', 'test.patch')
    }, /file not found/i)
  })
})
```

---

## 性能基准

### 测试环境

- **测试文件**: React 库 (1.31 MB → 1.86 MB)
- **平台**: Linux x64
- **Node.js**: 20.x

### 测试结果

**Diff 性能（补丁生成）**

| 配置 | 时间 | 补丁大小 | 吞吐量 | 说明 |
|------|------|----------|--------|------|
| 默认 (级别 6, 并行) | 201 ms | 781.56 KB | 15.78 MB/s | 最佳平衡 |
| 快速 (级别 1, 并行) | 221 ms | 783.99 KB | 14.36 MB/s | 更快, 体积仅增 0.3% |
| 最佳 (级别 9, 并行) | 217 ms | 780.96 KB | 14.62 MB/s | 最小, 体积仅减 0.1% |
| 顺序 (级别 6, 串行) | 287 ms | 781.56 KB | 11.06 MB/s | 性能降低 30% |

**Patch 性能（补丁应用）**

- 时间: 50-59 ms
- 吞吐量: 35-41 MB/s
- 应用补丁比生成快约 3-4 倍

### 关键发现

1. **压缩级别**: 级别 1-9 补丁大小差异不到 0.5%，推荐使用默认级别 6
2. **并行处理**: 大文件 (>1MB) 性能提升显著，小文件 (<500KB) 并行开销大于收益
3. **vs bsdiff-node**: Diff 速度提升 32.7%，Patch 速度提升 93.0%，内存使用降低 75%

### 配置建议

**按文件大小选择**

| 文件大小 | 压缩级别 | 并行处理 | 原因 |
|---------|---------|---------|------|
| < 100KB | 1-3 | false | 小文件并行开销大 |
| 100KB - 1MB | 6 | true | 平衡性能和大小 |
| 1MB - 10MB | 6 | true | 并行优势明显 |
| > 10MB | 6-9 | true | 大文件推荐更高压缩 |

**按使用场景选择**

| 场景 | 推荐配置 | 说明 |
|------|---------|------|
| 开发测试 | 级别 1, 并行开启 | 快速迭代 |
| 生产环境 | 级别 6, 并行开启 | 最佳平衡 |
| 软件分发 | 级别 9, 并行开启 | 最小补丁 |

### 运行基准测试

```bash
pnpm run bench
```

**自定义基准测试**

```typescript
import { diffWithOptionsAndStatsSync } from '@bsdiff-rust/node'

const configs = [
  { compressionLevel: 1, enableParallel: true },
  { compressionLevel: 6, enableParallel: true },
  { compressionLevel: 9, enableParallel: true },
]

configs.forEach(config => {
  const stats = diffWithOptionsAndStatsSync('old.zip', 'new.zip', 'test.patch', config)
  console.log(`配置: ${JSON.stringify(config)}`)
  console.log(`时间: ${stats.elapsedMs}ms`)
  console.log(`大小: ${(stats.patchSize / 1024).toFixed(2)} KB`)
})
```

### 性能监控

```typescript
// 设置性能基准
const BASELINE = {
  maxDiffTime: 250,   // ms
  maxPatchTime: 70,   // ms
}

it('should not regress performance', () => {
  const stats = diffWithStatsSync('old.zip', 'new.zip', 'test.patch')
  assert.ok(stats.elapsedMs < BASELINE.maxDiffTime)
  
  const patchStats = patchWithStatsSync('old.zip', 'result.zip', 'test.patch')
  assert.ok(patchStats.elapsedMs < BASELINE.maxPatchTime)
})
```

---

## 相关资源

- [主 README](../README-ZH.md)
- [GitHub 仓库](https://github.com/Sphinm/bsdiff-rust)
- [npm 包](https://www.npmjs.com/package/@bsdiff-rust/node)

