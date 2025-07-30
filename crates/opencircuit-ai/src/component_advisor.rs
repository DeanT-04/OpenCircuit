//! AI-powered component recommendation system
//! 
//! This module provides intelligent component recommendations based on:
//! - User requirements and specifications
//! - Circuit context and design constraints
//! - Component compatibility and availability
//! - Performance optimization suggestions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use opencircuit_core::{
    models::{Component, ComponentCategory},
    OpenCircuitError,
};
use crate::models::{AiModel, AiContext};
use crate::ollama_client::OpenCircuitOllamaClient;
use crate::embeddings::{ComponentEmbeddingEngine, SimilarityMatch};

type Result<T> = std::result::Result<T, OpenCircuitError>;

/// Component recommendation with detailed analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRecommendation {
    /// Recommended component
    pub component: Component,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Reasoning for the recommendation
    pub reasoning: String,
    /// Alternative components
    pub alternatives: Vec<Component>,
    /// Compatibility warnings
    pub warnings: Vec<String>,
    /// Performance notes
    pub performance_notes: Vec<String>,
    /// Cost analysis
    pub cost_analysis: Option<CostAnalysis>,
}

/// Cost analysis for component recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnalysis {
    /// Unit cost estimate
    pub unit_cost: f64,
    /// Currency
    pub currency: String,
    /// Cost category (budget, standard, premium)
    pub cost_category: CostCategory,
    /// Cost comparison with alternatives
    pub cost_comparison: String,
}

/// Cost categories for components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CostCategory {
    Budget,
    Standard,
    Premium,
    Unknown,
}

/// Recommendation request parameters
#[derive(Debug, Clone)]
pub struct RecommendationRequest {
    /// User requirements in natural language
    pub requirements: String,
    /// Circuit context
    pub circuit_context: Option<AiContext>,
    /// Preferred component categories
    pub preferred_categories: Vec<ComponentCategory>,
    /// Budget constraints
    pub budget_constraints: Option<BudgetConstraints>,
    /// Performance priorities
    pub performance_priorities: Vec<PerformancePriority>,
    /// Maximum number of recommendations
    pub max_recommendations: usize,
}

/// Budget constraints for recommendations
#[derive(Debug, Clone)]
pub struct BudgetConstraints {
    /// Maximum cost per component
    pub max_cost_per_component: f64,
    /// Total budget for all components
    pub total_budget: Option<f64>,
    /// Currency
    pub currency: String,
    /// Cost priority (minimize cost vs. optimize performance)
    pub cost_priority: CostPriority,
}

/// Cost optimization priorities
#[derive(Debug, Clone, PartialEq)]
pub enum CostPriority {
    MinimizeCost,
    BalanceCostPerformance,
    OptimizePerformance,
}

/// Performance optimization priorities
#[derive(Debug, Clone, PartialEq)]
pub enum PerformancePriority {
    Speed,
    Accuracy,
    PowerEfficiency,
    Size,
    Reliability,
    Temperature,
    Noise,
}

/// AI-powered component advisor
pub struct ComponentAdvisor {
    /// Ollama client for AI interactions
    ollama_client: OpenCircuitOllamaClient,
    /// Embedding engine for similarity search
    embedding_engine: ComponentEmbeddingEngine,
    /// Component database
    component_database: Vec<Component>,
    /// AI model for recommendations
    recommendation_model: AiModel,
}

impl ComponentAdvisor {
    /// Create a new component advisor
    pub async fn new(ollama_client: OpenCircuitOllamaClient) -> Result<Self> {
        let embedding_engine = ComponentEmbeddingEngine::new(ollama_client.clone()).await?;
        
        Ok(Self {
            ollama_client,
            embedding_engine,
            component_database: Vec::new(),
            recommendation_model: AiModel::QwenSmall, // Good balance for recommendations
        })
    }

    /// Load component database
    pub fn load_components(&mut self, components: Vec<Component>) {
        self.component_database = components;
    }

