-- Migration 001: Initial component database schema
-- This migration creates the basic component storage tables
-- Version: 1
-- Applied: 2025-01-27

-- Create components table for storing electronic component information
CREATE TABLE IF NOT EXISTS components (
    id TEXT PRIMARY KEY,                    -- UUID as string
    part_number TEXT UNIQUE NOT NULL,       -- Manufacturer part number
    manufacturer TEXT NOT NULL,             -- Component manufacturer
    category TEXT NOT NULL,                 -- Component category (resistor, capacitor, etc.)
    description TEXT,                       -- Human-readable description
    datasheet_url TEXT,                     -- URL to component datasheet
    specifications TEXT,                    -- JSON specifications (resistance, capacitance, etc.)
    footprint TEXT,                         -- PCB footprint name
    symbol TEXT,                           -- Schematic symbol name
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create component_vectors table for AI embeddings
CREATE TABLE IF NOT EXISTS component_vectors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    component_id TEXT NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    vector BLOB,                           -- Embedding vector as binary data
    embedding_model TEXT NOT NULL,         -- Model used to generate embedding
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create component_categories table for hierarchical categorization
CREATE TABLE IF NOT EXISTS component_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,             -- Category name
    parent_id INTEGER REFERENCES component_categories(id), -- Parent category for hierarchy
    description TEXT,                      -- Category description
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create schema_version table for migration tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_components_part_number ON components(part_number);
CREATE INDEX IF NOT EXISTS idx_components_manufacturer ON components(manufacturer);
CREATE INDEX IF NOT EXISTS idx_components_category ON components(category);
CREATE INDEX IF NOT EXISTS idx_component_vectors_component_id ON component_vectors(component_id);
CREATE INDEX IF NOT EXISTS idx_component_categories_parent ON component_categories(parent_id);

-- Insert default component categories
INSERT OR IGNORE INTO component_categories (name, description) VALUES
    ('Resistors', 'Passive components that resist current flow'),
    ('Capacitors', 'Passive components that store electrical energy'),
    ('Inductors', 'Passive components that store magnetic energy'),
    ('Diodes', 'Semiconductor devices that allow current in one direction'),
    ('Transistors', 'Semiconductor devices for switching and amplification'),
    ('Integrated Circuits', 'Complex semiconductor devices with multiple functions'),
    ('Connectors', 'Components for electrical connections'),
    ('Switches', 'Components for controlling electrical connections'),
    ('Sensors', 'Components that detect physical phenomena'),
    ('Power Management', 'Components for power regulation and distribution');

-- Record this migration
INSERT OR IGNORE INTO schema_version (version) VALUES (1);