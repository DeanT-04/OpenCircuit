# Ollama Model Selection Guide

## Overview

This guide helps you choose the right Ollama models for OpenCircuit based on your hardware capabilities, use case requirements, and performance needs. We follow a progressive testing approach, starting with ultra-lightweight models.

## Testing Strategy: Start Small, Scale Up

### Phase 1: Ultra-Lightweight Testing (Recommended Start)
Begin with the smallest possible model to validate the integration and basic functionality.

**Primary Test Model:**
- **qwen2.5:0.5b** (500MB)
  - Fastest download and startup
  - Minimal memory footprint
  - Perfect for integration testing
  - Basic circuit knowledge validation

### Phase 2: Production Validation
Once ultra-lightweight testing succeeds, move to production-ready models.

**Recommended Production Model:**
- **qwen2.5:1b** (1GB)
  - Balanced performance and size
  - Good circuit design knowledge
  - Suitable for most OpenCircuit tasks

### Phase 3: Advanced Features
For complex circuit analysis and advanced AI features.

**Advanced Models:**
- **qwen2.5:3b** (3GB)
  - Superior reasoning capabilities
  - Complex circuit analysis
  - Advanced component recommendations

## Model Categories

### 1. Ultra-Lightweight Models (< 1GB)

#### qwen2.5:0.5b
- **Size:** 500MB
- **Memory:** ~1GB RAM
- **Use Case:** Initial testing, basic queries
- **Strengths:** Fast, minimal resources
- **Limitations:** Basic reasoning, limited context

```powershell
ollama pull qwen2.5:0.5b
```

#### phi3:mini
- **Size:** 2.3GB
- **Memory:** ~3GB RAM
- **Use Case:** Alternative lightweight option
- **Strengths:** Microsoft-optimized, good efficiency
- **Limitations:** Larger than qwen2.5:0.5b

```powershell
ollama pull phi3:mini
```

### 2. Balanced Models (1-3GB)

#### qwen2.5:1b
- **Size:** 1GB
- **Memory:** ~2GB RAM
- **Use Case:** Production deployment
- **Strengths:** Good balance of size and capability
- **Best For:** General circuit assistance, component lookup

```powershell
ollama pull qwen2.5:1b
```

#### qwen2.5-coder:1.5b
- **Size:** 1.5GB
- **Memory:** ~2.5GB RAM
- **Use Case:** Code generation, Rust development
- **Strengths:** Specialized for programming tasks
- **Best For:** Generating circuit simulation code

```powershell
ollama pull qwen2.5-coder:1.5b
```

#### gemma2:2b
- **Size:** 2GB
- **Memory:** ~3GB RAM
- **Use Case:** Alternative to Qwen models
- **Strengths:** Google-developed, good reasoning
- **Best For:** Circuit analysis, design recommendations

```powershell
ollama pull gemma2:2b
```

### 3. Advanced Models (3GB+)

#### qwen2.5:3b
- **Size:** 3GB
- **Memory:** ~4-5GB RAM
- **Use Case:** Complex circuit design, advanced analysis
- **Strengths:** Superior reasoning, larger context
- **Best For:** Complex PCB layout, multi-stage circuits

```powershell
ollama pull qwen2.5:3b
```

#### qwen2.5:7b
- **Size:** 7GB
- **Memory:** ~8-10GB RAM
- **Use Case:** Research, complex engineering tasks
- **Strengths:** Excellent reasoning, comprehensive knowledge
- **Best For:** Advanced circuit optimization, research

```powershell
ollama pull qwen2.5:7b
```

## Hardware Requirements

### Minimum System (Ultra-Lightweight)
- **RAM:** 4GB total (2GB available)
- **Storage:** 1GB free space
- **CPU:** Any modern processor
- **Recommended Model:** qwen2.5:0.5b

### Standard System (Balanced)
- **RAM:** 8GB total (4GB available)
- **Storage:** 3GB free space
- **CPU:** Multi-core processor
- **Recommended Model:** qwen2.5:1b or gemma2:2b

### High-Performance System (Advanced)
- **RAM:** 16GB+ total (8GB+ available)
- **Storage:** 10GB+ free space
- **CPU:** High-performance multi-core
- **GPU:** Optional (CUDA/ROCm support)
- **Recommended Model:** qwen2.5:3b or larger

### GPU Acceleration (Optional)
- **NVIDIA:** RTX 3060+ with 8GB+ VRAM
- **AMD:** RX 6700 XT+ with 8GB+ VRAM
- **Benefits:** Faster inference, larger models possible

## Use Case Mapping

### Circuit Design Assistant
**Recommended:** qwen2.5:1b
- Component recommendations
- Basic circuit analysis
- Design pattern suggestions
- Troubleshooting guidance

### Code Generation
**Recommended:** qwen2.5-coder:1.5b
- Rust code generation
- Simulation scripts
- Test case creation
- API integration code

