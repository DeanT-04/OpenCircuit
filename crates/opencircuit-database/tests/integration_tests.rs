use anyhow::Result;
use opencircuit_core::models::{Component, ComponentCategory, ComponentSearchFilter, SpecValue, PriceInfo, PriceBreak, AvailabilityInfo};
use opencircuit_database::{ComponentDatabase, ComponentSearchEngine};
use std::collections::HashMap;
use uuid::Uuid;
use chrono;

/// Integration tests that simulate real user scenarios

#[test]
fn test_complete_user_workflow() -> Result<()> {
    // Scenario: User wants to find and manage resistors for a project
    let db = ComponentDatabase::new()?;
    
    // 1. User adds some sample components
    let resistor_1k = create_sample_resistor("R1001", "1kΩ", "Texas Instruments")?;
    let resistor_10k = create_sample_resistor("R1002", "10kΩ", "Vishay")?;
    let capacitor_100nf = create_sample_capacitor("C2001", "100nF", "Murata")?;
    
    // Store the IDs before creating components
    let _id1 = resistor_1k.id.clone();
    let _id2 = resistor_10k.id.clone();
    let _id3 = capacitor_100nf.id.clone();
    
    db.create_component(&resistor_1k)?;
    db.create_component(&resistor_10k)?;
    db.create_component(&capacitor_100nf)?;
    
    // 2. User searches for resistors
    let resistor_results = db.get_components_by_category(&ComponentCategory::Resistors, Some(10))?;
    assert!(resistor_results.len() >= 2);
    
    // 3. User searches for specific resistance value
    let search_results = db.search_components("R1001", Some(5))?;
    assert!(!search_results.is_empty());
    assert!(search_results.iter().any(|r| r.component.part_number == "R1001"));
    
    // 4. User updates a component
    let mut updated_resistor = resistor_1k.clone();
    updated_resistor.description = "Updated 1kΩ precision resistor".to_string();
    let update_result = db.update_component(&updated_resistor)?;
    assert!(update_result, "Should successfully update component");
    
    let retrieved = db.get_component(&resistor_1k.id)?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().description, "Updated 1kΩ precision resistor");
    
    // Delete a component
    let delete_result = db.delete_component(&resistor_1k.id)?;
    assert!(delete_result, "Should successfully delete component");
    
    // Verify deletion
    let deleted = db.get_component(&resistor_1k.id)?;
    assert!(deleted.is_none(), "Component should be deleted");
    
    println!("✅ Complete user workflow test passed");
    Ok(())
}

#[test]
fn test_advanced_search_scenarios() -> Result<()> {
    let engine = ComponentSearchEngine::new()?;
    let db = ComponentDatabase::new()?;
    
    // Add test components
    let components = vec![
        create_sample_resistor("R1001", "1kΩ", "Texas Instruments")?,
        create_sample_resistor("R1002", "4.7kΩ", "Vishay")?,
        create_sample_resistor("R1003", "100Ω", "Panasonic")?,
        create_sample_capacitor("C2001", "100nF", "Murata")?,
        create_sample_capacitor("C2002", "10µF", "Samsung")?,
    ];
    
    for component in &components {
        db.create_component(component)?;
    }
    
    // Test natural language queries
    let test_queries = vec![
        ("1k resistor", "Should find 1kΩ resistor"),
        ("4.7kohm", "Should find 4.7kΩ resistor"),
        ("100nf capacitor", "Should find 100nF capacitor"),
        ("texas instruments", "Should find TI components"),
        ("murata", "Should find Murata components"),
    ];
    
    for (query, description) in test_queries {
        let results = engine.search(query, Some(10))?;
        println!("Query: '{}' - {} - Found {} results", query, description, results.len());
        
        // Verify we get relevant results
        if !results.is_empty() {
            println!("  Top result: {} (score: {:.1})", 
                results[0].component.part_number, 
                results[0].relevance_score);
        }
    }
    
    // Test category-specific searches
    let resistor_results = engine.search_by_category(&ComponentCategory::Resistors, None, Some(10))?;
    assert!(resistor_results.len() >= 3);
    
    let capacitor_results = engine.search_by_category(&ComponentCategory::Capacitors, None, Some(10))?;
    assert!(capacitor_results.len() >= 2);
    
    // Test manufacturer search
    let ti_results = engine.search_by_manufacturer("Texas Instruments", Some(10))?;
    assert!(!ti_results.is_empty());
    
    println!("✅ Advanced search scenarios test passed");
    Ok(())
}

