use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::PathBuf;

/// Initialize database and run migrations
pub fn initialize_database() -> Result<Connection> {
    let db_path = get_database_path()?;
    let conn = Connection::open(&db_path)?;
    run_migrations(&conn)?;
    Ok(conn)
}

/// Run all database migrations
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    
    // Create migrations table if it doesn't exist
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )?;
    
    // Check if migration 001 has been applied
    let migration_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM migrations WHERE name = ?)",
            params!["001_initial"],
            |row| row.get(0),
        )?;
    
    if !migration_exists {
        apply_migration_001(conn)?;
        conn.execute(
            "INSERT INTO migrations (name) VALUES (?)",
            params!["001_initial"],
        )?;
    }
    
    Ok(())
}

/// Apply the initial migration
fn apply_migration_001(conn: &Connection) -> Result<()> {
    // Create component_categories table
    conn.execute(
        r#"
        CREATE TABLE component_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )?;
    
    // Create components table
    conn.execute(
        r#"
        CREATE TABLE components (
            id TEXT PRIMARY KEY,
            part_number TEXT NOT NULL,
            manufacturer TEXT NOT NULL,
            category TEXT NOT NULL,
            description TEXT,
            datasheet_url TEXT,
            specifications TEXT,
            footprint TEXT,
            symbol TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (category) REFERENCES component_categories(name)
        )
        "#,
        [],
    )?;
    
    // Create component_vectors table for AI embeddings
    conn.execute(
        r#"
        CREATE TABLE component_vectors (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            component_id TEXT NOT NULL,
            vector_data BLOB,
            vector_type TEXT NOT NULL DEFAULT 'description',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (component_id) REFERENCES components(id) ON DELETE CASCADE
        )
        "#,
        [],
    )?;
    
    // Create indexes for performance
    conn.execute("CREATE INDEX idx_components_part_number ON components(part_number)", [])?;
    conn.execute("CREATE INDEX idx_components_manufacturer ON components(manufacturer)", [])?;
    conn.execute("CREATE INDEX idx_components_category ON components(category)", [])?;
    conn.execute("CREATE INDEX idx_component_vectors_component_id ON component_vectors(component_id)", [])?;
    
    // Insert default component categories
    let categories = [
        ("Resistors", "Fixed and variable resistors"),
        ("Capacitors", "Ceramic, electrolytic, and film capacitors"),
        ("Inductors", "Coils and chokes"),
        ("Diodes", "Signal, power, and Zener diodes"),
        ("Transistors", "BJT, FET, and MOSFET transistors"),
        ("Integrated Circuits", "Analog and digital ICs"),
        ("Connectors", "Headers, sockets, and terminal blocks"),
        ("Switches", "Tactile, toggle, and rotary switches"),
        ("Crystals", "Oscillators and resonators"),
        ("Sensors", "Temperature, pressure, and motion sensors"),
        ("Power", "Voltage regulators and power modules"),
        ("Mechanical", "Enclosures, heat sinks, and hardware"),
    ];
    
    for (name, description) in &categories {
        conn.execute(
            "INSERT INTO component_categories (name, description) VALUES (?, ?)",
            params![name, description],
        )?;
    }
    
    Ok(())
}

/// Get the database file path
pub fn get_database_path() -> Result<PathBuf> {
    let app_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?
        .join("OpenCircuit");
    
    std::fs::create_dir_all(&app_dir)?;
    Ok(app_dir.join("components.db"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_initialization() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        
        run_migrations(&conn).unwrap();
        
        // Check that tables were created
        let table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('components', 'component_categories', 'component_vectors')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        
        assert_eq!(table_count, 3);
    }

    #[test]
    fn test_migration_idempotency() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        
        // Run migrations twice
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();
        
        // Should not fail and should have the same result
        let migration_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM migrations", [], |row| row.get(0))
            .unwrap();
        
        assert_eq!(migration_count, 1);
    }
}