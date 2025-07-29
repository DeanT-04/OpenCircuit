# Ollama Setup and Configuration Guide

## Overview

This guide covers the complete setup and configuration of Ollama for OpenCircuit's local AI integration. Ollama provides a simple way to run large language models locally, ensuring privacy, cost-effectiveness, and offline functionality.

## Installation

### Windows Installation

```powershell
# Download and install Ollama
Invoke-WebRequest -Uri "https://ollama.ai/download/windows" -OutFile "ollama-installer.exe"
Start-Process -FilePath "ollama-installer.exe" -Wait

# Refresh environment variables
refreshenv

# Verify installation
ollama --version
```

### Alternative Installation Methods

#### Using Chocolatey
```powershell
choco install ollama
```

#### Manual Download
Visit [ollama.ai](https://ollama.ai) and download the Windows installer directly.

## Initial Setup

### Start Ollama Service
```powershell
# Start the Ollama server
ollama serve
```

The server will start on `http://localhost:11434` by default.

### Pull Ultra-Lightweight Model for Testing
```powershell
# Pull the smallest model for initial testing (500MB)
ollama pull qwen2.5:0.5b

# Verify model installation
ollama list

# Test basic functionality
ollama run qwen2.5:0.5b "Hello, can you help with circuit design?"
```

## Model Management

### Recommended Models for OpenCircuit

#### Ultra-Lightweight (Testing)
- **qwen2.5:0.5b** (500MB) - For initial testing and validation
- **qwen2.5:1b** (1GB) - Balanced performance for production

#### Production Models
- **qwen2.5:3b** (3GB) - Advanced reasoning for complex circuits
- **qwen2.5-coder:1.5b** (1.5GB) - Specialized for code generation

#### Specialized Models
- **gemma2:2b** (2GB) - Alternative lightweight option
- **phi3:mini** (2.3GB) - Microsoft's efficient model

### Model Operations

```powershell
# List available models
ollama list

# Pull a specific model
ollama pull qwen2.5:1b

# Remove a model to free space
ollama rm qwen2.5:0.5b

# Show model information
ollama show qwen2.5:1b

# Copy a model
ollama cp qwen2.5:1b my-custom-model
```

## Configuration

### Environment Variables

Create or update your `.env` file:

```env
# Ollama Configuration
OLLAMA_HOST=http://localhost:11434
OLLAMA_MODEL=qwen2.5:0.5b
OLLAMA_MODELS=C:\Users\%USERNAME%\.ollama\models
OLLAMA_KEEP_ALIVE=5m
OLLAMA_MAX_LOADED_MODELS=3
OLLAMA_NUM_PARALLEL=1
```

### Advanced Configuration

#### Custom Models Directory
```powershell
# Set custom models directory
$env:OLLAMA_MODELS='C:\path\to\your\models'
ollama serve
```

#### Performance Tuning
```powershell
# Enable Flash Attention for memory efficiency
$env:OLLAMA_FLASH_ATTENTION='1'

# Increase parallel requests
$env:OLLAMA_NUM_PARALLEL='2'

# Adjust keep-alive duration
$env:OLLAMA_KEEP_ALIVE='10m'
```

#### Memory Management
```powershell
# Limit maximum loaded models
$env:OLLAMA_MAX_LOADED_MODELS='2'

# Set request queue size
$env:OLLAMA_MAX_QUEUE='256'
```

## Testing and Validation

### Basic Functionality Test
```powershell
# Test text generation
ollama run qwen2.5:0.5b "Explain how a resistor works in simple terms"

# Test circuit-specific knowledge
ollama run qwen2.5:0.5b "What is the purpose of a decoupling capacitor?"

# Test code generation
ollama run qwen2.5-coder:1.5b "Write a Rust function to calculate resistor values"
```

### Performance Benchmarking
```powershell
# Time a simple query
Measure-Command { ollama run qwen2.5:0.5b "Hello world" }

# Test streaming response
ollama run qwen2.5:0.5b "Count from 1 to 10" --stream
```

## Integration with OpenCircuit

### Rust Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
ollama-rs = "0.3.1"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### Basic Client Setup

```rust
use ollama_rs::Ollama;

// Initialize Ollama client
let ollama = Ollama::default(); // Connects to localhost:11434

// For custom configuration
let ollama = Ollama::new("http://localhost".to_string(), 11434);
```

### Environment Integration

```rust
use std::env;

fn get_ollama_config() -> (String, u16, String) {
    let host = env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost".to_string());
    let port = env::var("OLLAMA_PORT")
        .unwrap_or_else(|_| "11434".to_string())
        .parse()
        .unwrap_or(11434);
    let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:0.5b".to_string());
    
    (host, port, model)
}
```

## Troubleshooting

### Common Issues

#### Ollama Service Not Starting
```powershell
# Check if port is in use
netstat -an | findstr :11434

# Kill existing processes
taskkill /f /im ollama.exe

# Restart service
ollama serve
```

#### Model Download Failures
```powershell
# Check internet connection
ping ollama.ai

# Clear model cache
Remove-Item -Recurse -Force "$env:USERPROFILE\.ollama\models\*"

# Retry download
ollama pull qwen2.5:0.5b
```

#### Memory Issues
```powershell
# Check available memory
Get-WmiObject -Class Win32_OperatingSystem | Select-Object TotalVisibleMemorySize,FreePhysicalMemory

# Use smaller model
ollama pull qwen2.5:0.5b

# Reduce parallel requests
$env:OLLAMA_NUM_PARALLEL='1'
```

### Performance Optimization

#### For Low-Memory Systems
- Use `qwen2.5:0.5b` (500MB)
- Set `OLLAMA_MAX_LOADED_MODELS=1`
- Enable `OLLAMA_FLASH_ATTENTION=1`

#### For High-Performance Systems
- Use `qwen2.5:3b` or larger models
- Increase `OLLAMA_NUM_PARALLEL`
- Load multiple models simultaneously

## Security Considerations

### Network Security
- Ollama runs on localhost by default
- No external API calls or data transmission
- All processing happens locally

### Data Privacy
- No telemetry or usage tracking
- Models and conversations stay on your machine
- No internet connection required after model download

### Model Verification
```powershell
# Verify model integrity
ollama show qwen2.5:0.5b --verbose

# Check model source
ollama show qwen2.5:0.5b | findstr "From"
```

## Next Steps

1. **Test Ultra-Lightweight Model**: Start with `qwen2.5:0.5b` for validation
2. **Integrate with OpenCircuit**: Use `ollama-rs` for Rust integration
3. **Scale Up**: Move to larger models based on performance needs
4. **Optimize**: Tune configuration for your hardware
5. **Monitor**: Track performance and adjust settings

For detailed integration examples, see the [Ollama Integration Guide](./ollama_integration.md).