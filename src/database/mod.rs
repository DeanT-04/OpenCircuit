use anyhow::Result;
use rusqlite::{Connection, params, Row};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub mod schema;

/// Component record structure for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRecord {
    pub id: String,
    pub part_number: String,
    pub manufacturer: String,
    pub category: String,
    pub description: Option<String>,
    pub datasheet_url: Option<String>,
    pub specifications: Option<String>, // JSON string
    pub footprint: Option<String>,
    pub symbol: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Filter criteria for component searches
#[derive(Debug, Default)]
pub struct ComponentFilter {
    pub manufacturer: Option<String>,
    pub category: Option<String>,
    pub part_number_contains: Option<String>,
    pub description_contains: Option<String>,
}

/// Database connection wrapper with thread safety
pub struct Database {
    connection: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new database connection and initialize schema
    pub fn new() -> Result<Self> {
        let conn = schema::initialize_database()?;
        Ok(Database {
            connection: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create a new component record
    pub fn create_component(&self, component: &ComponentRecord) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO components (
                id, part_number, manufacturer, category, description,
                datasheet_url, specifications, footprint, symbol
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                component.id,
                component.part_number,
                component.manufacturer,
                component.category,
                component.description,
                component.datasheet_url,
                component.specifications,
                component.footprint,
                component.symbol
            ],
        )?;
        Ok(())
    }

    /// Get a component by ID
    pub fn get_component(&self, id: &str) -> Result<Option<ComponentRecord>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, part_number, manufacturer, category, description,
                   datasheet_url, specifications, footprint, symbol,
                   created_at, updated_at
            FROM components WHERE id = ?
            "#,
        )?;

