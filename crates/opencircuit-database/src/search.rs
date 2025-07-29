use anyhow::Result;
use opencircuit_core::models::{Component, ComponentCategory, ComponentSearchFilter, ComponentSearchResult, SpecValue};
use std::collections::HashMap;
use crate::components::ComponentDatabase;

/// Advanced search engine for components
pub struct ComponentSearchEngine {
    db: ComponentDatabase,
}

impl ComponentSearchEngine {
    /// Create a new search engine instance
    pub fn new() -> Result<Self> {
        let db = ComponentDatabase::new()?;
        Ok(Self { db })
    }

    /// Perform a comprehensive search with multiple strategies
    pub fn search(&self, query: &str, limit: Option<u32>) -> Result<Vec<ComponentSearchResult>> {
        let mut all_results = Vec::new();

        // Strategy 1: Direct text search
        let text_results = self.db.search_components(query, limit)?;
        all_results.extend(text_results);

        // Strategy 2: Parse query for specific searches
        if let Some(parsed_filter) = self.parse_query_to_filter(query) {
            let filter_results = self.db.search_components_advanced(&parsed_filter, limit)?;
            all_results.extend(filter_results);
        }

        // Strategy 3: Fuzzy matching for part numbers
        let fuzzy_results = self.fuzzy_search_part_numbers(query, limit)?;
        all_results.extend(fuzzy_results);

        // Deduplicate and merge results
        let merged_results = self.merge_and_deduplicate_results(all_results);

        // Apply final limit
        let final_results = if let Some(limit) = limit {
            merged_results.into_iter().take(limit as usize).collect()
        } else {
            merged_results
        };

        Ok(final_results)
    }

    /// Search by category with optional filters
    pub fn search_by_category(
        &self,
        category: &ComponentCategory,
        additional_filter: Option<ComponentSearchFilter>,
        limit: Option<u32>,
    ) -> Result<Vec<ComponentSearchResult>> {
        let mut filter = ComponentSearchFilter::new().with_category(category.clone());

        // Merge additional filters if provided
        if let Some(additional) = additional_filter {
            if let Some(manufacturer) = additional.manufacturer {
                filter = filter.with_manufacturer(manufacturer);
            }
            if let Some(part_number) = additional.part_number_contains {
                filter = filter.with_part_number_contains(part_number);
            }
            if let Some(description) = additional.description_contains {
                filter = filter.with_description_contains(description);
            }
            for (key, value) in additional.specifications {
                filter = filter.with_specification(key, value);
            }
            if additional.has_datasheet == Some(true) {
                filter = filter.with_datasheet_required();
            }
            if additional.has_footprint == Some(true) {
                filter = filter.with_footprint_required();
            }
            if additional.in_stock_only == Some(true) {
                filter = filter.in_stock_only();
            }
        }

        self.db.search_components_advanced(&filter, limit)
    }

    /// Search for components with specific specifications
    pub fn search_by_specifications(
        &self,
        specifications: HashMap<String, SpecValue>,
        limit: Option<u32>,
    ) -> Result<Vec<ComponentSearchResult>> {
        let mut filter = ComponentSearchFilter::new();
        for (key, value) in specifications {
            filter = filter.with_specification(key, value);
        }

        self.db.search_components_advanced(&filter, limit)
    }

    /// Find components by manufacturer
    pub fn search_by_manufacturer(
        &self,
        manufacturer: &str,
        limit: Option<u32>,
    ) -> Result<Vec<ComponentSearchResult>> {
        let filter = ComponentSearchFilter::new()
            .with_manufacturer(manufacturer.to_string());

        self.db.search_components_advanced(&filter, limit)
    }

    /// Search for components with datasheet available
    pub fn search_with_datasheet(&self, limit: Option<u32>) -> Result<Vec<ComponentSearchResult>> {
        let filter = ComponentSearchFilter::new()
            .with_datasheet_required();

        self.db.search_components_advanced(&filter, limit)
    }

