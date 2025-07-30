# 🤖 Human-Like Integration Tests

This directory contains comprehensive tests that simulate real user interactions with OpenCircuit, designed to test the system as if actual humans were using it.

## 🎯 Test Overview

The tests are organized into realistic user scenarios covering different expertise levels and project types:

### Test Categories

1. **Beginner Tests** - First-time users with simple projects
2. **Professional Tests** - Experienced engineers with complex requirements  
3. **Student Tests** - Educational scenarios with learning focus
4. **IoT/Maker Tests** - Modern hobbyist projects
5. **Edge Cases** - Error handling and boundary conditions
6. **Workflow Tests** - Complete project lifecycles

## 🚀 Running the Tests

### Option 1: Run All Tests
```bash
cargo test --test user_simulation_tests
```

### Option 2: Run Specific Test Scenarios
```bash
# Run beginner scenarios
cargo test --test user_simulation_tests test_beginner

# Run professional scenarios  
cargo test --test user_simulation_tests test_professional

# Run IoT scenarios
cargo test --test user_simulation_tests test_iot
```

### Option 3: Interactive Test Runner
```bash
cargo run --bin test_runner
```

## 📋 Test Scenarios

### Scenario 1: Sarah's First LED Blinker
- **User Type**: Complete beginner
- **Budget**: $20
- **Goal**: Simple LED blinking circuit
- **Expected Journey**: Basic component discovery, simple wiring, first success

### Scenario 2: David's Industrial Sensor
- **User Type**: Professional engineer
- **Budget**: $200
- **Goal**: 4-20mA sensor interface
- **Expected Journey**: Precision components, detailed specifications, compliance checking

### Scenario 3: Alex's Learning Amplifier
- **User Type**: Electronics student
- **Budget**: $15
- **Goal**: Understanding transistor amplifiers
- **Expected Journey**: Educational explanations, theory + practice, guided learning

### Scenario 4: Maya's IoT Weather Station
- **User Type**: IoT developer/hobbyist
- **Budget**: $50
- **Goal**: ESP32-based weather monitoring
- **Expected Journey**: Wireless connectivity, power optimization, cloud integration

## 🔧 Test Architecture

### User Simulation Framework

```rust
// Each test follows this pattern:
let user = TestUser::new("Name", UserType);
user.describe_project();
user.set_requirements([...]);
user.set_budget(min, max);

// AI interaction simulation
let response = ai_assistant.chat(user.query);
let components = component_recommender.recommend(user.requirements);

// Validation
assert!(components.fits_budget());
assert!(components.meets_requirements());
```

### Mock Data Integration

Tests use realistic mock data including:
- Real component names and prices
- Actual distributor stock levels
- Typical lead times
- Common specifications

## 📊 Test Validation

### What We Test

1. **User Experience Flow**
   - Natural language processing
   - Context-aware recommendations
   - Progressive disclosure of complexity

2. **Component Selection**
   - Budget adherence
   - Technical suitability
   - Availability checking

3. **Error Handling**
   - Impossible requirements
   - Budget constraints
   - Missing components

4. **Educational Value**
   - Explanations for beginners
   - Advanced options for experts
   - Learning resources

### Success Criteria

- ✅ All user types can complete their projects
- ✅ Budget constraints are respected
- ✅ Technical requirements are met
- ✅ Educational content is appropriate for user level
- ✅ Error cases are handled gracefully
- ✅ Real-world component availability is considered

## 🎮 Interactive Testing

For manual testing and exploration, use the interactive test runner:

```bash
cargo run --bin interactive_test
```

This opens a simple CLI interface where you can:
- Select user personas
- Enter project descriptions
- See AI recommendations in real-time
- Test edge cases interactively

## 📈 Test Coverage

| User Type | Scenarios | Components | Budget Ranges |
|-----------|-----------|------------|---------------|
| Beginner | 5 | 15+ | $5-$25 |
| Hobbyist | 8 | 25+ | $15-$75 |
| Professional | 6 | 30+ | $50-$500 |
| Student | 4 | 20+ | $10-$50 |

## 🔍 Debugging Tests

### Enable Detailed Logging
```bash
RUST_LOG=debug cargo test --test user_simulation_tests
```

### Run Single Test with Output
```bash
cargo test --test user_simulation_tests test_beginner_led_blinker -- --nocapture
```

### Generate Test Report
```bash
cargo test --test user_simulation_tests -- --format json > test_report.json
```

## 🚀 Adding New Test Scenarios

To add a new test scenario:

1. Create a new test function following the existing patterns
2. Define user persona and requirements
3. Add mock components if needed
4. Include validation assertions
5. Update this README with the new scenario

### Template for New Test

```rust
#[test]
fn test_new_scenario() {
    let user = TestUser::new("Name", UserType::NewType);
    
    // Define requirements
    let requirements = vec![...];
    
    // Test logic
    let components = simulate_recommendation(requirements);
    
    // Validation
    assert!(validate_components(components));
}
```

## 📞 Support

For questions about these tests or to suggest new scenarios:
- Open an issue with the `testing` label
- Tag with `user-experience` for UX-related tests
- Include specific user persona details

Happy testing! 🎉