#[test]
fn test_specification_filtering() -> Result<()> {
    let db = ComponentDatabase::new_in_memory()?;
    
    // Create components with specific specifications
    let mut resistor_smd = create_sample_resistor("R0603-1K", "1kΩ", "Yageo")?;
    resistor_smd.specifications.insert("package".to_string(), SpecValue::String("0603".to_string()));
    resistor_smd.specifications.insert("tolerance".to_string(), SpecValue::String("1%".to_string()));
    
    let mut resistor_through_hole = create_sample_resistor("R-TH-1K", "1kΩ", "Vishay")?;
    resistor_through_hole.specifications.insert("package".to_string(), SpecValue::String("TH".to_string()));
    resistor_through_hole.specifications.insert("tolerance".to_string(), SpecValue::String("5%".to_string()));
    
    // Create components in database
    db.create_component(&resistor_smd)?;
    db.create_component(&resistor_through_hole)?;
    
    // Test filtering by package specification
    let package_filter = ComponentSearchFilter::new()
        .with_category(ComponentCategory::Resistors)
        .with_specification("package".to_string(), SpecValue::String("0603".to_string()));
    
    let package_results = db.search_components_advanced(&package_filter, Some(10))?;
    assert_eq!(package_results.len(), 1, "Should find exactly 1 component with 0603 package");
    assert_eq!(package_results[0].component.part_number, "R0603-1K");
    
    // Test filtering by tolerance specification
    let tolerance_filter = ComponentSearchFilter::new()
        .with_category(ComponentCategory::Resistors)
        .with_specification("tolerance".to_string(), SpecValue::String("5%".to_string()));
    
    let tolerance_results = db.search_components_advanced(&tolerance_filter, Some(10))?;
    assert_eq!(tolerance_results.len(), 1, "Should find exactly 1 component with 5% tolerance");
    assert_eq!(tolerance_results[0].component.part_number, "R-TH-1K");
    
    // Test filtering by multiple specifications
    let multi_filter = ComponentSearchFilter::new()
        .with_category(ComponentCategory::Resistors)
        .with_specification("package".to_string(), SpecValue::String("0603".to_string()))
        .with_specification("tolerance".to_string(), SpecValue::String("1%".to_string()));
    
    let multi_results = db.search_components_advanced(&multi_filter, Some(10))?;
    assert_eq!(multi_results.len(), 1, "Should find exactly 1 component with both 0603 package and 1% tolerance");
    assert_eq!(multi_results[0].component.part_number, "R0603-1K");
    
    // Test filtering with no matches
    let no_match_filter = ComponentSearchFilter::new()
        .with_category(ComponentCategory::Resistors)
        .with_specification("package".to_string(), SpecValue::String("0805".to_string()));
    
    let no_match_results = db.search_components_advanced(&no_match_filter, Some(10))?;
    assert_eq!(no_match_results.len(), 0, "Should find no components with 0805 package");
    
    println!("✅ Specification filtering test passed");
    Ok(())
}

#[test]
fn test_bulk_operations() -> Result<()> {
    let db = ComponentDatabase::new_in_memory()?;
    let engine = ComponentSearchEngine::new()?;
    
    // Create a batch of components
    let mut components = Vec::new();
    for i in 1..=50 {
        let mut resistor = create_sample_resistor(
            &format!("R{:04}", i), 
            &format!("{}kΩ", i), 
            "Bulk Manufacturer"
        )?;
        resistor.specifications.insert("series".to_string(), SpecValue::String("E24".to_string()));
        components.push(resistor);
    }
    
    // Bulk import
    let imported_count = db.bulk_import_components(components)?;
    assert_eq!(imported_count, 50);
    
    // Test bulk search
    let bulk_results = engine.search("Bulk Manufacturer", Some(100))?;
    assert!(bulk_results.len() >= 50);
    
    println!("✅ Bulk operations test passed");
    Ok(())
}

