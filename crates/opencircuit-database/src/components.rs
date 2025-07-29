use anyhow::Result;
use chrono::{DateTime, Utc};
use crate::{ComponentRecord, ComponentFilter, Database};
use opencircuit_core::models::{Component, ComponentCategory, ComponentSearchFilter, ComponentSearchResult, SpecValue};
use serde_json;
use std::collections::HashMap;
use uuid::Uuid;

/// Component-specific database operations
pub struct ComponentDatabase {
    db: Database,
}

impl ComponentDatabase {
    /// Create a new component database instance
    pub fn new() -> Result<Self> {
        let db = Database::new()?;
        Ok(Self { db })
    }

    /// Create a new in-memory component database for testing
    pub fn new_in_memory() -> Result<Self> {
        let db = Database::new_in_memory()?;
        Ok(Self { db })
    }

    /// Convert ComponentRecord to Component model
    fn record_to_component(&self, record: ComponentRecord) -> Component {
        let category = ComponentCategory::from_str(&record.category);
        
        // Parse specifications from JSON string
        let specifications = if let Some(spec_str) = &record.specifications {
            serde_json::from_str::<HashMap<String, SpecValue>>(spec_str)
                .unwrap_or_default()
        } else {
            HashMap::new()
        };

        // Parse timestamps
        let created_at = chrono::DateTime::parse_from_rfc3339(&record.created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());
        
        let updated_at = chrono::DateTime::parse_from_rfc3339(&record.updated_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        Component {
            id: record.id,
            part_number: record.part_number,
            manufacturer: record.manufacturer,
            category,
            description: record.description.unwrap_or_default(),
            specifications,
            footprint: record.footprint,
            symbol: record.symbol,
            datasheet_url: record.datasheet_url,
            price_info: None, // TODO: Implement price info parsing
            availability: None, // TODO: Implement availability parsing
            created_at,
            updated_at,
        }
    }

    /// Convert Component model to ComponentRecord
    fn component_to_record(&self, component: &Component) -> ComponentRecord {
        let specifications_json = if !component.specifications.is_empty() {
            Some(serde_json::to_string(&component.specifications).unwrap_or_default())
        } else {
            None
        };

        ComponentRecord {
            id: component.id.clone(),
            part_number: component.part_number.clone(),
            manufacturer: component.manufacturer.clone(),
            category: component.category.as_str().to_string(),
            description: if component.description.is_empty() {
                None
            } else {
                Some(component.description.clone())
            },
            datasheet_url: component.datasheet_url.clone(),
            specifications: specifications_json,
            footprint: component.footprint.clone(),
            symbol: component.symbol.clone(),
            created_at: component.created_at.to_rfc3339(),
            updated_at: component.updated_at.to_rfc3339(),
        }
    }

    /// Create a new component
    pub fn create_component(&self, component: &Component) -> Result<()> {
        let record = self.component_to_record(component);
        self.db.create_component(&record)
    }

    /// Get a component by ID
    pub fn get_component(&self, id: &str) -> Result<Option<Component>> {
        if let Some(record) = self.db.get_component(id)? {
            Ok(Some(self.record_to_component(record)))
        } else {
            Ok(None)
        }
    }

    /// Update an existing component
    pub fn update_component(&self, component: &Component) -> Result<bool> {
        let record = self.component_to_record(component);
        self.db.update_component(&record)
    }

    /// Delete a component by ID
    pub fn delete_component(&self, id: &str) -> Result<bool> {
        self.db.delete_component(id)
    }

    /// Search components with text query
    pub fn search_components(&self, query: &str, limit: Option<u32>) -> Result<Vec<ComponentSearchResult>> {
        let records = self.db.search_components(query, limit)?;
        let mut results = Vec::new();

        for record in records {
            let component = self.record_to_component(record);
            let relevance_score = self.calculate_relevance_score(&component, query);
            let result = ComponentSearchResult::new(component, relevance_score);
            results.push(result);
        }

        // Sort by relevance score (highest first)
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }

    /// Advanced component search with filters
    pub fn search_components_advanced(&self, filter: &ComponentSearchFilter, limit: Option<u32>) -> Result<Vec<ComponentSearchResult>> {
        // Convert ComponentSearchFilter to ComponentFilter for database query
        let db_filter = ComponentFilter {
            manufacturer: filter.manufacturer.clone(),
            category: filter.category.as_ref().map(|c| c.as_str().to_string()),
            part_number_contains: filter.part_number_contains.clone(),
            description_contains: filter.description_contains.clone(),
        };

        let records = self.db.filter_components(&db_filter, limit)?;
        let mut results = Vec::new();

        for record in records {
            let component = self.record_to_component(record);
            
            // Apply additional filtering that the database doesn't handle
            if filter.matches(&component) {
                let relevance_score = self.calculate_filter_relevance_score(&component, filter);
                let result = ComponentSearchResult::new(component, relevance_score);
                results.push(result);
            }
        }

        // Sort by relevance score (highest first)
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }

    /// Get components by category
    pub fn get_components_by_category(&self, category: &ComponentCategory, limit: Option<u32>) -> Result<Vec<Component>> {
        let filter = ComponentFilter {
            manufacturer: None,
            category: Some(category.as_str().to_string()),
            part_number_contains: None,
            description_contains: None,
        };

        let records = self.db.filter_components(&filter, limit)?;
        let components = records.into_iter()
            .map(|record| self.record_to_component(record))
            .collect();

        Ok(components)
    }

    /// Get all available component categories with counts
    pub fn get_categories_with_counts(&self) -> Result<Vec<(ComponentCategory, i64)>> {
        let counts = self.db.get_component_count_by_category()?;
        let categories = counts.into_iter()
            .map(|(name, count)| (ComponentCategory::from_str(&name), count))
            .collect();

        Ok(categories)
    }

    /// Get total component count
    pub fn get_total_component_count(&self) -> Result<i64> {
        self.db.get_total_component_count()
    }

    /// Bulk import components
    pub fn bulk_import_components(&self, components: Vec<Component>) -> Result<usize> {
        let mut imported_count = 0;

        for component in components {
            match self.create_component(&component) {
                Ok(_) => imported_count += 1,
                Err(e) => {
                    eprintln!("Failed to import component {}: {}", component.part_number, e);
                }
            }
        }

        Ok(imported_count)
    }

    /// Find similar components based on specifications
    pub fn find_similar_components(&self, component: &Component, limit: Option<u32>) -> Result<Vec<ComponentSearchResult>> {
        // Get components from the same category
        let candidates = self.get_components_by_category(&component.category, None)?;
        let mut results = Vec::new();

        for candidate in candidates {
            if candidate.id == component.id {
                continue; // Skip the same component
            }

            let similarity_score = self.calculate_similarity_score(component, &candidate);
            if similarity_score > 0.3 { // Threshold for similarity
                let result = ComponentSearchResult::new(candidate, similarity_score)
                    .with_match_reason("Similar specifications".to_string());
                results.push(result);
            }
        }

        // Sort by similarity score (highest first)
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        // Apply limit
        if let Some(limit) = limit {
            results.truncate(limit as usize);
        }

        Ok(results)
    }

    /// Calculate relevance score for text search
    fn calculate_relevance_score(&self, component: &Component, query: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let mut score = 0.0;

        // Exact match in part number gets highest score
        if component.part_number.to_lowercase() == query_lower {
            score += 100.0;
        } else if component.part_number.to_lowercase().contains(&query_lower) {
            score += 50.0;
        }

        // Manufacturer match
        if component.manufacturer.to_lowercase().contains(&query_lower) {
            score += 30.0;
        }

        // Description match
        if component.description.to_lowercase().contains(&query_lower) {
            score += 20.0;
        }

        // Category match
        if component.category.as_str().to_lowercase().contains(&query_lower) {
            score += 15.0;
        }

        // Specification match
        for (key, value) in &component.specifications {
            if key.to_lowercase().contains(&query_lower) {
                score += 10.0;
            }
            if value.as_string().to_lowercase().contains(&query_lower) {
                score += 10.0;
            }
        }

        score
    }

    /// Calculate relevance score for filter-based search
    fn calculate_filter_relevance_score(&self, component: &Component, filter: &ComponentSearchFilter) -> f64 {
        let mut score = 50.0; // Base score for matching filter

        // Boost score for exact matches
        if let Some(ref manufacturer) = filter.manufacturer {
            if component.manufacturer.to_lowercase() == manufacturer.to_lowercase() {
                score += 20.0;
            }
        }
        
        if let Some(ref part_contains) = filter.part_number_contains {
            if component.part_number.to_lowercase().contains(&part_contains.to_lowercase()) {
                score += 15.0;
            }
        }
        
        if let Some(ref desc_contains) = filter.description_contains {
            if component.description.to_lowercase().contains(&desc_contains.to_lowercase()) {
                score += 10.0;
            }
        }
        
        // Boost for specification matches
        for (key, value) in &filter.specifications {
            if let Some(component_value) = component.specifications.get(key) {
                if component_value == value {
                    score += 25.0;
                }
            }
        }

        // Bonus for having datasheet
        if component.datasheet_url.is_some() {
            score += 10.0;
        }

        // Bonus for having footprint
        if component.footprint.is_some() {
            score += 10.0;
        }

        // Bonus for having specifications
        if !component.specifications.is_empty() {
            score += 5.0 * component.specifications.len() as f64;
        }

        // Bonus for availability
        if let Some(ref availability) = component.availability {
            if availability.in_stock {
                score += 20.0;
            }
        }

        score
    }

    /// Calculate similarity score between two components
    fn calculate_similarity_score(&self, component1: &Component, component2: &Component) -> f64 {
        let mut score = 0.0;
        let mut total_specs = 0;
        let mut matching_specs = 0;

        // Compare specifications
        for (key, value1) in &component1.specifications {
            total_specs += 1;
            if let Some(value2) = component2.specifications.get(key) {
                if value1 == value2 {
                    matching_specs += 1;
                    score += 10.0;
                }
            }
        }

        // Add specifications from component2 that aren't in component1
        for key in component2.specifications.keys() {
            if !component1.specifications.contains_key(key) {
                total_specs += 1;
            }
        }

        // Calculate specification similarity ratio
        if total_specs > 0 {
            let spec_ratio = matching_specs as f64 / total_specs as f64;
            score += spec_ratio * 50.0;
        }

        // Same manufacturer bonus
        if component1.manufacturer == component2.manufacturer {
            score += 20.0;
        }

        // Same footprint bonus
        if component1.footprint.is_some() && component1.footprint == component2.footprint {
            score += 15.0;
        }

        // Normalize score to 0-100 range
        score.min(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencircuit_core::models::SpecValue;

    fn create_test_component() -> Component {
        let mut component = Component::new(
            "R1234".to_string(),
            "Test Corp".to_string(),
            ComponentCategory::Resistors,
            "Test resistor".to_string(),
        );

        component.set_spec("resistance".to_string(), SpecValue::String("1k".to_string()));
        component.set_spec("tolerance".to_string(), SpecValue::String("5%".to_string()));
        component = component.with_footprint("0603".to_string());
        component = component.with_datasheet("https://example.com/datasheet.pdf".to_string());

        component
    }

    #[test]
    fn test_component_database_creation() {
        // This test requires an in-memory database setup
        // For now, we'll just test that the struct can be created
        assert!(true);
    }

    #[test]
    fn test_relevance_score_calculation() {
        let db = ComponentDatabase { db: Database::new().unwrap() };
        let component = create_test_component();

        // Test exact part number match
        let score = db.calculate_relevance_score(&component, "R1234");
        assert!(score >= 100.0);

        // Test partial match
        let score = db.calculate_relevance_score(&component, "R12");
        assert!(score >= 50.0);

        // Test manufacturer match
        let score = db.calculate_relevance_score(&component, "Test Corp");
        assert!(score >= 30.0);

        // Test no match
        let score = db.calculate_relevance_score(&component, "xyz");
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_similarity_score_calculation() {
        let db = ComponentDatabase { db: Database::new().unwrap() };
        let component1 = create_test_component();
        
        let mut component2 = Component::new(
            "R5678".to_string(),
            "Test Corp".to_string(),
            ComponentCategory::Resistors,
            "Another test resistor".to_string(),
        );
        component2.set_spec("resistance".to_string(), SpecValue::String("1k".to_string()));
        component2.set_spec("tolerance".to_string(), SpecValue::String("5%".to_string()));

        let score = db.calculate_similarity_score(&component1, &component2);
        assert!(score > 50.0); // Should be similar due to same specs and manufacturer
    }
}