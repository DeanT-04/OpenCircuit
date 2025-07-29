# OpenCircuit AI Tests

This directory contains integration tests for the OpenCircuit AI module.

## Prerequisites

Before running the tests, ensure you have:

1. **Ollama installed**: Download from [https://ollama.ai/download](https://ollama.ai/download)
2. **Ollama server running**: Start with `ollama serve`
3. **At least one model pulled**: Run `ollama pull qwen2.5:0.5b` (recommended lightweight model)

## Running Tests

### All Integration Tests
```bash
cargo test --test ollama_integration
```

### Specific Test
```bash
cargo test --test ollama_integration test_basic_chat
```

### With Output
```bash
cargo test --test ollama_integration -- --nocapture
```

## Test Coverage

- **test_ollama_server_connection**: Verifies Ollama server connectivity
- **test_model_availability**: Checks available AI models
- **test_basic_chat**: Tests basic conversation functionality
- **test_component_suggestion**: Tests electronic component recommendations
- **test_circuit_analysis**: Tests circuit analysis capabilities
- **test_model_switching**: Tests switching between different AI models
- **test_ai_service_creation**: Tests AI service instantiation

## Notes

- Tests are designed to be non-destructive and will gracefully handle missing dependencies
- If Ollama is not running, tests will log warnings but not fail
- Tests use the actual Ollama API, so they require network connectivity
- Consider running tests in CI with a containerized Ollama instance for reliability