#[test]
fn test_search_suggestions() -> Result<()> {
    let engine = ComponentSearchEngine::new()?;
    let db = ComponentDatabase::new_in_memory()?;
    
    // Add components for suggestion testing
    let components = vec![
        create_sample_resistor("LM358", "Op-Amp", "Texas Instruments")?,
        create_sample_resistor("LM324", "Quad Op-Amp", "Texas Instruments")?,
        create_sample_resistor("LM741", "Op-Amp", "Texas Instruments")?,
    ];
    
    for component in &components {
        db.create_component(component)?;
    }
    
    // Test partial queries
    let suggestions = engine.get_search_suggestions("LM", Some(5))?;
    println!("Suggestions for 'LM': {:?}", suggestions);
    assert!(!suggestions.is_empty());
    
    let ti_suggestions = engine.get_search_suggestions("Texas", Some(5))?;
    println!("Suggestions for 'Texas': {:?}", ti_suggestions);
    assert!(!ti_suggestions.is_empty());
    
    println!("✅ Search suggestions test passed");
    Ok(())
}

#[test]
fn test_fuzzy_search_real_scenarios() -> Result<()> {
    let engine = ComponentSearchEngine::new()?;
    let db = ComponentDatabase::new()?;
    
    // Add components with realistic part numbers
    let components = vec![
        create_sample_resistor("RC0603FR-071KL", "1kΩ", "Yageo")?,
        create_sample_resistor("RC0603FR-074K7L", "4.7kΩ", "Yageo")?,
        create_sample_resistor("CRCW060310K0FKEA", "10kΩ", "Vishay")?,
    ];
    
    for component in &components {
        db.create_component(component)?;
    }
    
    // Test fuzzy searches with typos
    let typo_queries = vec![
        ("RC0603FR-071KL", "RC0603FR071KL"), // Missing dash
        ("CRCW060310K0FKEA", "CRCW060310K0FKEA"), // Exact match
        ("RC0603", "RC0603"), // Partial match
    ];
    
    for (_, query) in typo_queries {
        let results = engine.search(query, Some(5))?;
        println!("Fuzzy search for '{}': {} results", query, results.len());
        
        if !results.is_empty() {
            println!("  Best match: {} (score: {:.1})", 
                results[0].component.part_number, 
                results[0].relevance_score);
        }
    }
    
    println!("✅ Fuzzy search real scenarios test passed");
    Ok(())
}

#[test]
fn test_performance_with_large_dataset() -> Result<()> {
    let db = ComponentDatabase::new()?;
    let engine = ComponentSearchEngine::new()?;
    
    // Create a larger dataset
    let mut components = Vec::new();
    let manufacturers = vec!["TI", "Vishay", "Murata", "Samsung", "Panasonic"];
    let categories = vec![ComponentCategory::Resistors, ComponentCategory::Capacitors, ComponentCategory::Inductors];
    
    for i in 1..=200 {
        let manufacturer = &manufacturers[i % manufacturers.len()];
        let category = &categories[i % categories.len()];
        
        let component = match category {
            ComponentCategory::Resistors => create_sample_resistor(
                &format!("R{:05}", i), 
                &format!("{}kΩ", i % 100 + 1), 
                manufacturer
            )?,
            ComponentCategory::Capacitors => create_sample_capacitor(
                &format!("C{:05}", i), 
                &format!("{}nF", (i % 100 + 1) * 10), 
                manufacturer
            )?,
            _ => create_sample_resistor(&format!("L{:05}", i), "1µH", manufacturer)?,
        };
        
        components.push(component);
    }
    
    // Bulk import
    let start = std::time::Instant::now();
    let _count = db.bulk_import_components(components)?;
    let import_time = start.elapsed();
    println!("Bulk import of 200 components took: {:?}", import_time);
    
    // Test search performance
    let search_start = std::time::Instant::now();
    let search_results = engine.search("resistor", Some(100))?;
    let search_time = search_start.elapsed();
    println!("Search found {} results in {:?}", search_results.len(), search_time);
    assert!(search_time.as_millis() < 1000, "Search should complete within 1 second");
    
    // Test category filtering performance
    let filter_start = std::time::Instant::now();
    let category_results = engine.search_by_category(&ComponentCategory::Resistors, None, Some(100))?;
    let category_time = filter_start.elapsed();
    println!("Category search found {} results in {:?}", category_results.len(), category_time);
    assert!(category_time.as_millis() < 500, "Category search should complete within 500ms");
    
    // Verify reasonable performance (should be under 2 seconds for these operations)
    assert!(import_time.as_millis() < 2000, "Import took too long: {:?}", import_time);
    assert!(search_time.as_millis() < 200, "Search took too long: {:?}", search_time);
    assert!(category_time.as_millis() < 100, "Filtering took too long: {:?}", category_time);
    
    println!("✅ Performance test passed");
    Ok(())
}

