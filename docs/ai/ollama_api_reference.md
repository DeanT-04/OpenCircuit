# Ollama API Reference

## Overview

This document provides a comprehensive reference for the Ollama API endpoints and their usage within OpenCircuit. All examples are tailored for circuit design and analysis use cases.

## Base Configuration

### Environment Variables

```bash
# Core Ollama settings
OLLAMA_HOST=http://localhost:11434
OLLAMA_MODEL=qwen2.5:0.5b

# Performance tuning
OLLAMA_NUM_PARALLEL=1
OLLAMA_MAX_LOADED_MODELS=1
OLLAMA_KEEP_ALIVE=5m
OLLAMA_MAX_QUEUE=512
```

### API Base URL

All API requests are made to: `http://localhost:11434/api/`

## Core Endpoints

### 1. Generate Completion

**Endpoint:** `POST /api/generate`

Generate text completion for circuit design queries.

```json
{
  "model": "qwen2.5:0.5b",
  "prompt": "Analyze this op-amp circuit and explain its function: [circuit description]",
  "stream": false,
  "options": {
    "temperature": 0.7,
    "top_p": 0.9,
    "max_tokens": 500
  }
}
```

**Response:**
```json
{
  "model": "qwen2.5:0.5b",
  "created_at": "2024-01-15T10:30:00Z",
  "response": "This op-amp circuit is configured as a non-inverting amplifier...",
  "done": true,
  "context": [1, 2, 3, ...],
  "total_duration": 1500000000,
  "load_duration": 500000000,
  "prompt_eval_count": 25,
  "prompt_eval_duration": 300000000,
  "eval_count": 150,
  "eval_duration": 700000000
}
```

### 2. Streaming Generation

**Endpoint:** `POST /api/generate` (with `"stream": true`)

Real-time streaming for circuit analysis.

```json
{
  "model": "qwen2.5:0.5b",
  "prompt": "Step-by-step analysis of this power supply circuit: [circuit details]",
  "stream": true,
  "options": {
    "temperature": 0.5
  }
}
```

**Streaming Response:**
```json
{"model":"qwen2.5:0.5b","created_at":"2024-01-15T10:30:00Z","response":"Step","done":false}
{"model":"qwen2.5:0.5b","created_at":"2024-01-15T10:30:01Z","response":" 1:","done":false}
{"model":"qwen2.5:0.5b","created_at":"2024-01-15T10:30:02Z","response":" Input","done":false}
...
{"model":"qwen2.5:0.5b","created_at":"2024-01-15T10:30:15Z","response":"","done":true,"context":[...],"total_duration":15000000000}
```

### 3. Chat Completion

**Endpoint:** `POST /api/chat`

Multi-turn conversation for circuit design assistance.

```json
{
  "model": "qwen2.5:0.5b",
  "messages": [
    {
      "role": "system",
      "content": "You are an expert electronics engineer. Help with circuit design and analysis."
    },
    {
      "role": "user",
      "content": "I need to design a 555 timer circuit for a 1Hz blink rate"
    }
  ],
  "stream": false,
  "options": {
    "temperature": 0.6
  }
}
```

**Response:**
```json
{
  "model": "qwen2.5:0.5b",
  "created_at": "2024-01-15T10:30:00Z",
  "message": {
    "role": "assistant",
    "content": "For a 1Hz blink rate with a 555 timer in astable mode, you'll need..."
  },
  "done": true,
  "total_duration": 2000000000,
  "load_duration": 100000000,
  "prompt_eval_count": 45,
  "prompt_eval_duration": 400000000,
  "eval_count": 200,
  "eval_duration": 1500000000
}
```

### 4. Embeddings

**Endpoint:** `POST /api/embeddings`

Generate embeddings for circuit component search and similarity.

```json
{
  "model": "qwen2.5:0.5b",
  "prompt": "LM317 voltage regulator adjustable output 1.25V to 37V"
}
```

**Response:**
```json
{
  "embedding": [0.123, -0.456, 0.789, ...]
}
```