    /// Search for in-stock components only
    pub fn search_in_stock(&self, limit: Option<u32>) -> Result<Vec<ComponentSearchResult>> {
        let filter = ComponentSearchFilter::new()
            .in_stock_only();

        self.db.search_components_advanced(&filter, limit)
    }

    /// Get search suggestions based on partial input
    pub fn get_search_suggestions(&self, partial_query: &str, limit: Option<u32>) -> Result<Vec<String>> {
        let limit = limit.unwrap_or(10);
        let mut suggestions = Vec::new();

        // Get components that match the partial query
        let results = self.db.search_components(partial_query, Some(limit * 2))?;

        for result in results.iter().take(limit as usize) {
            let component = &result.component;
            
            // Add part number suggestions
            if component.part_number.to_lowercase().contains(&partial_query.to_lowercase()) {
                suggestions.push(component.part_number.clone());
            }

            // Add manufacturer suggestions
            if component.manufacturer.to_lowercase().contains(&partial_query.to_lowercase()) {
                suggestions.push(component.manufacturer.clone());
            }

            // Add category suggestions
            if component.category.as_str().to_lowercase().contains(&partial_query.to_lowercase()) {
                suggestions.push(component.category.as_str().to_string());
            }
        }

        // Remove duplicates and sort
        suggestions.sort();
        suggestions.dedup();
        suggestions.truncate(limit as usize);

        Ok(suggestions)
    }

    /// Parse natural language query into structured filter
    fn parse_query_to_filter(&self, query: &str) -> Option<ComponentSearchFilter> {
        let query_lower = query.to_lowercase();
        let mut filter = ComponentSearchFilter::new();
        let mut has_criteria = false;

        // Parse resistance values
        if let Some(resistance) = self.extract_resistance(&query_lower) {
            filter = filter.with_specification("resistance".to_string(), resistance);
            has_criteria = true;
        }

        // Parse capacitance values
        if let Some(capacitance) = self.extract_capacitance(&query_lower) {
            filter = filter.with_specification("capacitance".to_string(), capacitance);
            has_criteria = true;
        }

        // Parse voltage values
        if let Some(voltage) = self.extract_voltage(&query_lower) {
            filter = filter.with_specification("voltage".to_string(), voltage);
            has_criteria = true;
        }

        // Parse package/footprint
        if let Some(package) = self.extract_package(&query_lower) {
            filter = filter.with_specification("package".to_string(), SpecValue::String(package));
            has_criteria = true;
        }

        // Parse manufacturer names
        if let Some(manufacturer) = self.extract_manufacturer(&query_lower) {
            filter = filter.with_manufacturer(manufacturer);
            has_criteria = true;
        }

        if has_criteria {
            Some(filter)
        } else {
            None
        }
    }

