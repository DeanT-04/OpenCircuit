# Ollama Performance Optimization Guide

## Overview

This guide provides comprehensive strategies for optimizing Ollama performance in OpenCircuit, focusing on the ultra-lightweight model testing approach and progressive scaling strategy.

## Hardware Requirements and Optimization

### Minimum Requirements for Ultra-Lightweight Testing

**For qwen2.5:0.5b (Initial Testing):**
- **RAM:** 1GB minimum, 2GB recommended
- **Storage:** 500MB for model + 1GB working space
- **CPU:** Any modern x64 processor (2+ cores recommended)
- **GPU:** Optional (CPU-only operation supported)

**For qwen2.5:1b (Production):**
- **RAM:** 2GB minimum, 4GB recommended
- **Storage:** 800MB for model + 2GB working space
- **CPU:** 4+ cores recommended for responsive performance
- **GPU:** Optional but beneficial for faster inference

**For qwen2.5:3b (Advanced Features):**
- **RAM:** 4GB minimum, 8GB recommended
- **Storage:** 2GB for model + 4GB working space
- **CPU:** 6+ cores recommended
- **GPU:** Highly recommended (4GB+ VRAM)

### Memory Optimization

#### System Configuration

```bash
# Windows PowerShell - Optimize virtual memory
# Set page file to system managed or 2x RAM size
wmic computersystem where name="%computername%" set AutomaticManagedPagefile=True

# Set Ollama memory limits
$env:OLLAMA_MAX_LOADED_MODELS = "1"
$env:OLLAMA_NUM_PARALLEL = "1"
$env:OLLAMA_KEEP_ALIVE = "5m"
```

#### Model-Specific Memory Settings

```bash
# Ultra-lightweight (qwen2.5:0.5b)
OLLAMA_NUM_CTX=1024          # Smaller context window
OLLAMA_NUM_BATCH=1           # Single batch processing
OLLAMA_NUM_THREAD=2          # Limit thread usage

# Balanced (qwen2.5:1b)
OLLAMA_NUM_CTX=2048          # Standard context
OLLAMA_NUM_BATCH=4           # Small batch size
OLLAMA_NUM_THREAD=4          # Moderate threading

# Advanced (qwen2.5:3b)
OLLAMA_NUM_CTX=4096          # Large context
OLLAMA_NUM_BATCH=8           # Larger batches
OLLAMA_NUM_THREAD=8          # Full threading
```

### CPU Optimization

#### Thread Configuration

```rust
// Rust configuration for optimal CPU usage
use std::env;

pub struct PerformanceConfig {
    pub num_threads: usize,
    pub batch_size: usize,
    pub context_size: usize,
}

impl PerformanceConfig {
    pub fn for_model(model_size: &str) -> Self {
        match model_size {
            "0.5b" => Self {
                num_threads: 2,
                batch_size: 1,
                context_size: 1024,
            },
            "1b" => Self {
                num_threads: 4,
                batch_size: 4,
                context_size: 2048,
            },
            "3b" => Self {
                num_threads: 8,
                batch_size: 8,
                context_size: 4096,
            },
            _ => Self::default(),
        }
    }
    
    pub fn apply_to_environment(&self) {
        env::set_var("OLLAMA_NUM_THREAD", self.num_threads.to_string());
        env::set_var("OLLAMA_NUM_BATCH", self.batch_size.to_string());
        env::set_var("OLLAMA_NUM_CTX", self.context_size.to_string());
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            num_threads: 4,
            batch_size: 4,
            context_size: 2048,
        }
    }
}
```

#### CPU Affinity (Windows)

```powershell
# Set Ollama process to use specific CPU cores
$process = Get-Process "ollama" -ErrorAction SilentlyContinue
if ($process) {
    # Use cores 0-3 for better performance isolation
    $process.ProcessorAffinity = 15  # Binary: 1111 (cores 0,1,2,3)
}
```

### GPU Acceleration

#### NVIDIA GPU Setup