        let component = stmt.query_row(params![id], |row| {
            Ok(ComponentRecord {
                id: row.get(0)?,
                part_number: row.get(1)?,
                manufacturer: row.get(2)?,
                category: row.get(3)?,
                description: row.get(4)?,
                datasheet_url: row.get(5)?,
                specifications: row.get(6)?,
                footprint: row.get(7)?,
                symbol: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        });

        match component {
            Ok(comp) => Ok(Some(comp)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Update an existing component
    pub fn update_component(&self, component: &ComponentRecord) -> Result<bool> {
        let conn = self.connection.lock().unwrap();
        let rows_affected = conn.execute(
            r#"
            UPDATE components SET
                part_number = ?, manufacturer = ?, category = ?, description = ?,
                datasheet_url = ?, specifications = ?, footprint = ?, symbol = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            params![
                component.part_number,
                component.manufacturer,
                component.category,
                component.description,
                component.datasheet_url,
                component.specifications,
                component.footprint,
                component.symbol,
                component.id
            ],
        )?;
        Ok(rows_affected > 0)
    }

    /// Delete a component by ID
    pub fn delete_component(&self, id: &str) -> Result<bool> {
        let conn = self.connection.lock().unwrap();
        let rows_affected = conn.execute("DELETE FROM components WHERE id = ?", params![id])?;
        Ok(rows_affected > 0)
    }

    /// Search components with text query
    pub fn search_components(&self, query: &str, limit: Option<u32>) -> Result<Vec<ComponentRecord>> {
        let conn = self.connection.lock().unwrap();
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        
        let sql = format!(
            r#"
            SELECT id, part_number, manufacturer, category, description,
                   datasheet_url, specifications, footprint, symbol,
                   created_at, updated_at
            FROM components 
            WHERE part_number LIKE ? OR manufacturer LIKE ? OR description LIKE ?
            ORDER BY part_number{}
            "#,
            limit_clause
        );

        let mut stmt = conn.prepare(&sql)?;
        let search_pattern = format!("%{}%", query);
        
        let component_iter = stmt.query_map(
            params![search_pattern, search_pattern, search_pattern],
            |row| {
                Ok(ComponentRecord {
                    id: row.get(0)?,
                    part_number: row.get(1)?,
                    manufacturer: row.get(2)?,
                    category: row.get(3)?,
                    description: row.get(4)?,
                    datasheet_url: row.get(5)?,
                    specifications: row.get(6)?,
                    footprint: row.get(7)?,
                    symbol: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            },
        )?;

        let mut components = Vec::new();
        for component in component_iter {
            components.push(component?);
        }
        Ok(components)
    }

    /// Filter components based on criteria
    pub fn filter_components(&self, filter: &ComponentFilter, limit: Option<u32>) -> Result<Vec<ComponentRecord>> {
        let conn = self.connection.lock().unwrap();
        
        let mut conditions = Vec::new();
        let mut params_vec: Vec<String> = Vec::new();
        
        if let Some(ref manufacturer) = filter.manufacturer {
            conditions.push("manufacturer = ?");
            params_vec.push(manufacturer.clone());
        }
        
        if let Some(ref category) = filter.category {
            conditions.push("category = ?");
            params_vec.push(category.clone());
        }
        
        if let Some(ref part_number) = filter.part_number_contains {
            conditions.push("part_number LIKE ?");
            params_vec.push(format!("%{}%", part_number));
        }
        
        if let Some(ref description) = filter.description_contains {
            conditions.push("description LIKE ?");
            params_vec.push(format!("%{}%", description));
        }
        
        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };
        
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        
        let sql = format!(
            r#"
            SELECT id, part_number, manufacturer, category, description,
                   datasheet_url, specifications, footprint, symbol,
                   created_at, updated_at
            FROM components{}
            ORDER BY part_number{}
            "#,
            where_clause, limit_clause
        );

        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&str> = params_vec.iter().map(|s| s.as_str()).collect();
        let component_iter = stmt.query_map(rusqlite::params_from_iter(params_refs), |row| {
            Ok(ComponentRecord {
                id: row.get(0)?,
                part_number: row.get(1)?,
                manufacturer: row.get(2)?,
                category: row.get(3)?,
                description: row.get(4)?,
                datasheet_url: row.get(5)?,
                specifications: row.get(6)?,
                footprint: row.get(7)?,
                symbol: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;

        let mut components = Vec::new();
        for component in component_iter {
            components.push(component?);
        }
        Ok(components)
    }

    /// Get all available component categories
    pub fn get_categories(&self) -> Result<Vec<(String, Option<String>)>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare("SELECT name, description FROM component_categories ORDER BY name")?;
        
        let category_iter = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })?;

        let mut categories = Vec::new();
        for category in category_iter {
            categories.push(category?);
        }
        Ok(categories)
    }

    /// Get component count by category
    pub fn get_component_count_by_category(&self) -> Result<Vec<(String, i64)>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT category, COUNT(*) as count 
            FROM components 
            GROUP BY category 
            ORDER BY count DESC
            "#,
        )?;
        
        let count_iter = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut counts = Vec::new();
        for count in count_iter {
            counts.push(count?);
        }
        Ok(counts)
    }

    /// Get total component count
    pub fn get_total_component_count(&self) -> Result<i64> {
        let conn = self.connection.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM components", [], |row| row.get(0))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use uuid::Uuid;

    fn create_test_component() -> ComponentRecord {
        ComponentRecord {
            id: Uuid::new_v4().to_string(),
            part_number: "R1234".to_string(),
            manufacturer: "Test Corp".to_string(),
            category: "Resistors".to_string(),
            description: Some("Test resistor".to_string()),
            datasheet_url: Some("https://example.com/datasheet.pdf".to_string()),
            specifications: Some(r#"{"resistance": "1k", "tolerance": "5%"}"#.to_string()),
            footprint: Some("0603".to_string()),
            symbol: Some("resistor".to_string()),
            created_at: "2025-01-27T12:00:00Z".to_string(),
            updated_at: "2025-01-27T12:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_database_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        schema::run_migrations(&conn).unwrap();
        
        let db = Database {
            connection: Arc::new(Mutex::new(conn)),
        };
        
        // Database creation should succeed
        assert!(db.get_categories().is_ok());
    }

    #[test]
    fn test_component_crud() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        schema::run_migrations(&conn).unwrap();
        
        let db = Database {
            connection: Arc::new(Mutex::new(conn)),
        };
        
        let component = create_test_component();
        let original_id = component.id.clone();
        
        // Create component
        db.create_component(&component).unwrap();
        
        // Read component
        let retrieved = db.get_component(&original_id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.part_number, "R1234");
        assert_eq!(retrieved.manufacturer, "Test Corp");
        
        // Update component
        let mut updated = retrieved.clone();
        updated.description = Some("Updated description".to_string());
        db.update_component(&updated).unwrap();
        
        let retrieved_updated = db.get_component(&original_id).unwrap().unwrap();
        assert_eq!(retrieved_updated.description, Some("Updated description".to_string()));
        
        // Delete component
        let deleted = db.delete_component(&original_id).unwrap();
        assert!(deleted);
        
        let retrieved_deleted = db.get_component(&original_id).unwrap();
        assert!(retrieved_deleted.is_none());
    }

    #[test]
    fn test_component_search() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        schema::run_migrations(&conn).unwrap();
        
        let db = Database {
            connection: Arc::new(Mutex::new(conn)),
        };
        
        let component = create_test_component();
        db.create_component(&component).unwrap();
        
        // Search by part number
        let results = db.search_components("R1234", None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].part_number, "R1234");
        
        // Search by manufacturer
        let results = db.search_components("Test Corp", None).unwrap();
        assert_eq!(results.len(), 1);
        
        // Search with no results
        let results = db.search_components("nonexistent", None).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_categories() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        schema::run_migrations(&conn).unwrap();
        
        let db = Database {
            connection: Arc::new(Mutex::new(conn)),
        };
        
        let categories = db.get_categories().unwrap();
        assert!(categories.len() >= 10); // Should have default categories
        assert!(categories.iter().any(|(name, _)| name == "Resistors"));
    }
}