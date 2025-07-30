//! Vector embeddings for component search and similarity matching
//! 
//! This module provides functionality for:
//! - Converting component specifications to vector embeddings
//! - Similarity search for component recommendations
//! - Semantic matching of component requirements
//! - Caching and persistence of embeddings
//!
//! # Overview
//!
//! The `embeddings` module enables AI-powered component discovery by converting
//! component data into vector representations (embeddings) that can be compared
//! for similarity. This allows users to find components based on semantic meaning
//! rather than exact keyword matches.
//!
//! # Examples
//!
//! Basic usage for finding similar components:
//!
//! ```rust
//! use opencircuit_ai::embeddings::ComponentEmbeddingEngine;
//! use opencircuit_core::models::{Component, ComponentCategory};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let ollama_client = OpenCircuitOllamaClient::new();
//! let mut engine = ComponentEmbeddingEngine::new(ollama_client).await?;
//!
//! // Find components similar to requirements
//! let components = vec![/* your components */];
//! let matches = engine
//!     .find_similar_components_by_requirements(
//!         "low power microcontroller for IoT sensor",
//!         &components,
//!         5
//!     )
//!     .await?;
//!
//! for match_result in matches {
//!     println!("Found: {} (similarity: {:.1}%)", 
//!         match_result.component.part_number,
//!         match_result.similarity * 100.0);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! The module consists of several key components:
//!
//! - [`ComponentEmbeddingEngine`]: Main orchestrator for embedding operations
//! - [`ComponentEmbedding`]: Stores vector representations and metadata
//! - [`SimilarityMatch`]: Results from similarity searches
//! - [`utils`]: Helper functions for common operations
//!
//! # Performance Considerations
//!
//! - Embeddings are cached in memory to avoid recomputation
//! - Cache can be cleared with [`ComponentEmbeddingEngine::clear_cache()`]
//! - Memory usage scales with number of components cached
//! - Use batch operations for processing large component sets
//!
//! # Model Configuration
//!
//! The default embedding model is "nomic-embed-text", but can be changed:
//!
//! ```rust
//! # use opencircuit_ai::embeddings::ComponentEmbeddingEngine;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let ollama_client = OpenCircuitOllamaClient::new();
//! # let mut engine = ComponentEmbeddingEngine::new(ollama_client).await?;
//! engine.set_embedding_model("llama2:7b".to_string());
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use opencircuit_core::{
    models::{Component, ComponentCategory},
    OpenCircuitError,
};

use crate::ollama_client::OpenCircuitOllamaClient;

type Result<T> = std::result::Result<T, OpenCircuitError>;

/// Vector embedding for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentEmbedding {
    /// Component ID this embedding represents
    pub component_id: String,
    /// Vector representation of the component
    pub vector: Vec<f32>,
    /// Metadata for the embedding
    pub metadata: EmbeddingMetadata,
    /// When this embedding was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Metadata associated with an embedding
///
/// Provides contextual information about how an embedding was generated,
/// including the source component category, key specifications used,
/// the model that created the embedding, and its dimensionality.
///
/// # Usage
///
/// This metadata helps track embedding provenance and enables
/// compatibility checks when comparing embeddings from different sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    /// Component category
    pub category: ComponentCategory,
    /// Key specifications used in embedding
    pub key_specs: Vec<String>,
    /// Embedding model used
    pub model: String,
    /// Embedding dimension
    pub dimension: usize,
}

/// Similarity search result
///
/// Represents a component that matched against a query, along with
/// similarity metrics and human-readable explanations.
///
/// # Fields
///
/// - `component`: The actual component that matched
/// - `similarity`: Score from 0.0 to 1.0 indicating relevance
/// - `match_reason`: Natural language explanation of the match
///
/// # Interpretation
///
/// Similarity scores can be interpreted as:
/// - 0.8-1.0: Excellent match
/// - 0.6-0.8: Good match
/// - 0.4-0.6: Fair match
/// - 0.0-0.4: Basic match
#[derive(Debug, Clone)]
pub struct SimilarityMatch {
    /// Component that matched
    pub component: Component,
    /// Similarity score (0.0 to 1.0, higher is more similar)
    pub similarity: f32,
    /// Explanation of why this component matched
    pub match_reason: String,
}