```bash
# Check GPU availability
nvidia-smi

# Configure Ollama for GPU usage
OLLAMA_NUM_GPU=1             # Use 1 GPU layer
OLLAMA_MAIN_GPU=0            # Primary GPU index
OLLAMA_GPU_MEMORY_FRACTION=0.8  # Use 80% of GPU memory
```

#### AMD GPU Setup (ROCm)

```bash
# Check ROCm installation
rocm-smi

# Configure for AMD GPU
OLLAMA_NUM_GPU=1
HSA_OVERRIDE_GFX_VERSION=10.3.0  # Adjust for your GPU
```

#### Intel GPU Setup

```bash
# Intel Arc/Xe GPU support
OLLAMA_NUM_GPU=1
ONEAPI_ROOT=/opt/intel/oneapi
```

## Model Loading and Management

### Progressive Model Loading Strategy

```rust
use ollama_rs::Ollama;
use std::time::{Duration, Instant};
use anyhow::Result;

pub struct ModelManager {
    ollama: Ollama,
    current_model: Option<String>,
    model_load_times: std::collections::HashMap<String, Duration>,
}

impl ModelManager {
    pub fn new(ollama: Ollama) -> Self {
        Self {
            ollama,
            current_model: None,
            model_load_times: std::collections::HashMap::new(),
        }
    }
    
    pub async fn load_model_progressively(&mut self) -> Result<String> {
        // Start with ultra-lightweight for testing
        let models = vec!["qwen2.5:0.5b", "qwen2.5:1b", "qwen2.5:3b"];
        
        for model in models {
            match self.test_model_performance(model).await {
                Ok(load_time) => {
                    println!("Model {} loaded successfully in {:?}", model, load_time);
                    self.model_load_times.insert(model.to_string(), load_time);
                    self.current_model = Some(model.to_string());
                    return Ok(model.to_string());
                }
                Err(e) => {
                    println!("Model {} failed to load: {}", model, e);
                    continue;
                }
            }
        }
        
        Err(anyhow::anyhow!("No suitable model could be loaded"))
    }
    
    async fn test_model_performance(&self, model: &str) -> Result<Duration> {
        let start = Instant::now();
        
        // Test with a simple circuit query
        let test_prompt = "What is Ohm's law?";
        let request = ollama_rs::generation::completion::GenerationRequest::new(
            model.to_string(),
            test_prompt.to_string()
        );
        
        let _response = self.ollama.generate(request).await?;
        let load_time = start.elapsed();
        
        // Performance thresholds
        let max_acceptable_time = match model {
            "qwen2.5:0.5b" => Duration::from_secs(10),
            "qwen2.5:1b" => Duration::from_secs(15),
            "qwen2.5:3b" => Duration::from_secs(30),
            _ => Duration::from_secs(20),
        };
        
        if load_time > max_acceptable_time {
            return Err(anyhow::anyhow!("Model load time exceeded threshold"));
        }
        
        Ok(load_time)
    }
    
    pub fn get_optimal_model(&self) -> Option<&String> {
        // Return the largest model that loaded successfully
        for model in ["qwen2.5:3b", "qwen2.5:1b", "qwen2.5:0.5b"] {
            if self.model_load_times.contains_key(model) {
                return self.model_load_times.keys().find(|k| k.as_str() == model);
            }
        }
        None
    }
}
```

### Model Preloading and Caching

```rust
pub struct ModelCache {
    preloaded_models: Vec<String>,
    cache_strategy: CacheStrategy,
}

#[derive(Debug, Clone)]
pub enum CacheStrategy {
    KeepAll,
    KeepRecent(usize),
    KeepBySize(usize), // MB
}

impl ModelCache {
    pub async fn preload_essential_models(&mut self, ollama: &Ollama) -> Result<()> {
        let essential_models = vec!["qwen2.5:0.5b"];
        
        for model in essential_models {
            match self.warm_up_model(ollama, model).await {
                Ok(_) => {
                    self.preloaded_models.push(model.to_string());
                    println!("Preloaded model: {}", model);
                }
                Err(e) => {
                    println!("Failed to preload {}: {}", model, e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn warm_up_model(&self, ollama: &Ollama, model: &str) -> Result<()> {
        // Send a minimal request to load the model into memory
        let request = ollama_rs::generation::completion::GenerationRequest::new(
            model.to_string(),
            "Hi".to_string()
        );
        
        let _response = ollama.generate(request).await?;
        Ok(())
    }
}
```