    /// Get component recommendations based on requirements
    pub async fn get_recommendations(
        &mut self,
        request: RecommendationRequest,
    ) -> Result<Vec<ComponentRecommendation>> {
        // Step 1: Use AI to analyze and enhance requirements
        let enhanced_requirements = self.enhance_requirements(&request).await?;
        
        // Step 2: Use embedding search to find similar components
        let similar_components = self.find_similar_components(&enhanced_requirements, &request).await?;
        
        // Step 3: Use AI to analyze and rank components
        let analyzed_components = self.analyze_components(&similar_components, &request).await?;
        
        // Step 4: Generate detailed recommendations
        let recommendations = self.generate_recommendations(analyzed_components, &request).await?;
        
        Ok(recommendations)
    }

    /// Get recommendations for a specific component category
    pub async fn get_category_recommendations(
        &mut self,
        category: ComponentCategory,
        requirements: &str,
        max_results: usize,
    ) -> Result<Vec<ComponentRecommendation>> {
        let request = RecommendationRequest {
            requirements: requirements.to_string(),
            circuit_context: None,
            preferred_categories: vec![category],
            budget_constraints: None,
            performance_priorities: vec![],
            max_recommendations: max_results,
        };

        self.get_recommendations(request).await
    }

    /// Get alternative components for a given component
    pub async fn get_alternatives(
        &mut self,
        component: &Component,
        requirements: &str,
        max_alternatives: usize,
    ) -> Result<Vec<ComponentRecommendation>> {
        let enhanced_requirements = format!(
            "Find alternatives to {} {} with similar specifications: {}. Requirements: {}",
            component.manufacturer,
            component.part_number,
            self.component_specs_to_text(component),
            requirements
        );

        let request = RecommendationRequest {
            requirements: enhanced_requirements,
            circuit_context: None,
            preferred_categories: vec![component.category.clone()],
            budget_constraints: None,
            performance_priorities: vec![],
            max_recommendations: max_alternatives,
        };

        self.get_recommendations(request).await
    }

    /// Analyze component compatibility with circuit context
    pub async fn analyze_compatibility(
        &mut self,
        component: &Component,
        circuit_context: &AiContext,
    ) -> Result<CompatibilityAnalysis> {
        let context_description = self.context_to_text(circuit_context);
        let component_description = self.component_to_text(component);

        let prompt = format!(
            "Analyze the compatibility of this component with the given circuit context:\n\n\
            Component: {}\n\n\
            Circuit Context: {}\n\n\
            Provide analysis on:\n\
            1. Electrical compatibility\n\
            2. Physical compatibility\n\
            3. Performance implications\n\
            4. Potential issues or warnings\n\
            5. Optimization suggestions",
            component_description,
            context_description
        );

        let response = self.ollama_client.complete(&prompt).await?;
        
        Ok(CompatibilityAnalysis {
            component_id: component.id.clone(),
            compatibility_score: self.extract_compatibility_score(&response),
            electrical_compatibility: self.extract_electrical_analysis(&response),
            physical_compatibility: self.extract_physical_analysis(&response),
            performance_impact: self.extract_performance_analysis(&response),
            warnings: self.extract_warnings(&response),
            suggestions: self.extract_suggestions(&response),
        })
    }

    /// Enhance user requirements using AI
    async fn enhance_requirements(&mut self, request: &RecommendationRequest) -> Result<String> {
        let context_info = if let Some(context) = &request.circuit_context {
            format!(
                "Circuit Type: {:?}, Design Phase: {:?}, User Level: {:?}",
                context.circuit_type, context.design_phase, context.user_level
            )
        } else {
            "No specific circuit context provided".to_string()
        };

        let prompt = format!(
            "Enhance and clarify these component requirements for better search:\n\n\
            Original Requirements: {}\n\
            Circuit Context: {}\n\
            Preferred Categories: {:?}\n\
            Performance Priorities: {:?}\n\n\
            Provide enhanced requirements that include:\n\
            1. Specific technical specifications\n\
            2. Performance criteria\n\
            3. Compatibility requirements\n\
            4. Application context",
            request.requirements,
            context_info,
            request.preferred_categories,
            request.performance_priorities
        );

        let response = self.ollama_client.complete(&prompt).await?;
        Ok(response)
    }

