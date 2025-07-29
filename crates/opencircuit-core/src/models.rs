use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for components
pub type ComponentId = String;

/// Component category enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentCategory {
    Resistors,
    Capacitors,
    Inductors,
    Diodes,
    Transistors,
    IntegratedCircuits,
    Connectors,
    Switches,
    Crystals,
    Sensors,
    Power,
    Mechanical,
    Custom(String),
}

impl ComponentCategory {
    pub fn as_str(&self) -> &str {
        match self {
            ComponentCategory::Resistors => "Resistors",
            ComponentCategory::Capacitors => "Capacitors",
            ComponentCategory::Inductors => "Inductors",
            ComponentCategory::Diodes => "Diodes",
            ComponentCategory::Transistors => "Transistors",
            ComponentCategory::IntegratedCircuits => "Integrated Circuits",
            ComponentCategory::Connectors => "Connectors",
            ComponentCategory::Switches => "Switches",
            ComponentCategory::Crystals => "Crystals",
            ComponentCategory::Sensors => "Sensors",
            ComponentCategory::Power => "Power",
            ComponentCategory::Mechanical => "Mechanical",
            ComponentCategory::Custom(name) => name,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Resistors" => ComponentCategory::Resistors,
            "Capacitors" => ComponentCategory::Capacitors,
            "Inductors" => ComponentCategory::Inductors,
            "Diodes" => ComponentCategory::Diodes,
            "Transistors" => ComponentCategory::Transistors,
            "Integrated Circuits" => ComponentCategory::IntegratedCircuits,
            "Connectors" => ComponentCategory::Connectors,
            "Switches" => ComponentCategory::Switches,
            "Crystals" => ComponentCategory::Crystals,
            "Sensors" => ComponentCategory::Sensors,
            "Power" => ComponentCategory::Power,
            "Mechanical" => ComponentCategory::Mechanical,
            _ => ComponentCategory::Custom(s.to_string()),
        }
    }
}

/// Component specification value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpecValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Range { min: f64, max: f64, unit: Option<String> },
    List(Vec<String>),
}

impl SpecValue {
    pub fn as_string(&self) -> String {
        match self {
            SpecValue::String(s) => s.clone(),
            SpecValue::Number(n) => n.to_string(),
            SpecValue::Integer(i) => i.to_string(),
            SpecValue::Boolean(b) => b.to_string(),
            SpecValue::Range { min, max, unit } => {
                if let Some(unit) = unit {
                    format!("{}-{} {}", min, max, unit)
                } else {
                    format!("{}-{}", min, max)
                }
            }
            SpecValue::List(list) => list.join(", "),
        }
    }
}

/// Component pricing information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceInfo {
    pub currency: String,
    pub price_breaks: Vec<PriceBreak>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub supplier: String,
}

/// Price break for quantity pricing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceBreak {
    pub quantity: u32,
    pub unit_price: f64,
}

/// Component availability information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AvailabilityInfo {
    pub in_stock: bool,
    pub quantity_available: Option<u32>,
    pub lead_time_days: Option<u32>,
    pub minimum_order_quantity: Option<u32>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub supplier: String,
}

/// Core component model
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    pub id: ComponentId,
    pub part_number: String,
    pub manufacturer: String,
    pub category: ComponentCategory,
    pub description: String,
    pub specifications: HashMap<String, SpecValue>,
    pub footprint: Option<String>,
    pub symbol: Option<String>,
    pub datasheet_url: Option<String>,
    pub price_info: Option<PriceInfo>,
    pub availability: Option<AvailabilityInfo>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Component {
    pub fn new(
        part_number: String,
        manufacturer: String,
        category: ComponentCategory,
        description: String,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            part_number,
            manufacturer,
            category,
            description,
            specifications: HashMap::new(),
            footprint: None,
            symbol: None,
            datasheet_url: None,
            price_info: None,
            availability: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_id(mut self, id: ComponentId) -> Self {
        self.id = id;
        self
    }

    pub fn with_specifications(mut self, specs: HashMap<String, SpecValue>) -> Self {
        self.specifications = specs;
        self
    }

    pub fn with_footprint(mut self, footprint: String) -> Self {
        self.footprint = Some(footprint);
        self
    }

    pub fn with_symbol(mut self, symbol: String) -> Self {
        self.symbol = Some(symbol);
        self
    }

    pub fn with_datasheet(mut self, url: String) -> Self {
        self.datasheet_url = Some(url);
        self
    }

    pub fn with_price_info(mut self, price_info: PriceInfo) -> Self {
        self.price_info = Some(price_info);
        self
    }

    pub fn with_availability(mut self, availability: AvailabilityInfo) -> Self {
        self.availability = Some(availability);
        self
    }

    pub fn update(&mut self) {
        self.updated_at = chrono::Utc::now();
    }

    /// Get a specification value by key
    pub fn get_spec(&self, key: &str) -> Option<&SpecValue> {
        self.specifications.get(key)
    }

    /// Set a specification value
    pub fn set_spec(&mut self, key: String, value: SpecValue) {
        self.specifications.insert(key, value);
        self.update();
    }

    /// Check if component matches search criteria
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        
        // Search in part number
        if self.part_number.to_lowercase().contains(&query_lower) {
            return true;
        }
        
        // Search in manufacturer
        if self.manufacturer.to_lowercase().contains(&query_lower) {
            return true;
        }
        
        // Search in description
        if self.description.to_lowercase().contains(&query_lower) {
            return true;
        }
        
        // Search in category
        if self.category.as_str().to_lowercase().contains(&query_lower) {
            return true;
        }
        
        // Search in specifications
        for (key, value) in &self.specifications {
            if key.to_lowercase().contains(&query_lower) {
                return true;
            }
            if value.as_string().to_lowercase().contains(&query_lower) {
                return true;
            }
        }
        
        false
    }
}