## Request Optimization

### Batching and Queuing

```rust
use tokio::sync::mpsc;
use std::collections::VecDeque;

pub struct RequestBatcher {
    batch_size: usize,
    batch_timeout: Duration,
    pending_requests: VecDeque<CircuitRequest>,
    sender: mpsc::Sender<Vec<CircuitRequest>>,
}

#[derive(Debug, Clone)]
pub struct CircuitRequest {
    pub id: String,
    pub prompt: String,
    pub priority: RequestPriority,
    pub response_sender: tokio::sync::oneshot::Sender<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl RequestBatcher {
    pub fn new(batch_size: usize, batch_timeout: Duration) -> Self {
        let (sender, mut receiver) = mpsc::channel(100);
        
        // Spawn batch processor
        tokio::spawn(async move {
            while let Some(batch) = receiver.recv().await {
                Self::process_batch(batch).await;
            }
        });
        
        Self {
            batch_size,
            batch_timeout,
            pending_requests: VecDeque::new(),
            sender,
        }
    }
    
    pub async fn add_request(&mut self, request: CircuitRequest) -> Result<()> {
        self.pending_requests.push_back(request);
        
        if self.pending_requests.len() >= self.batch_size {
            self.flush_batch().await?;
        }
        
        Ok(())
    }
    
    async fn flush_batch(&mut self) -> Result<()> {
        if self.pending_requests.is_empty() {
            return Ok(());
        }
        
        let batch: Vec<_> = self.pending_requests.drain(..).collect();
        self.sender.send(batch).await?;
        
        Ok(())
    }
    
    async fn process_batch(mut batch: Vec<CircuitRequest>) {
        // Sort by priority
        batch.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Process high-priority requests first
        for request in batch {
            // Process individual request
            // This would integrate with your Ollama client
            println!("Processing request: {}", request.id);
        }
    }
}
```

### Connection Pooling

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct OllamaConnectionPool {
    clients: Vec<Ollama>,
    semaphore: Arc<Semaphore>,
    current_index: std::sync::atomic::AtomicUsize,
}

impl OllamaConnectionPool {
    pub fn new(pool_size: usize, host: String, port: u16) -> Self {
        let clients: Vec<_> = (0..pool_size)
            .map(|_| Ollama::new(host.clone(), port))
            .collect();
        
        Self {
            clients,
            semaphore: Arc::new(Semaphore::new(pool_size)),
            current_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }
    
    pub async fn get_client(&self) -> Result<&Ollama> {
        let _permit = self.semaphore.acquire().await?;
        
        let index = self.current_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed) 
            % self.clients.len();
        
        Ok(&self.clients[index])
    }
}
```

## Response Optimization

### Streaming with Backpressure

```rust
use tokio_stream::{Stream, StreamExt};
use std::pin::Pin;

pub struct OptimizedStreaming {
    ollama: Ollama,
    buffer_size: usize,
    max_concurrent: usize,
}

impl OptimizedStreaming {
    pub fn new(ollama: Ollama) -> Self {
        Self {
            ollama,
            buffer_size: 1024,
            max_concurrent: 3,
        }
    }
    