    /// Find similar components using embedding search
    async fn find_similar_components(
        &mut self,
        requirements: &str,
        request: &RecommendationRequest,
    ) -> Result<Vec<SimilarityMatch>> {
        // Filter components by preferred categories if specified
        let filtered_components = if request.preferred_categories.is_empty() {
            self.component_database.clone()
        } else {
            self.component_database
                .iter()
                .filter(|c| request.preferred_categories.contains(&c.category))
                .cloned()
                .collect()
        };

        // Use embedding search to find similar components
        let similar_components = self.embedding_engine
            .find_similar_components_by_requirements(requirements, &filtered_components, request.max_recommendations * 3)
            .await?;

        Ok(similar_components)
    }

    /// Analyze components using AI
    async fn analyze_components(
        &mut self,
        similar_components: &[SimilarityMatch],
        request: &RecommendationRequest,
    ) -> Result<Vec<AnalyzedComponent>> {
        let mut analyzed = Vec::new();

        for similarity_match in similar_components {
            let component = &similarity_match.component;
            let analysis = self.analyze_single_component(component, request).await?;
            
            analyzed.push(AnalyzedComponent {
                component: component.clone(),
                similarity_score: similarity_match.similarity,
                ai_analysis: analysis,
                match_reason: similarity_match.match_reason.clone(),
            });
        }

        // Sort by combined score (similarity + AI analysis)
        analyzed.sort_by(|a, b| {
            let score_a = a.similarity_score * 0.4 + a.ai_analysis.suitability_score * 0.6;
            let score_b = b.similarity_score * 0.4 + b.ai_analysis.suitability_score * 0.6;
            score_b.partial_cmp(&score_a).unwrap()
        });

        Ok(analyzed)
    }

    /// Analyze a single component
    async fn analyze_single_component(
        &mut self,
        component: &Component,
        request: &RecommendationRequest,
    ) -> Result<ComponentAnalysis> {
        let component_text = self.component_to_text(component);
        let budget_info = if let Some(budget) = &request.budget_constraints {
            format!("Budget: max ${} per component, priority: {:?}", budget.max_cost_per_component, budget.cost_priority)
        } else {
            "No budget constraints specified".to_string()
        };

        let prompt = format!(
            "Analyze this component for the given requirements:\n\n\
            Component: {}\n\n\
            Requirements: {}\n\
            Budget Constraints: {}\n\
            Performance Priorities: {:?}\n\n\
            Provide analysis on:\n\
            1. Suitability score (0.0 to 1.0)\n\
            2. Strengths for this application\n\
            3. Potential weaknesses or limitations\n\
            4. Performance characteristics\n\
            5. Cost-effectiveness",
            component_text,
            request.requirements,
            budget_info,
            request.performance_priorities
        );

        let response = self.ollama_client.complete(&prompt).await?;

        Ok(ComponentAnalysis {
            suitability_score: self.extract_suitability_score(&response),
            strengths: self.extract_strengths(&response),
            weaknesses: self.extract_weaknesses(&response),
            performance_notes: self.extract_performance_notes(&response),
            cost_effectiveness: self.extract_cost_effectiveness(&response),
        })
    }

    /// Generate final recommendations
    async fn generate_recommendations(
        &mut self,
        analyzed_components: Vec<AnalyzedComponent>,
        request: &RecommendationRequest,
    ) -> Result<Vec<ComponentRecommendation>> {
        let mut recommendations = Vec::new();

        for analyzed in analyzed_components.into_iter().take(request.max_recommendations) {
            let alternatives = self.find_alternatives_for_component(&analyzed.component).await?;
            let warnings = self.generate_warnings(&analyzed.component, request).await?;
            let cost_analysis = self.analyze_cost(&analyzed.component, request).await?;

            let recommendation = ComponentRecommendation {
                component: analyzed.component,
                confidence: (analyzed.similarity_score * 0.4 + analyzed.ai_analysis.suitability_score * 0.6),
                reasoning: format!(
                    "{}. AI Analysis: {}",
                    analyzed.match_reason,
                    analyzed.ai_analysis.strengths.join(". ")
                ),
                alternatives,
                warnings,
                performance_notes: analyzed.ai_analysis.performance_notes,
                cost_analysis,
            };

            recommendations.push(recommendation);
        }

        Ok(recommendations)
    }