    /// Extract resistance value from query
    fn extract_resistance(&self, query: &str) -> Option<SpecValue> {
        // Look for patterns like "1k", "10ohm", "4.7k", "100R"
        let resistance_patterns = [
            r"(\d+(?:\.\d+)?)\s*k(?:ohm)?",
            r"(\d+(?:\.\d+)?)\s*m(?:ohm)?",
            r"(\d+(?:\.\d+)?)\s*(?:ohm|r)",
        ];

        for pattern in &resistance_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if let Some(captures) = regex.captures(query) {
                    if let Some(value_str) = captures.get(1) {
                        if let Ok(value) = value_str.as_str().parse::<f64>() {
                            let unit = if pattern.contains("k") {
                                "kΩ"
                            } else if pattern.contains("m") {
                                "mΩ"
                            } else {
                                "Ω"
                            };
                            return Some(SpecValue::String(format!("{}{}", value, unit)));
                        }
                    }
                }
            }
        }

        None
    }

    /// Extract capacitance value from query
    fn extract_capacitance(&self, query: &str) -> Option<SpecValue> {
        // Look for patterns like "100nF", "10uF", "1pF"
        let capacitance_patterns = [
            r"(\d+(?:\.\d+)?)\s*pf",
            r"(\d+(?:\.\d+)?)\s*nf",
            r"(\d+(?:\.\d+)?)\s*uf",
            r"(\d+(?:\.\d+)?)\s*mf",
        ];

        for pattern in &capacitance_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if let Some(captures) = regex.captures(query) {
                    if let Some(value_str) = captures.get(1) {
                        if let Ok(value) = value_str.as_str().parse::<f64>() {
                            let unit = if pattern.contains("pf") {
                                "pF"
                            } else if pattern.contains("nf") {
                                "nF"
                            } else if pattern.contains("uf") {
                                "µF"
                            } else {
                                "mF"
                            };
                            return Some(SpecValue::String(format!("{}{}", value, unit)));
                        }
                    }
                }
            }
        }

        None
    }

    /// Extract voltage value from query
    fn extract_voltage(&self, query: &str) -> Option<SpecValue> {
        // Look for patterns like "5V", "3.3V", "12 volt"
        if let Ok(regex) = regex::Regex::new(r"(\d+(?:\.\d+)?)\s*v(?:olt)?") {
            if let Some(captures) = regex.captures(query) {
                if let Some(value_str) = captures.get(1) {
                    if let Ok(value) = value_str.as_str().parse::<f64>() {
                        return Some(SpecValue::String(format!("{}V", value)));
                    }
                }
            }
        }

        None
    }

    /// Extract package/footprint from query
    fn extract_package(&self, query: &str) -> Option<String> {
        let common_packages = [
            "0603", "0805", "1206", "1210", "2512",
            "sot23", "sot223", "soic", "tssop", "qfn",
            "bga", "dip", "sip", "to220", "to92",
        ];

        for package in &common_packages {
            if query.contains(package) {
                return Some(package.to_uppercase());
            }
        }

        None
    }

    /// Extract manufacturer from query
    fn extract_manufacturer(&self, query: &str) -> Option<String> {
        let common_manufacturers = [
            "texas instruments", "ti", "analog devices", "adi",
            "maxim", "linear technology", "ltc", "microchip",
            "atmel", "st", "stmicroelectronics", "nxp",
            "infineon", "vishay", "murata", "tdk",
            "samsung", "panasonic", "nichicon", "kemet",
        ];

        for manufacturer in &common_manufacturers {
            if query.contains(manufacturer) {
                return Some(manufacturer.to_string());
            }
        }

        None
    }

    /// Perform fuzzy search on part numbers
    fn fuzzy_search_part_numbers(&self, query: &str, limit: Option<u32>) -> Result<Vec<ComponentSearchResult>> {
        // For now, implement a simple fuzzy search
        // In a production system, you might use a more sophisticated algorithm like Levenshtein distance
        let mut results = Vec::new();
        
        // Get all components and calculate fuzzy match scores
        let all_components = self.db.get_components_by_category(&ComponentCategory::Resistors, None)?;
        
        for component in all_components {
            let similarity = self.calculate_fuzzy_similarity(&component.part_number, query);
            if similarity > 0.6 { // Threshold for fuzzy matching
                let result = ComponentSearchResult::new(component, similarity * 100.0)
                    .with_match_reason("Fuzzy part number match".to_string());
                results.push(result);
            }
        }

        // Sort by similarity score
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        // Apply limit
        if let Some(limit) = limit {
            results.truncate(limit as usize);
        }

        Ok(results)
    }

    /// Calculate fuzzy similarity between two strings
    fn calculate_fuzzy_similarity(&self, s1: &str, s2: &str) -> f64 {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        if s1_lower == s2_lower {
            return 1.0;
        }

        // Calculate Levenshtein-like similarity
        let len1 = s1_lower.len();
        let len2 = s2_lower.len();
        
        if len1 == 0 || len2 == 0 {
            return 0.0;
        }

        let max_len = len1.max(len2);
        let min_len = len1.min(len2);
        
        // Count matching characters in order
        let mut matches = 0;
        let chars1: Vec<char> = s1_lower.chars().collect();
        let chars2: Vec<char> = s2_lower.chars().collect();
        
        for i in 0..min_len {
            if chars1[i] == chars2[i] {
                matches += 1;
            }
        }
        
        // Add bonus for common substrings
        let mut common_chars = 0;
        for c in chars1.iter() {
            if chars2.contains(c) {
                common_chars += 1;
            }
        }
        
        // Calculate similarity score
        let position_similarity = matches as f64 / max_len as f64;
        let character_similarity = common_chars as f64 / max_len as f64;
        let length_similarity = min_len as f64 / max_len as f64;
        
        // Weighted average
        (position_similarity * 0.5 + character_similarity * 0.3 + length_similarity * 0.2)
    }

    /// Merge and deduplicate search results
    fn merge_and_deduplicate_results(&self, mut results: Vec<ComponentSearchResult>) -> Vec<ComponentSearchResult> {
        // Sort by component ID to group duplicates together
        results.sort_by(|a, b| a.component.id.cmp(&b.component.id));

        let mut merged = Vec::new();
        let mut current_id = String::new();
        let mut best_score = 0.0;
        let mut best_result: Option<ComponentSearchResult> = None;

        for result in results {
            if result.component.id != current_id {
                // Save the previous best result
                if let Some(best) = best_result.take() {
                    merged.push(best);
                }

                // Start tracking new component
                current_id = result.component.id.clone();
                best_score = result.relevance_score;
                best_result = Some(result);
            } else if result.relevance_score > best_score {
                // Update best result for this component
                best_score = result.relevance_score;
                best_result = Some(result);
            }
        }

        // Don't forget the last result
        if let Some(best) = best_result {
            merged.push(best);
        }

        // Sort by relevance score (highest first)
        merged.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resistance_extraction() {
        let engine = ComponentSearchEngine::new().unwrap();
        
        assert!(engine.extract_resistance("1k resistor").is_some());
        assert!(engine.extract_resistance("4.7kohm").is_some());
        assert!(engine.extract_resistance("100ohm").is_some());
        assert!(engine.extract_resistance("no resistance here").is_none());
    }

    #[test]
    fn test_capacitance_extraction() {
        let engine = ComponentSearchEngine::new().unwrap();
        
        assert!(engine.extract_capacitance("100nf capacitor").is_some());
        assert!(engine.extract_capacitance("10uf").is_some());
        assert!(engine.extract_capacitance("1pf").is_some());
        assert!(engine.extract_capacitance("no capacitance here").is_none());
    }

    #[test]
    fn test_voltage_extraction() {
        let engine = ComponentSearchEngine::new().unwrap();
        
        assert!(engine.extract_voltage("5v regulator").is_some());
        assert!(engine.extract_voltage("3.3 volt").is_some());
        assert!(engine.extract_voltage("no voltage here").is_none());
    }

    #[test]
    fn test_package_extraction() {
        let engine = ComponentSearchEngine::new().unwrap();
        
        assert_eq!(engine.extract_package("0603 resistor"), Some("0603".to_string()));
        assert_eq!(engine.extract_package("sot23 transistor"), Some("SOT23".to_string()));
        assert!(engine.extract_package("no package here").is_none());
    }

    #[test]
    fn test_fuzzy_similarity() {
        let engine = ComponentSearchEngine::new().unwrap();
        
        assert!(engine.calculate_fuzzy_similarity("R1234", "R1235") > 0.8);
        assert!(engine.calculate_fuzzy_similarity("R1234", "C1234") > 0.6);
        assert!(engine.calculate_fuzzy_similarity("R1234", "xyz") < 0.3);
    }
}