    pub async fn stream_with_backpressure(
        &self,
        request: ollama_rs::generation::completion::GenerationRequest
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let mut stream = self.ollama.generate_stream(request).await?;
        
        let buffered_stream = async_stream::stream! {
            let mut buffer = String::with_capacity(self.buffer_size);
            
            while let Some(result) = stream.next().await {
                match result {
                    Ok(responses) => {
                        for response in responses {
                            buffer.push_str(&response.response);
                            
                            // Yield when buffer is full or response is complete
                            if buffer.len() >= self.buffer_size || response.done {
                                yield Ok(buffer.clone());
                                buffer.clear();
                            }
                        }
                    }
                    Err(e) => yield Err(e),
                }
            }
            
            // Yield any remaining content
            if !buffer.is_empty() {
                yield Ok(buffer);
            }
        };
        
        Ok(Box::pin(buffered_stream))
    }
}
```

## Monitoring and Metrics

### Performance Monitoring

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub model_name: String,
    pub request_count: u64,
    pub total_response_time: Duration,
    pub average_response_time: Duration,
    pub tokens_per_second: f64,
    pub memory_usage_mb: f64,
    pub error_count: u64,
}

pub struct MetricsCollector {
    metrics: HashMap<String, PerformanceMetrics>,
    start_time: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            start_time: Instant::now(),
        }
    }
    
    pub fn record_request(&mut self, model: &str, response_time: Duration, tokens: usize, error: bool) {
        let metrics = self.metrics.entry(model.to_string()).or_insert_with(|| {
            PerformanceMetrics {
                model_name: model.to_string(),
                request_count: 0,
                total_response_time: Duration::ZERO,
                average_response_time: Duration::ZERO,
                tokens_per_second: 0.0,
                memory_usage_mb: 0.0,
                error_count: 0,
            }
        });
        
        metrics.request_count += 1;
        metrics.total_response_time += response_time;
        metrics.average_response_time = metrics.total_response_time / metrics.request_count as u32;
        
        if response_time.as_secs_f64() > 0.0 {
            metrics.tokens_per_second = tokens as f64 / response_time.as_secs_f64();
        }
        
        if error {
            metrics.error_count += 1;
        }
    }
    
    pub fn get_metrics(&self, model: &str) -> Option<&PerformanceMetrics> {
        self.metrics.get(model)
    }
    
    pub fn print_summary(&self) {
        println!("\n=== Performance Summary ===");
        for (model, metrics) in &self.metrics {
            println!("Model: {}", model);
            println!("  Requests: {}", metrics.request_count);
            println!("  Avg Response Time: {:?}", metrics.average_response_time);
            println!("  Tokens/sec: {:.2}", metrics.tokens_per_second);
            println!("  Error Rate: {:.2}%", 
                (metrics.error_count as f64 / metrics.request_count as f64) * 100.0);
            println!();
        }
    }
}
```

### System Resource Monitoring

```rust
use sysinfo::{System, SystemExt, ProcessExt};

pub struct ResourceMonitor {
    system: System,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }
    
    pub fn check_system_resources(&mut self) -> SystemResources {
        self.system.refresh_all();
        
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;
        
        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        
        // Find Ollama process
        let ollama_memory = self.system.processes()
            .values()
            .find(|p| p.name().contains("ollama"))
            .map(|p| p.memory())
            .unwrap_or(0);
        
        SystemResources {
            memory_usage_percent,
            cpu_usage_percent: cpu_usage,
            ollama_memory_mb: ollama_memory as f64 / 1024.0 / 1024.0,
            available_memory_mb: (total_memory - used_memory) as f64 / 1024.0 / 1024.0,
        }
    }
    
    pub fn should_scale_down(&mut self) -> bool {
        let resources = self.check_system_resources();
        
        // Scale down if memory usage > 85% or CPU > 90%
        resources.memory_usage_percent > 85.0 || resources.cpu_usage_percent > 90.0
    }
    
    pub fn can_scale_up(&mut self) -> bool {
        let resources = self.check_system_resources();
        
        // Can scale up if memory usage < 60% and CPU < 70%
        resources.memory_usage_percent < 60.0 && resources.cpu_usage_percent < 70.0
    }
}

#[derive(Debug)]
pub struct SystemResources {
    pub memory_usage_percent: f64,
    pub cpu_usage_percent: f32,
    pub ollama_memory_mb: f64,
    pub available_memory_mb: f64,
}
```