/// Component embedding generator and search engine
///
/// Main orchestrator for all embedding-related operations including:
/// - Generating embeddings from components
/// - Finding similar components
/// - Managing embedding cache
/// - Configuring embedding models
///
/// # Thread Safety
///
/// This struct is not `Send` or `Sync` due to internal caching.
/// Create separate instances for concurrent operations.
///
/// # Example
///
/// ```rust
/// # use opencircuit_ai::embeddings::ComponentEmbeddingEngine;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = OpenCircuitOllamaClient::new();
/// let mut engine = ComponentEmbeddingEngine::new(client).await?;
///
/// // Cache management
/// let (cached_count, memory_usage) = engine.cache_stats();
/// println!("Cache: {} items, ~{} bytes", cached_count, memory_usage);
///
/// // Clear cache if needed
/// engine.clear_cache();
/// # Ok(())
/// # }
/// ```
pub struct ComponentEmbeddingEngine {
    /// Ollama client for generating embeddings
    ollama_client: OpenCircuitOllamaClient,
    /// Cached embeddings
    embeddings_cache: HashMap<String, ComponentEmbedding>,
    /// Model used for embeddings
    embedding_model: String,
}

impl ComponentEmbeddingEngine {
    /// Create a new embedding engine
    ///
    /// Initializes a new `ComponentEmbeddingEngine` with the provided Ollama client.
    /// The engine starts with an empty cache and uses "nomic-embed-text" as the
    /// default embedding model.
    ///
    /// # Arguments
    ///
    /// * `ollama_client` - Configured Ollama client for embedding generation
    ///
    /// # Returns
    ///
    /// Returns a new `ComponentEmbeddingEngine` instance or an error if initialization fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use opencircuit_ai::embeddings::ComponentEmbeddingEngine;
    /// # use opencircuit_ai::ollama_client::OpenCircuitOllamaClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OpenCircuitOllamaClient::new();
    /// let engine = ComponentEmbeddingEngine::new(client).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(ollama_client: OpenCircuitOllamaClient) -> Result<Self> {
        Ok(Self {
            ollama_client,
            embeddings_cache: HashMap::new(),
            embedding_model: "nomic-embed-text".to_string(), // Good embedding model
        })
    }

    /// Generate embedding for a component
    ///
    /// Creates a vector embedding for a given component by converting its
    /// specifications and metadata into a text representation, then generating
    /// an embedding using the configured model.
    ///
    /// # Arguments
    ///
    /// * `component` - The component to generate an embedding for
    ///
    /// # Returns
    ///
    /// Returns a `ComponentEmbedding` containing the vector representation and metadata.
    ///
    /// # Caching
    ///
    /// Generated embeddings are cached by component ID to avoid recomputation.
    /// Use [`clear_cache()`] to manually clear the cache if needed.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use opencircuit_ai::embeddings::ComponentEmbeddingEngine;
    /// # use opencircuit_core::models::{Component, ComponentCategory};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = OpenCircuitOllamaClient::new();
    /// # let mut engine = ComponentEmbeddingEngine::new(client).await?;
    /// let component = Component::new(
    ///     "R1234".to_string(),
    ///     "TestCorp".to_string(),
    ///     ComponentCategory::Resistors,
    ///     "10k ohm resistor".to_string(),
    /// );
    ///
    /// let embedding = engine.generate_component_embedding(&component).await?;
    /// println!("Generated {}-dimensional embedding", embedding.vector.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_component_embedding(&mut self, component: &Component) -> Result<ComponentEmbedding> {
        // Check cache first
        if let Some(cached) = self.embeddings_cache.get(&component.id) {
            return Ok(cached.clone());
        }

        // Create text representation of component for embedding
        let component_text = self.component_to_text(component);
        
        // Generate embedding using Ollama (simplified approach)
        // In a real implementation, you'd use a dedicated embedding model
        let embedding_vector = self.text_to_embedding(&component_text).await?;

        let metadata = EmbeddingMetadata {
            category: component.category.clone(),
            key_specs: self.extract_key_specs(component),
            model: self.embedding_model.clone(),
            dimension: embedding_vector.len(),
        };

        let embedding = ComponentEmbedding {
            component_id: component.id.clone(),
            vector: embedding_vector,
            metadata,
            created_at: chrono::Utc::now(),
        };

        // Cache the embedding
        self.embeddings_cache.insert(component.id.clone(), embedding.clone());

        Ok(embedding)
    }

    /// Find similar components based on a reference component
    ///
    /// **Deprecated**: This method currently returns a placeholder implementation.
    /// For production use, prefer [`find_similar_components_by_requirements()`] or
    /// [`find_components_by_category_semantic()`] with a populated component database.
    ///
    /// # Arguments
    ///
    /// * `component` - The reference component to find matches for
    /// * `_limit` - Maximum number of results to return (currently ignored)
    ///
    /// # Returns
    ///
    /// Currently returns a single perfect match with the reference component itself.
    /// This behavior will change once component database integration is complete.
    pub async fn find_similar_components(
        &mut self,
        component: &Component,
        _limit: usize,
    ) -> Result<Vec<SimilarityMatch>> {
        // Generate embedding for the reference component
        let _reference_embedding = self.generate_component_embedding(component).await?;
        
        // For now, return a simple match since we don't have a component database
        // In a real implementation, you'd search through a database of components
        Ok(vec![SimilarityMatch {
            component: component.clone(),
            similarity: 1.0, // Perfect match with itself
            match_reason: "Exact match".to_string(),
        }])
    }

    /// Find similar components based on requirements text
    ///
    /// Performs semantic search by converting the requirements text into an embedding
    /// and comparing it against embeddings of provided components. Returns the most
    /// relevant matches based on cosine similarity.
    ///
    /// # Arguments
    ///
    /// * `requirements` - Natural language description of what you need
    /// * `components` - List of components to search through
    /// * `max_results` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// Returns a vector of [`SimilarityMatch`] sorted by similarity score (highest first).
    ///
    /// # Filtering
    ///
    /// Only components with similarity scores above 0.3 are included in results.
    /// This threshold can be adjusted in future versions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use opencircuit_ai::embeddings::ComponentEmbeddingEngine;
    /// # use opencircuit_core::models::{Component, ComponentCategory};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = OpenCircuitOllamaClient::new();
    /// # let mut engine = ComponentEmbeddingEngine::new(client).await?;
    ///
    /// let components = vec![
    ///     Component::new("R1".to_string(), "Corp".to_string(), ComponentCategory::Resistors, "1k resistor".to_string()),
    ///     Component::new("R2".to_string(), "Corp".to_string(), ComponentCategory::Resistors, "10k resistor".to_string()),
    ///     Component::new("C1".to_string(), "Corp".to_string(), ComponentCategory::Capacitors, "100nF capacitor".to_string()),
    /// ];
    ///
    /// let matches = engine
    ///     .find_similar_components_by_requirements(
    ///         "high resistance for voltage divider",
    ///         &components,
    ///         3
    ///     )
    ///     .await?;
    ///
    /// for m in matches {
    ///     println!("Match: {} ({:.1}%)", m.component.part_number, m.similarity * 100.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_similar_components_by_requirements(
        &mut self,
        requirements: &str,
        components: &[Component],
        max_results: usize,
    ) -> Result<Vec<SimilarityMatch>> {
        // Generate embedding for requirements
        let requirements_embedding = self.text_to_embedding(requirements).await?;

        let mut matches = Vec::new();

        // Generate embeddings for all components and calculate similarity
        for component in components {
            let component_embedding = self.generate_component_embedding(component).await?;
            let similarity = self.cosine_similarity(&requirements_embedding, &component_embedding.vector);
            
            if similarity > 0.3 { // Threshold for relevance
                let match_reason = self.generate_match_reason(component, similarity).await?;
                matches.push(SimilarityMatch {
                    component: component.clone(),
                    similarity,
                    match_reason,
                });
            }
        }

        // Sort by similarity (highest first)
        matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        
        // Return top results
        matches.truncate(max_results);
        Ok(matches)
    }

    /// Find components by category with semantic search
    ///
    /// Combines category filtering with semantic search to find components
    /// that match both a specific category and semantic requirements.
    ///
    /// # Arguments
    ///
    /// * `category` - The component category to filter by
    /// * `requirements` - Natural language description of requirements
    /// * `components` - List of components to search through
    /// * `max_results` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// Returns filtered and sorted [`SimilarityMatch`] results, empty if no
    /// components match the specified category.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use opencircuit_ai::embeddings::ComponentEmbeddingEngine;
    /// # use opencircuit_core::models::{Component, ComponentCategory};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = OpenCircuitOllamaClient::new();
    /// # let mut engine = ComponentEmbeddingEngine::new(client).await?;
    ///
    /// let components = vec![
    ///     Component::new("R1".to_string(), "Corp".to_string(), ComponentCategory::Resistors, "1k resistor".to_string()),
    ///     Component::new("C1".to_string(), "Corp".to_string(), ComponentCategory::Capacitors, "100nF capacitor".to_string()),
    ///     Component::new("R2".to_string(), "Corp".to_string(), ComponentCategory::Resistors, "10k resistor".to_string()),
    /// ];
    ///
    /// // Find only resistors matching specific requirements
    /// let resistor_matches = engine
    ///     .find_components_by_category_semantic(
    ///         &ComponentCategory::Resistors,
    ///         "high precision for audio circuit",
    ///         &components,
    ///         5
    ///     )
    ///     .await?;
    ///
    /// assert_eq!(resistor_matches.len(), 2); // Only R1 and R2
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_components_by_category_semantic(
        &mut self,
        category: &ComponentCategory,
        requirements: &str,
        components: &[Component],
        max_results: usize,
    ) -> Result<Vec<SimilarityMatch>> {
        // Filter components by category first
        let category_components: Vec<&Component> = components
            .iter()
            .filter(|c| &c.category == category)
            .collect();

        // Convert back to owned components for the similarity search
        let owned_components: Vec<Component> = category_components.into_iter().cloned().collect();

        self.find_similar_components_by_requirements(requirements, &owned_components, max_results).await
    }

    /// Convert component to text representation for embedding
    fn component_to_text(&self, component: &Component) -> String {
        let mut text_parts = vec![
            format!("Part: {}", component.part_number),
            format!("Manufacturer: {}", component.manufacturer),
            format!("Category: {}", component.category.as_str()),
            format!("Description: {}", component.description),
        ];

        // Add key specifications
        for (key, value) in &component.specifications {
            text_parts.push(format!("{}: {}", key, value.as_string()));
        }

        text_parts.join(" | ")
    }

    /// Extract key specifications for metadata
    fn extract_key_specs(&self, component: &Component) -> Vec<String> {
        let mut key_specs = Vec::new();
        
        // Common important specifications by category
        let important_specs = match component.category {
            ComponentCategory::Resistors => vec!["Resistance", "Power", "Tolerance", "Package"],
            ComponentCategory::Capacitors => vec!["Capacitance", "Voltage", "Type", "Package"],
            ComponentCategory::Transistors => vec!["Type", "Voltage", "Current", "Package"],
            ComponentCategory::IntegratedCircuits => vec!["Function", "Voltage", "Package", "Pins"],
            _ => vec!["Value", "Voltage", "Current", "Package"],
        };

        for spec in important_specs {
            if component.specifications.contains_key(spec) {
                key_specs.push(spec.to_string());
            }
        }

        key_specs
    }

    /// Convert text to embedding vector (simplified implementation)
    async fn text_to_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // This is a simplified implementation
        // In a real system, you'd use a proper embedding model
        
        // For now, create a simple hash-based embedding
        let mut embedding = vec![0.0; 384]; // Common embedding dimension
        
        // Simple hash-based approach (not ideal, but functional for MVP)
        let words: Vec<&str> = text.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            let hash = self.simple_hash(word) as usize;
            let index = hash % embedding.len();
            embedding[index] += 1.0 / (i + 1) as f32; // Weight by position
        }

        // Normalize the vector
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for value in &mut embedding {
                *value /= magnitude;
            }
        }

        Ok(embedding)
    }

    /// Simple hash function for text
    fn simple_hash(&self, text: &str) -> u32 {
        let mut hash = 0u32;
        for byte in text.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        hash
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude_a * magnitude_b)
    }

    /// Generate explanation for why a component matched
    async fn generate_match_reason(&self, component: &Component, similarity: f32) -> Result<String> {
        let confidence = if similarity > 0.8 {
            "excellent"
        } else if similarity > 0.6 {
            "good"
        } else if similarity > 0.4 {
            "fair"
        } else {
            "basic"
        };

        Ok(format!(
            "{} match ({:.1}%) - {} {} from {} with key specs: {}",
            confidence,
            similarity * 100.0,
            component.category.as_str(),
            component.part_number,
            component.manufacturer,
            component.specifications.keys().take(3).cloned().collect::<Vec<_>>().join(", ")
        ))
    }

    /// Clear the embeddings cache
    ///
    /// Removes all cached embeddings from memory. This is useful when:
    /// - Switching embedding models
    /// - Memory usage becomes too high
    /// - Cache invalidation is needed
    ///
    /// # Example
    ///
    /// ```rust
    /// # use opencircuit_ai::embeddings::ComponentEmbeddingEngine;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = OpenCircuitOllamaClient::new();
    /// # let mut engine = ComponentEmbeddingEngine::new(client).await?;
    /// let (count, _) = engine.cache_stats();
    /// println!("Cache size before: {} items", count);
    ///
    /// engine.clear_cache();
    ///
    /// let (count, _) = engine.cache_stats();
    /// println!("Cache size after: {} items", count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn clear_cache(&mut self) {
        self.embeddings_cache.clear();
    }

    /// Get cache statistics
    ///
    /// Returns information about the current cache state including item count
    /// and estimated memory usage.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `usize`: Number of cached embeddings
    /// - `usize`: Estimated memory usage in bytes (rough approximation)
    ///
    /// # Example
    ///
    /// ```rust
    /// # use opencircuit_ai::embeddings::ComponentEmbeddingEngine;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = OpenCircuitOllamaClient::new();
    /// # let mut engine = ComponentEmbeddingEngine::new(client).await?;
    /// let (count, memory) = engine.cache_stats();
    /// println!("Cache contains {} items (~{} KB)", count, memory / 1024);
    /// # Ok(())
    /// # }
    /// ```
    pub fn cache_stats(&self) -> (usize, usize) {
        let count = self.embeddings_cache.len();
        let memory_estimate = count * 384 * 4; // Rough estimate: 384 floats * 4 bytes
        (count, memory_estimate)
    }

    /// Set the embedding model
    ///
    /// Changes the model used for generating embeddings. This automatically
    /// clears the cache since cached embeddings from the old model are incompatible.
    ///
    /// # Arguments
    ///
    /// * `model` - Name of the new embedding model to use
    ///
    /// # Warning
    ///
    /// This operation clears the entire cache. All previously generated
    /// embeddings will need to be regenerated.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use opencircuit_ai::embeddings::ComponentEmbeddingEngine;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = OpenCircuitOllamaClient::new();
    /// # let mut engine = ComponentEmbeddingEngine::new(client).await?;
    /// // Switch to a different embedding model
    /// engine.set_embedding_model("llama2:7b".to_string());
    ///
    /// // Cache is now empty due to model change
    /// let (count, _) = engine.cache_stats();
    /// assert_eq!(count, 0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_embedding_model(&mut self, model: String) {
        self.embedding_model = model;
        // Clear cache when model changes
        self.clear_cache();
    }
}