### Advanced Analysis
**Recommended:** qwen2.5:3b
- Complex circuit optimization
- Multi-layer PCB design
- Signal integrity analysis
- Thermal considerations

### Research and Development
**Recommended:** qwen2.5:7b
- Cutting-edge circuit techniques
- Research paper analysis
- Novel component evaluation
- Advanced optimization

## Performance Comparison

### Response Time (Approximate)
| Model | Simple Query | Complex Analysis | Code Generation |
|-------|-------------|------------------|-----------------|
| qwen2.5:0.5b | 1-2s | 5-10s | 3-8s |
| qwen2.5:1b | 2-3s | 8-15s | 5-12s |
| qwen2.5:3b | 3-5s | 15-30s | 10-20s |
| qwen2.5:7b | 5-10s | 30-60s | 20-40s |

### Memory Usage
| Model | Idle | Active | Peak |
|-------|------|--------|------|
| qwen2.5:0.5b | 800MB | 1.2GB | 1.5GB |
| qwen2.5:1b | 1.5GB | 2.2GB | 2.8GB |
| qwen2.5:3b | 3.5GB | 4.5GB | 5.5GB |
| qwen2.5:7b | 7.5GB | 9GB | 11GB |

## Model Selection Workflow

### Step 1: Assess Your Hardware
```powershell
# Check available RAM
Get-WmiObject -Class Win32_OperatingSystem | Select-Object TotalVisibleMemorySize,FreePhysicalMemory

# Check available storage
Get-WmiObject -Class Win32_LogicalDisk | Select-Object DeviceID,Size,FreeSpace
```

### Step 2: Start with Ultra-Lightweight
```powershell
# Always start here for testing
ollama pull qwen2.5:0.5b
ollama run qwen2.5:0.5b "Test basic circuit knowledge"
```

### Step 3: Validate Integration
Test the model with OpenCircuit-specific queries:
```powershell
ollama run qwen2.5:0.5b "What is the purpose of a bypass capacitor?"
ollama run qwen2.5:0.5b "Calculate the resistance for an LED with 3.3V supply and 20mA current"
```

### Step 4: Scale Up Based on Results
If ultra-lightweight works well:
```powershell
# Move to balanced model
ollama pull qwen2.5:1b
ollama rm qwen2.5:0.5b  # Free space if needed
```

### Step 5: Optimize for Use Case
Choose specialized models based on primary use:
```powershell
# For code-heavy workflows
ollama pull qwen2.5-coder:1.5b

# For complex analysis
ollama pull qwen2.5:3b
```

## Model Management Best Practices

### Storage Optimization
```powershell
# List all models with sizes
ollama list

# Remove unused models
ollama rm model-name

# Keep only essential models
ollama rm qwen2.5:0.5b  # After upgrading to 1b
```

### Performance Monitoring
```powershell
# Monitor memory usage during inference
Get-Process ollama | Select-Object ProcessName,WorkingSet,PagedMemorySize

# Time model responses
Measure-Command { ollama run qwen2.5:1b "Quick test" }
```

### Fallback Strategy
Configure multiple models for different scenarios:
1. **Primary:** qwen2.5:1b (general use)
2. **Fallback:** qwen2.5:0.5b (low memory situations)
3. **Specialized:** qwen2.5-coder:1.5b (code generation)

## Troubleshooting Model Issues

### Model Too Large for System
```powershell
# Switch to smaller model
ollama rm qwen2.5:3b
ollama pull qwen2.5:1b
```

### Slow Performance
```powershell
# Reduce parallel requests
$env:OLLAMA_NUM_PARALLEL='1'

# Enable Flash Attention
$env:OLLAMA_FLASH_ATTENTION='1'

# Use smaller model
ollama pull qwen2.5:0.5b
```

### Out of Memory Errors
```powershell
# Limit loaded models
$env:OLLAMA_MAX_LOADED_MODELS='1'

# Restart Ollama service
taskkill /f /im ollama.exe
ollama serve
```

## Future Model Considerations

### Upcoming Models to Watch
- **Qwen 3.0 series** - Next generation improvements
- **Gemma 3 models** - Google's latest developments
- **Phi-4** - Microsoft's efficiency improvements

### Model Update Strategy
1. Test new models in development environment
2. Validate with OpenCircuit use cases
3. Compare performance against current models
4. Gradual rollout to production

## Conclusion

Start your OpenCircuit integration with **qwen2.5:0.5b** for initial testing and validation. This ultra-lightweight approach ensures:

- Quick setup and testing
- Minimal resource requirements
- Fast iteration and debugging
- Clear upgrade path

Once validated, scale up to **qwen2.5:1b** for production use, with the option to specialize with **qwen2.5-coder:1.5b** for code generation or **qwen2.5:3b** for advanced analysis.

Remember: The best model is the one that meets your performance requirements while running reliably on your hardware.