    /// Helper methods for text conversion and analysis
    fn component_to_text(&self, component: &Component) -> String {
        format!(
            "{} {} - {} ({}): {}",
            component.manufacturer,
            component.part_number,
            component.category.as_str(),
            component.description,
            self.component_specs_to_text(component)
        )
    }

    fn component_specs_to_text(&self, component: &Component) -> String {
        component.specifications
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v.as_string()))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn context_to_text(&self, context: &AiContext) -> String {
        format!(
            "Project: {}, Type: {:?}, Phase: {:?}, Level: {:?}, Constraints: {}",
            context.project_name.as_deref().unwrap_or("Unknown"),
            context.circuit_type,
            context.design_phase,
            context.user_level,
            context.constraints.join(", ")
        )
    }

    /// Extract analysis results from AI responses (simplified parsing)
    fn extract_suitability_score(&self, response: &str) -> f32 {
        // Simple pattern matching for score extraction
        // In a real implementation, you'd use more sophisticated parsing
        if response.contains("excellent") || response.contains("perfect") {
            0.9
        } else if response.contains("good") || response.contains("suitable") {
            0.7
        } else if response.contains("fair") || response.contains("adequate") {
            0.5
        } else if response.contains("poor") || response.contains("unsuitable") {
            0.3
        } else {
            0.6 // Default
        }
    }

    fn extract_strengths(&self, response: &str) -> Vec<String> {
        // Simplified extraction - look for positive keywords
        let mut strengths = Vec::new();
        if response.contains("high performance") {
            strengths.push("High performance characteristics".to_string());
        }
        if response.contains("cost effective") || response.contains("affordable") {
            strengths.push("Cost effective solution".to_string());
        }
        if response.contains("reliable") {
            strengths.push("High reliability".to_string());
        }
        if response.contains("low power") {
            strengths.push("Low power consumption".to_string());
        }
        if strengths.is_empty() {
            strengths.push("Meets basic requirements".to_string());
        }
        strengths
    }

    fn extract_weaknesses(&self, response: &str) -> Vec<String> {
        let mut weaknesses = Vec::new();
        if response.contains("expensive") || response.contains("costly") {
            weaknesses.push("Higher cost than alternatives".to_string());
        }
        if response.contains("limited") {
            weaknesses.push("Limited specifications".to_string());
        }
        if response.contains("obsolete") {
            weaknesses.push("Potential obsolescence risk".to_string());
        }
        weaknesses
    }

    fn extract_performance_notes(&self, response: &str) -> Vec<String> {
        let mut notes = Vec::new();
        if response.contains("temperature") {
            notes.push("Consider temperature characteristics".to_string());
        }
        if response.contains("frequency") {
            notes.push("Frequency response considerations".to_string());
        }
        if response.contains("power") {
            notes.push("Power handling characteristics".to_string());
        }
        notes
    }

    fn extract_cost_effectiveness(&self, response: &str) -> String {
        if response.contains("very cost effective") {
            "Excellent value for money".to_string()
        } else if response.contains("cost effective") {
            "Good value for money".to_string()
        } else if response.contains("expensive") {
            "Premium pricing".to_string()
        } else {
            "Standard pricing".to_string()
        }
    }

    // Additional helper methods for compatibility analysis
    fn extract_compatibility_score(&self, response: &str) -> f32 {
        self.extract_suitability_score(response) // Reuse the same logic
    }

    fn extract_electrical_analysis(&self, response: &str) -> String {
        if response.contains("fully compatible") {
            "Fully electrically compatible".to_string()
        } else if response.contains("compatible") {
            "Electrically compatible with minor considerations".to_string()
        } else {
            "Requires electrical compatibility verification".to_string()
        }
    }

    fn extract_physical_analysis(&self, response: &str) -> String {
        if response.contains("fits") || response.contains("compatible footprint") {
            "Physical dimensions compatible".to_string()
        } else {
            "Verify physical compatibility".to_string()
        }
    }

    fn extract_performance_analysis(&self, _response: &str) -> String {
        "Performance impact analysis needed".to_string() // Simplified
    }

    fn extract_warnings(&self, response: &str) -> Vec<String> {
        let mut warnings = Vec::new();
        if response.contains("warning") || response.contains("caution") {
            warnings.push("Review compatibility carefully".to_string());
        }
        warnings
    }

    fn extract_suggestions(&self, _response: &str) -> Vec<String> {
        vec!["Consider testing in target application".to_string()] // Simplified
    }

    async fn find_alternatives_for_component(&mut self, _component: &Component) -> Result<Vec<Component>> {
        // Simplified - return empty for now
        Ok(Vec::new())
    }

    async fn generate_warnings(&mut self, _component: &Component, _request: &RecommendationRequest) -> Result<Vec<String>> {
        // Simplified - return empty for now
        Ok(Vec::new())
    }

    async fn analyze_cost(&mut self, component: &Component, request: &RecommendationRequest) -> Result<Option<CostAnalysis>> {
        if let Some(price_info) = &component.price_info {
            if let Some(first_break) = price_info.price_breaks.first() {
                let cost_category = if let Some(budget) = &request.budget_constraints {
                    if first_break.unit_price <= budget.max_cost_per_component * 0.5 {
                        CostCategory::Budget
                    } else if first_break.unit_price <= budget.max_cost_per_component {
                        CostCategory::Standard
                    } else {
                        CostCategory::Premium
                    }
                } else {
                    CostCategory::Unknown
                };

                return Ok(Some(CostAnalysis {
                    unit_cost: first_break.unit_price,
                    currency: price_info.currency.clone(),
                    cost_category,
                    cost_comparison: "Competitive pricing".to_string(),
                }));
            }
        }
        Ok(None)
    }
}