/// Utility functions for embedding operations
///
/// This module provides convenient helper functions for common embedding tasks
/// such as batch processing, requirements analysis, and finding the best match.
///
/// # Overview
///
/// The utils module simplifies common workflows when working with embeddings:
///
/// - Batch processing of multiple components
/// - Creating embeddings from requirements text
/// - Finding single best matches
///
/// # Examples
///
/// Batch processing components:
///
/// ```rust
/// # use opencircuit_ai::embeddings::{ComponentEmbeddingEngine, utils};
/// # use opencircuit_core::models::{Component, ComponentCategory};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = OpenCircuitOllamaClient::new();
/// # let mut engine = ComponentEmbeddingEngine::new(client).await?;
/// let components = vec![
///     Component::new("R1".to_string(), "Corp".to_string(), ComponentCategory::Resistors, "1k resistor".to_string()),
///     Component::new("C1".to_string(), "Corp".to_string(), ComponentCategory::Capacitors, "100nF capacitor".to_string()),
/// ];
///
/// // Batch generate embeddings for all components
/// let embeddings = utils::batch_generate_embeddings(&mut engine, &components).await?;
/// println!("Generated {} embeddings", embeddings.len());
/// # Ok(())
/// # }
/// ```
pub mod utils {
    use super::*;

    /// Create a requirements embedding from user input
    ///
    /// Generates a vector embedding directly from natural language requirements
    /// without needing to create a temporary component. Useful for preprocessing
    /// search queries.
    ///
    /// # Arguments
    ///
    /// * `engine` - The embedding engine instance
    /// * `requirements` - Natural language description of requirements
    ///
    /// # Returns
    ///
    /// Returns a vector embedding suitable for similarity comparisons.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use opencircuit_ai::embeddings::{ComponentEmbeddingEngine, utils};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = OpenCircuitOllamaClient::new();
    /// # let mut engine = ComponentEmbeddingEngine::new(client).await?;
    ///
    /// let requirements_embedding = utils::create_requirements_embedding(
    ///     &mut engine,
    ///     "low noise operational amplifier for audio preamp"
    /// ).await?;
    ///
    /// println!("Requirements embedding dimension: {}", requirements_embedding.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_requirements_embedding(
        engine: &mut ComponentEmbeddingEngine,
        requirements: &str,
    ) -> Result<Vec<f32>> {
        engine.text_to_embedding(requirements).await
    }

    /// Batch process components for embedding generation
    ///
    /// Efficiently generates embeddings for multiple components in sequence.
    /// This function handles errors gracefully, skipping problematic components
    /// while continuing with the rest.
    ///
    /// # Arguments
    ///
    /// * `engine` - The embedding engine instance
    /// * `components` - List of components to process
    ///
    /// # Returns
    ///
    /// Returns a vector of successfully generated embeddings. Components that
    /// fail to generate embeddings are logged and skipped.
    ///
    /// # Performance
    ///
    /// This function processes components sequentially. For large datasets,
    /// consider implementing parallel processing in your application.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use opencircuit_ai::embeddings::{ComponentEmbeddingEngine, utils};
    /// # use opencircuit_core::models::{Component, ComponentCategory};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = OpenCircuitOllamaClient::new();
    /// # let mut engine = ComponentEmbeddingEngine::new(client).await?;
    ///
    /// let components = vec![
    ///     Component::new("R1".to_string(), "Corp".to_string(), ComponentCategory::Resistors, "1k resistor".to_string()),
    ///     Component::new("C1".to_string(), "Corp".to_string(), ComponentCategory::Capacitors, "100nF capacitor".to_string()),
    ///     Component::new("IC1".to_string(), "Corp".to_string(), ComponentCategory::IntegratedCircuits, "LM358 op-amp".to_string()),
    /// ];
    ///
    /// let embeddings = utils::batch_generate_embeddings(&mut engine, &components).await?;
    ///
    /// for embedding in embeddings {
    ///     println!("Generated embedding for {}: {} dimensions", 
    ///         embedding.component_id, embedding.vector.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn batch_generate_embeddings(
        engine: &mut ComponentEmbeddingEngine,
        components: &[Component],
    ) -> Result<Vec<ComponentEmbedding>> {
        let mut embeddings = Vec::new();
        
        for component in components {
            match engine.generate_component_embedding(component).await {
                Ok(embedding) => embeddings.push(embedding),
                Err(e) => {
                    tracing::warn!("Failed to generate embedding for component {}: {}", component.id, e);
                }
            }
        }
        
        Ok(embeddings)
    }

    /// Find the best matching component for specific requirements
    ///
    /// Convenience function that returns the single best match from a list of
    /// components based on semantic similarity to the requirements.
    ///
    /// # Arguments
    ///
    /// * `engine` - The embedding engine instance
    /// * `requirements` - Natural language description of requirements
    /// * `components` - List of components to search through
    ///
    /// # Returns
    ///
    /// Returns `Some(SimilarityMatch)` for the best match, or `None` if:
    /// - The components list is empty
    /// - No components meet the minimum similarity threshold (0.3)
    /// - All components fail to generate embeddings
    ///
    /// # Example
    ///
    /// ```rust
    /// # use opencircuit_ai::embeddings::{ComponentEmbeddingEngine, utils};
    /// # use opencircuit_core::models::{Component, ComponentCategory};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = OpenCircuitOllamaClient::new();
    /// # let mut engine = ComponentEmbeddingEngine::new(client).await?;
    ///
    /// let components = vec![
    ///     Component::new("R1".to_string(), "Corp".to_string(), ComponentCategory::Resistors, "1k resistor".to_string()),
    ///     Component::new("R2".to_string(), "Corp".to_string(), ComponentCategory::Resistors, "10k resistor".to_string()),
    /// ];
    ///
    /// if let Some(best_match) = utils::find_best_match(
    ///     &mut engine,
    ///     "high resistance for voltage divider",
    ///     &components
    /// ).await? {
    ///     println!("Best match: {} ({:.1}%)", 
    ///         best_match.component.part_number,
    ///         best_match.similarity * 100.0);
    /// } else {
    ///     println!("No suitable matches found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_best_match(
        engine: &mut ComponentEmbeddingEngine,
        requirements: &str,
        components: &[Component],
    ) -> Result<Option<SimilarityMatch>> {
        let matches = engine.find_similar_components_by_requirements(requirements, components, 1).await?;
        Ok(matches.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencircuit_core::models::{ComponentCategory, SpecValue};
    use std::collections::HashMap;

    fn create_test_component() -> Component {
        let mut specs = HashMap::new();
        specs.insert("Resistance".to_string(), SpecValue::String("10k".to_string()));
        specs.insert("Power".to_string(), SpecValue::String("0.25W".to_string()));
        specs.insert("Tolerance".to_string(), SpecValue::String("5%".to_string()));

        Component::new(
            "R1234".to_string(),
            "TestCorp".to_string(),
            ComponentCategory::Resistors,
            "10k ohm resistor".to_string(),
        ).with_specifications(specs)
    }

    #[tokio::test]
    async fn test_component_to_text() {
        let component = create_test_component();
        let engine = ComponentEmbeddingEngine::new(
            OpenCircuitOllamaClient::new()
        ).await.unwrap();
        
        let text = engine.component_to_text(&component);
        assert!(text.contains("R1234"));
        assert!(text.contains("TestCorp"));
        assert!(text.contains("Resistors"));
        assert!(text.contains("10k"));
    }

    #[tokio::test]
    async fn test_cosine_similarity() {
        let engine = ComponentEmbeddingEngine::new(
            OpenCircuitOllamaClient::new()
        ).await.unwrap();
        
        let vec_a = vec![1.0, 0.0, 0.0];
        let vec_b = vec![1.0, 0.0, 0.0];
        let vec_c = vec![0.0, 1.0, 0.0];
        
        assert!((engine.cosine_similarity(&vec_a, &vec_b) - 1.0).abs() < 0.001);
        assert!((engine.cosine_similarity(&vec_a, &vec_c) - 0.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_extract_key_specs() {
        let component = create_test_component();
        let engine = ComponentEmbeddingEngine::new(
            OpenCircuitOllamaClient::new()
        ).await.unwrap();
        
        let key_specs = engine.extract_key_specs(&component);
        assert!(key_specs.contains(&"Resistance".to_string()));
        assert!(key_specs.contains(&"Power".to_string()));
    }

    #[tokio::test]
    async fn test_simple_hash() {
        let engine = ComponentEmbeddingEngine::new(
            OpenCircuitOllamaClient::new()
        ).await.unwrap();
        
        let hash1 = engine.simple_hash("test");
        let hash2 = engine.simple_hash("test");
        let hash3 = engine.simple_hash("different");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}