## Model Management

### 5. List Local Models

**Endpoint:** `GET /api/tags`

List all locally available models.

**Response:**
```json
{
  "models": [
    {
      "name": "qwen2.5:0.5b",
      "model": "qwen2.5:0.5b",
      "modified_at": "2024-01-15T10:00:00Z",
      "size": 394000000,
      "digest": "sha256:abc123...",
      "details": {
        "parent_model": "",
        "format": "gguf",
        "family": "qwen2",
        "families": ["qwen2"],
        "parameter_size": "0.5B",
        "quantization_level": "Q4_0"
      }
    },
    {
      "name": "qwen2.5:1b",
      "model": "qwen2.5:1b",
      "modified_at": "2024-01-15T09:30:00Z",
      "size": 712000000,
      "digest": "sha256:def456...",
      "details": {
        "parameter_size": "1B",
        "quantization_level": "Q4_0"
      }
    }
  ]
}
```

### 6. Show Model Information

**Endpoint:** `POST /api/show`

Get detailed information about a specific model.

```json
{
  "name": "qwen2.5:0.5b"
}
```

**Response:**
```json
{
  "modelfile": "FROM qwen2.5:0.5b\nPARAMETER temperature 0.7\n...",
  "parameters": "temperature 0.7\ntop_p 0.9\n...",
  "template": "{{ if .System }}System: {{ .System }}\n{{ end }}User: {{ .Prompt }}\nAssistant:",
  "details": {
    "parent_model": "",
    "format": "gguf",
    "family": "qwen2",
    "families": ["qwen2"],
    "parameter_size": "0.5B",
    "quantization_level": "Q4_0"
  },
  "model_info": {
    "general.architecture": "qwen2",
    "general.file_type": 2,
    "general.parameter_count": 494033920,
    "general.quantization_version": 2,
    "qwen2.attention.head_count": 14,
    "qwen2.attention.head_count_kv": 2,
    "qwen2.attention.layer_norm_rms_epsilon": 1e-06,
    "qwen2.block_count": 24,
    "qwen2.context_length": 32768,
    "qwen2.embedding_length": 896,
    "qwen2.feed_forward_length": 4864,
    "qwen2.rope.freq_base": 1000000,
    "tokenizer.ggml.bos_token_id": 151643,
    "tokenizer.ggml.eos_token_id": 151645,
    "tokenizer.ggml.model": "gpt2",
    "tokenizer.ggml.tokens": ["!", "\"", "#", ...]
  }
}
```

### 7. Copy Model

**Endpoint:** `POST /api/copy`

Create a copy of an existing model with a new name.

```json
{
  "source": "qwen2.5:0.5b",
  "destination": "circuit-assistant:0.5b"
}
```

### 8. Delete Model

**Endpoint:** `DELETE /api/delete`

Remove a model from local storage.

```json
{
  "name": "qwen2.5:0.5b"
}
```

### 9. Pull Model

**Endpoint:** `POST /api/pull`

Download a model from the Ollama registry.

```json
{
  "name": "qwen2.5:1b",
  "stream": true
}
```

**Streaming Response:**
```json
{"status":"pulling manifest"}
{"status":"downloading","digest":"sha256:abc123...","total":712000000,"completed":0}
{"status":"downloading","digest":"sha256:abc123...","total":712000000,"completed":71200000}
...
{"status":"verifying sha256 digest"}
{"status":"writing manifest"}
{"status":"removing any unused layers"}
{"status":"success"}
```

### 10. Push Model

**Endpoint:** `POST /api/push`

Upload a custom model to a registry.

```json
{
  "name": "custom/circuit-assistant:latest",
  "stream": true
}
```

### 11. Create Model

**Endpoint:** `POST /api/create`

Create a custom model from a Modelfile.

```json
{
  "name": "circuit-assistant:custom",
  "modelfile": "FROM qwen2.5:0.5b\nSYSTEM You are an expert electronics engineer specializing in circuit design and analysis.\nPARAMETER temperature 0.6\nPARAMETER top_p 0.9",
  "stream": true
}
```