/// Search filter criteria
#[derive(Debug, Clone, Default)]
pub struct ComponentSearchFilter {
    pub manufacturer: Option<String>,
    pub category: Option<ComponentCategory>,
    pub part_number_contains: Option<String>,
    pub description_contains: Option<String>,
    pub specifications: HashMap<String, SpecValue>,
    pub has_datasheet: Option<bool>,
    pub has_footprint: Option<bool>,
    pub in_stock_only: Option<bool>,
}

impl ComponentSearchFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_manufacturer(mut self, manufacturer: String) -> Self {
        self.manufacturer = Some(manufacturer);
        self
    }

    pub fn with_category(mut self, category: ComponentCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub fn with_part_number_contains(mut self, part_number: String) -> Self {
        self.part_number_contains = Some(part_number);
        self
    }

    pub fn with_description_contains(mut self, description: String) -> Self {
        self.description_contains = Some(description);
        self
    }

    pub fn with_specification(mut self, key: String, value: SpecValue) -> Self {
        self.specifications.insert(key, value);
        self
    }

    pub fn with_datasheet_required(mut self) -> Self {
        self.has_datasheet = Some(true);
        self
    }

    pub fn with_footprint_required(mut self) -> Self {
        self.has_footprint = Some(true);
        self
    }

    pub fn in_stock_only(mut self) -> Self {
        self.in_stock_only = Some(true);
        self
    }

    /// Check if a component matches this filter
    pub fn matches(&self, component: &Component) -> bool {
        // Check manufacturer
        if let Some(ref manufacturer) = self.manufacturer {
            if !component.manufacturer.eq_ignore_ascii_case(manufacturer) {
                return false;
            }
        }

        // Check category
        if let Some(ref category) = self.category {
            if component.category != *category {
                return false;
            }
        }

        // Check part number contains
        if let Some(ref part_number) = self.part_number_contains {
            if !component.part_number.to_lowercase().contains(&part_number.to_lowercase()) {
                return false;
            }
        }

        // Check description contains
        if let Some(ref description) = self.description_contains {
            if !component.description.to_lowercase().contains(&description.to_lowercase()) {
                return false;
            }
        }

        // Check specifications
        for (key, expected_value) in &self.specifications {
            if let Some(actual_value) = component.get_spec(key) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check datasheet requirement
        if let Some(true) = self.has_datasheet {
            if component.datasheet_url.is_none() {
                return false;
            }
        }

        // Check footprint requirement
        if let Some(true) = self.has_footprint {
            if component.footprint.is_none() {
                return false;
            }
        }

        // Check stock requirement
        if let Some(true) = self.in_stock_only {
            if let Some(ref availability) = component.availability {
                if !availability.in_stock {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

/// Search result with relevance scoring
#[derive(Debug, Clone)]
pub struct ComponentSearchResult {
    pub component: Component,
    pub relevance_score: f64,
    pub match_reasons: Vec<String>,
}

impl ComponentSearchResult {
    pub fn new(component: Component, relevance_score: f64) -> Self {
        Self {
            component,
            relevance_score,
            match_reasons: Vec::new(),
        }
    }

    pub fn with_match_reason(mut self, reason: String) -> Self {
        self.match_reasons.push(reason);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_creation() {
        let component = Component::new(
            "R1234".to_string(),
            "Test Corp".to_string(),
            ComponentCategory::Resistors,
            "Test resistor".to_string(),
        );

        assert_eq!(component.part_number, "R1234");
        assert_eq!(component.manufacturer, "Test Corp");
        assert_eq!(component.category, ComponentCategory::Resistors);
        assert_eq!(component.description, "Test resistor");
        assert!(!component.id.is_empty());
    }

    #[test]
    fn test_component_search_matching() {
        let mut component = Component::new(
            "R1234".to_string(),
            "Test Corp".to_string(),
            ComponentCategory::Resistors,
            "1k ohm resistor".to_string(),
        );

        component.set_spec("resistance".to_string(), SpecValue::String("1k".to_string()));

        assert!(component.matches_search("R1234"));
        assert!(component.matches_search("Test Corp"));
        assert!(component.matches_search("resistor"));
        assert!(component.matches_search("1k"));
        assert!(!component.matches_search("capacitor"));
    }

    #[test]
    fn test_component_filter() {
        let component = Component::new(
            "R1234".to_string(),
            "Test Corp".to_string(),
            ComponentCategory::Resistors,
            "Test resistor".to_string(),
        );

        let filter = ComponentSearchFilter::new()
            .with_manufacturer("Test Corp".to_string())
            .with_category(ComponentCategory::Resistors);

        assert!(filter.matches(&component));

        let filter2 = ComponentSearchFilter::new()
            .with_manufacturer("Other Corp".to_string());

        assert!(!filter2.matches(&component));
    }

    #[test]
    fn test_category_conversion() {
        assert_eq!(ComponentCategory::Resistors.as_str(), "Resistors");
        assert_eq!(ComponentCategory::from_str("Resistors"), ComponentCategory::Resistors);
        assert_eq!(ComponentCategory::from_str("Custom Category"), ComponentCategory::Custom("Custom Category".to_string()));
    }

    #[test]
    fn test_spec_value_string_conversion() {
        let spec = SpecValue::Range { min: 1.0, max: 10.0, unit: Some("V".to_string()) };
        assert_eq!(spec.as_string(), "1-10 V");

        let spec2 = SpecValue::List(vec!["A".to_string(), "B".to_string()]);
        assert_eq!(spec2.as_string(), "A, B");
    }
}