## Optimization Strategies by Use Case

### Ultra-Lightweight Testing (qwen2.5:0.5b)

```bash
# Minimal resource configuration
OLLAMA_NUM_CTX=512           # Very small context
OLLAMA_NUM_BATCH=1           # Single request processing
OLLAMA_NUM_THREAD=2          # Minimal threading
OLLAMA_KEEP_ALIVE=1m         # Quick unload
OLLAMA_MAX_LOADED_MODELS=1   # Only one model
```

**Optimization Focus:**
- Minimize memory footprint
- Fast startup times
- Basic functionality validation
- Single-threaded operation

### Production Balanced (qwen2.5:1b)

```bash
# Balanced configuration
OLLAMA_NUM_CTX=2048          # Standard context
OLLAMA_NUM_BATCH=4           # Small batch processing
OLLAMA_NUM_THREAD=4          # Moderate threading
OLLAMA_KEEP_ALIVE=5m         # Standard keep-alive
OLLAMA_MAX_LOADED_MODELS=1   # Single model focus
```

**Optimization Focus:**
- Balance between performance and resources
- Responsive user interaction
- Moderate complexity handling
- Efficient resource utilization

### Advanced Features (qwen2.5:3b)

```bash
# High-performance configuration
OLLAMA_NUM_CTX=4096          # Large context window
OLLAMA_NUM_BATCH=8           # Larger batch sizes
OLLAMA_NUM_THREAD=8          # Full CPU utilization
OLLAMA_KEEP_ALIVE=10m        # Longer keep-alive
OLLAMA_MAX_LOADED_MODELS=2   # Allow model switching
OLLAMA_NUM_GPU=1             # GPU acceleration
```

**Optimization Focus:**
- Maximum performance
- Complex reasoning capabilities
- Large context handling
- GPU acceleration utilization

## Troubleshooting Performance Issues

### Common Performance Problems

1. **Slow Response Times**
   ```bash
   # Check model size and system resources
   ollama show qwen2.5:0.5b
   
   # Reduce context size
   OLLAMA_NUM_CTX=1024
   
   # Increase thread count (if CPU allows)
   OLLAMA_NUM_THREAD=6
   ```

2. **High Memory Usage**
   ```bash
   # Reduce keep-alive time
   OLLAMA_KEEP_ALIVE=30s
   
   # Limit loaded models
   OLLAMA_MAX_LOADED_MODELS=1
   
   # Use smaller batch sizes
   OLLAMA_NUM_BATCH=1
   ```

3. **CPU Bottlenecks**
   ```bash
   # Optimize thread usage
   OLLAMA_NUM_THREAD=4  # Match CPU cores
   
   # Reduce parallel requests
   OLLAMA_NUM_PARALLEL=1
   ```

### Performance Testing Script

```bash
#!/bin/bash
# performance_test.sh

echo "Testing Ollama Performance..."

# Test different models
models=("qwen2.5:0.5b" "qwen2.5:1b")
test_prompt="What is Ohm's law? Explain briefly."

for model in "${models[@]}"; do
    echo "Testing model: $model"
    
    start_time=$(date +%s.%N)
    
    response=$(curl -s -X POST http://localhost:11434/api/generate \
        -H "Content-Type: application/json" \
        -d "{\"model\":\"$model\",\"prompt\":\"$test_prompt\",\"stream\":false}")
    
    end_time=$(date +%s.%N)
    duration=$(echo "$end_time - $start_time" | bc)
    
    echo "Response time: ${duration}s"
    echo "Response length: $(echo "$response" | jq -r '.response' | wc -c) characters"
    echo "---"
done
```

This comprehensive optimization guide ensures that OpenCircuit can efficiently utilize Ollama across different hardware configurations and use cases, starting with ultra-lightweight testing and scaling up as needed.