## Circuit-Specific API Usage Examples

### Component Recommendation

```bash
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen2.5:0.5b",
    "prompt": "Recommend a microcontroller for a battery-powered IoT sensor that needs WiFi connectivity, low power consumption, and analog inputs for temperature and humidity sensors.",
    "options": {
      "temperature": 0.5,
      "max_tokens": 300
    }
  }'
```

### Circuit Analysis

```bash
curl -X POST http://localhost:11434/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen2.5:0.5b",
    "messages": [
      {
        "role": "system",
        "content": "You are an expert circuit analyst. Provide detailed technical analysis."
      },
      {
        "role": "user",
        "content": "Analyze this amplifier circuit: Op-amp LM358, R1=10kΩ (feedback), R2=1kΩ (input), Vcc=+12V, Vee=-12V. What is the gain and frequency response?"
      }
    ],
    "stream": false
  }'
```

### Streaming Circuit Design

```bash
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen2.5:0.5b",
    "prompt": "Design a complete power supply circuit that converts 120V AC to 5V DC at 2A output. Include transformer, rectifier, filter, and regulation stages with component values.",
    "stream": true,
    "options": {
      "temperature": 0.4
    }
  }'
```

## Error Handling

### Common Error Responses

**Model Not Found:**
```json
{
  "error": "model 'qwen2.5:2b' not found, try pulling it first"
}
```

**Invalid Request:**
```json
{
  "error": "invalid request: missing required field 'model'"
}
```

**Server Error:**
```json
{
  "error": "internal server error"
}
```

### HTTP Status Codes

- `200 OK` - Request successful
- `400 Bad Request` - Invalid request format
- `404 Not Found` - Model or endpoint not found
- `500 Internal Server Error` - Server error

## Performance Optimization

### Request Options

```json
{
  "options": {
    "temperature": 0.7,        // Creativity (0.0-2.0)
    "top_p": 0.9,             // Nucleus sampling (0.0-1.0)
    "top_k": 40,              // Top-k sampling
    "repeat_penalty": 1.1,     // Repetition penalty
    "seed": 42,               // Reproducible results
    "num_predict": 500,       // Max tokens to generate
    "num_ctx": 2048,          // Context window size
    "num_batch": 8,           // Batch size for processing
    "num_gpu": 1,             // Number of GPU layers
    "main_gpu": 0,            // Primary GPU index
    "low_vram": false,        // Low VRAM mode
    "f16_kv": true,           // Use 16-bit for key/value cache
    "logits_all": false,      // Return logits for all tokens
    "vocab_only": false,      // Only load vocabulary
    "use_mmap": true,         // Use memory mapping
    "use_mlock": false,       // Lock memory pages
    "num_thread": 8           // Number of threads
  }
}
```

### Batch Processing

For multiple circuit queries, use batch processing:

```bash
# Process multiple component queries
for query in "resistor for LED" "capacitor for filter" "inductor for boost converter"; do
  curl -X POST http://localhost:11434/api/generate \
    -H "Content-Type: application/json" \
    -d "{\"model\":\"qwen2.5:0.5b\",\"prompt\":\"$query\"}" &
done
wait
```

## Integration Patterns

### Health Check

```bash
curl -f http://localhost:11434/api/tags > /dev/null 2>&1
if [ $? -eq 0 ]; then
  echo "Ollama server is running"
else
  echo "Ollama server is not accessible"
fi
```

### Model Availability Check

```bash
MODEL_EXISTS=$(curl -s http://localhost:11434/api/tags | jq -r '.models[] | select(.name=="qwen2.5:0.5b") | .name')
if [ "$MODEL_EXISTS" = "qwen2.5:0.5b" ]; then
  echo "Model is available"
else
  echo "Model needs to be pulled"
  curl -X POST http://localhost:11434/api/pull -d '{"name":"qwen2.5:0.5b"}'
fi
```

This API reference provides the foundation for integrating Ollama's REST API with OpenCircuit's circuit design and analysis features.