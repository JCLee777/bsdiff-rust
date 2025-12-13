# 🚀 Benchmark Suite

Comprehensive performance benchmark suite for bsdiff-rust, testing various configurations, file sizes, and real-world scenarios.

## 📊 What's Tested

### 1. Configuration Options Comparison
Tests different compression levels and parallel processing:
- **Fast** (level 1, parallel) - Fastest compression
- **Default** (level 6, parallel) - Balanced performance
- **Best** (level 9, parallel) - Best compression
- **Sequential** (level 6, no parallel) - Single-threaded

### 2. Different File Sizes
Tests performance across various file sizes:
- 100KB
- 500KB
- 1MB
- 2MB
- 5MB
- 10MB

Measures both diff and patch performance with throughput calculations.

### 3. Different Change Ratios
Tests how change percentage affects performance:
- 1% changes
- 5% changes
- 10% changes
- 25% changes
- 50% changes

### 4. Parallel vs Sequential
Compares parallel processing against sequential for different file sizes.

### 5. Real-World Files
Tests with actual React library versions:
- React 0.3-stable.zip → 0.4-stable.zip
- Tests all three compression configurations
- Validates patch application

## 🏃 Running Benchmarks

```bash
# Install dependencies first
pnpm install

# Build the native module
pnpm run build

# Run full benchmark suite
pnpm run bench
```

## 📈 Key Metrics

Each benchmark provides:
- **⏱️ Time**: Operation duration in milliseconds
- **📦 Patch Size**: Generated patch file size
- **📊 Compression Ratio**: Patch size as percentage of total input
- **🚀 Throughput**: MB/s processing speed
- **✅ Validation**: Correctness verification

## 📋 Sample Results

### Configuration Comparison (2MB file)
| Configuration | Time | Throughput | Use Case |
|---------------|------|------------|----------|
| Fast (level 1) | 58ms | 68.97 MB/s | Development |
| Default (level 6) | 52ms | 76.92 MB/s | Production |
| Best (level 9) | 51ms | 78.43 MB/s | Distribution |

### Real-World Performance (React)
| Config | Diff Time | Patch Time | Patch Size | Throughput |
|--------|-----------|------------|------------|------------|
| Fast | 221ms | 50ms | 783.99 KB | 14.36 MB/s |
| Default | 201ms | 57ms | 781.56 KB | 15.78 MB/s |
| Best | 217ms | 59ms | 780.96 KB | 14.62 MB/s |

### File Size Scaling
| Size | Diff Time | Patch Time | Throughput |
|------|-----------|------------|------------|
| 100KB | 2ms | 1ms | 97.66 MB/s |
| 1MB | 27ms | 4ms | 74.07 MB/s |
| 5MB | 156ms | 21ms | 64.10 MB/s |
| 10MB | 365ms | 35ms | 54.79 MB/s |

## 🎯 Key Insights

1. **Compression Levels**
   - Fast (level 1): Minimal speed advantage for synthetic data
   - Default (level 6): Best balance for most scenarios
   - Best (level 9): Negligible size improvement (<1%)

2. **Parallel Processing**
   - Benefits vary based on file size and content
   - Most beneficial for real-world files (React example)
   - Overhead may reduce benefits for small or simple files

3. **Patch vs Diff**
   - Patching is significantly faster than generating (7-10x)
   - Patch throughput increases with file size
   - Very efficient for deployment scenarios

4. **Real-World Performance**
   - Default configuration offers best balance
   - Throughput: 14-15 MB/s for diff, 35-41 MB/s for patch
   - Patch application is production-ready fast

## 🔧 Customization

Modify `benchmark.ts` to:
- Add new test scenarios
- Adjust file sizes
- Change compression levels
- Test different data patterns