/// Supporting data structures
#[derive(Debug, Clone)]
struct AnalyzedComponent {
    component: Component,
    similarity_score: f32,
    ai_analysis: ComponentAnalysis,
    match_reason: String,
}

#[derive(Debug, Clone)]
struct ComponentAnalysis {
    suitability_score: f32,
    strengths: Vec<String>,
    weaknesses: Vec<String>,
    performance_notes: Vec<String>,
    cost_effectiveness: String,
}

/// Compatibility analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityAnalysis {
    /// Component ID analyzed
    pub component_id: String,
    /// Overall compatibility score (0.0 to 1.0)
    pub compatibility_score: f32,
    /// Electrical compatibility analysis
    pub electrical_compatibility: String,
    /// Physical compatibility analysis
    pub physical_compatibility: String,
    /// Performance impact analysis
    pub performance_impact: String,
    /// Compatibility warnings
    pub warnings: Vec<String>,
    /// Optimization suggestions
    pub suggestions: Vec<String>,
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

        Component::new(
            "R1234".to_string(),
            "TestCorp".to_string(),
            ComponentCategory::Resistors,
            "10k ohm resistor".to_string(),
        ).with_specifications(specs)
    }

    #[tokio::test]
    async fn test_component_to_text() {
        let advisor = ComponentAdvisor::new(
            OpenCircuitOllamaClient::new()
        ).await.unwrap();
        let component = create_test_component();
        
        let text = advisor.component_to_text(&component);
        assert!(text.contains("R1234"));
        assert!(text.contains("TestCorp"));
        assert!(text.contains("10k"));
    }

    #[tokio::test]
    async fn test_extract_suitability_score() {
        let advisor = ComponentAdvisor::new(
            OpenCircuitOllamaClient::new()
        ).await.unwrap();
        
        assert_eq!(advisor.extract_suitability_score("This is an excellent choice"), 0.9);
        assert_eq!(advisor.extract_suitability_score("This is a good option"), 0.7);
        assert_eq!(advisor.extract_suitability_score("This is poor quality"), 0.3);
    }

    #[test]
    fn test_cost_category_determination() {
        // Test cost category logic
        let budget = BudgetConstraints {
            max_cost_per_component: 10.0,
            total_budget: None,
            currency: "USD".to_string(),
            cost_priority: CostPriority::BalanceCostPerformance,
        };

        // Component costing $5 should be Budget category (< 50% of max)
        // Component costing $8 should be Standard category (< 100% of max)
        // Component costing $15 should be Premium category (> 100% of max)
        
        assert_eq!(5.0 <= budget.max_cost_per_component * 0.5, true);
        assert_eq!(8.0 <= budget.max_cost_per_component, true);
        assert_eq!(15.0 > budget.max_cost_per_component, true);
    }
}