// Helper functions
fn create_sample_resistor(part_number: &str, resistance: &str, manufacturer: &str) -> Result<Component> {
    let mut specifications = HashMap::new();
    specifications.insert("resistance".to_string(), SpecValue::String(resistance.to_string()));
    specifications.insert("tolerance".to_string(), SpecValue::String("5%".to_string()));
    specifications.insert("power".to_string(), SpecValue::String("0.25W".to_string()));
    
    let price_breaks = vec![
        PriceBreak { quantity: 1, unit_price: 0.10 },
        PriceBreak { quantity: 100, unit_price: 0.05 },
        PriceBreak { quantity: 1000, unit_price: 0.02 },
    ];
    
    Ok(Component::new(
        part_number.to_string(),
        manufacturer.to_string(),
        ComponentCategory::Resistors,
        format!("{} resistor from {}", resistance, manufacturer),
    )
    .with_specifications(specifications)
    .with_datasheet("https://example.com/datasheet.pdf".to_string())
    .with_footprint("0603".to_string())
    .with_price_info(PriceInfo {
        currency: "USD".to_string(),
        price_breaks,
        last_updated: chrono::Utc::now(),
        supplier: "DigiKey".to_string(),
    })
    .with_availability(AvailabilityInfo {
        in_stock: true,
        quantity_available: Some(1000),
        lead_time_days: Some(1),
        minimum_order_quantity: Some(1),
        last_updated: chrono::Utc::now(),
        supplier: "DigiKey".to_string(),
    }))
}

fn create_sample_capacitor(part_number: &str, capacitance: &str, manufacturer: &str) -> Result<Component> {
    let mut specifications = HashMap::new();
    specifications.insert("capacitance".to_string(), SpecValue::String(capacitance.to_string()));
    specifications.insert("voltage".to_string(), SpecValue::String("50V".to_string()));
    specifications.insert("tolerance".to_string(), SpecValue::String("10%".to_string()));
    
    let price_breaks = vec![
        PriceBreak { quantity: 1, unit_price: 0.15 },
        PriceBreak { quantity: 100, unit_price: 0.08 },
        PriceBreak { quantity: 1000, unit_price: 0.03 },
    ];
    
    Ok(Component::new(
        part_number.to_string(),
        manufacturer.to_string(),
        ComponentCategory::Capacitors,
        format!("{} ceramic capacitor from {}", capacitance, manufacturer),
    )
    .with_specifications(specifications)
    .with_datasheet("https://example.com/cap-datasheet.pdf".to_string())
    .with_footprint("0805".to_string())
    .with_price_info(PriceInfo {
        currency: "USD".to_string(),
        price_breaks,
        last_updated: chrono::Utc::now(),
        supplier: "Mouser".to_string(),
    })
    .with_availability(AvailabilityInfo {
        in_stock: true,
        quantity_available: Some(5000),
        lead_time_days: Some(2),
        minimum_order_quantity: Some(1),
        last_updated: chrono::Utc::now(),
        supplier: "Mouser".to_string(),
    }))
}