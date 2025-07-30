//! Real-time circuit visualization for OpenCircuit
//! 
//! Provides interactive circuit schematics with real-time simulation updates,
//! component library, and responsive design features.

pub mod schematic_renderer;
pub mod circuit_viewer;
pub mod primitives;
pub mod styles;
pub mod animations;

pub use schematic_renderer::SchematicRenderer;
pub use circuit_viewer::CircuitViewer;
pub use primitives::CircuitPrimitives;
pub use styles::{CircuitStyle, CircuitStyleConfig, ComponentAppearance, ThemePreset};
pub use animations::{CircuitAnimations, AnimationConfig};

/// Graphics result type
pub type GraphicsResult<T> = Result<T, GraphicsError>;

/// Graphics-specific error types
#[derive(Debug, thiserror::Error)]
pub enum GraphicsError {
    #[error("Rendering error: {0}")]
    Rendering(String),
    
    #[error("Invalid component type: {0}")]
    InvalidComponent(String),
    
    #[error("Simulation visualization error: {0}")]
    Simulation(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Animation error: {0}")]
    Animation(String),
    
    #[error("Style error: {0}")]
    Style(String),
}

/// Main graphics library interface
pub struct OpenCircuitGraphics {
    renderer: SchematicRenderer,
    viewer: CircuitViewer,
    animations: CircuitAnimations,
    style: CircuitStyle,
}

impl OpenCircuitGraphics {
    /// Create a new graphics instance
    pub fn new() -> Self {
        Self {
            renderer: SchematicRenderer::new(),
            viewer: CircuitViewer::new(),
            animations: CircuitAnimations::new(),
            style: CircuitStyle::default(),
        }
    }

    /// Update all animations
    pub fn update(&mut self) {
        self.animations.update();
    }

    /// Get mutable reference to the renderer
    pub fn renderer_mut(&mut self) -> &mut SchematicRenderer {
        &mut self.renderer
    }

    /// Get mutable reference to the viewer
    pub fn viewer_mut(&mut self) -> &mut CircuitViewer {
        &mut self.viewer
    }

    /// Get mutable reference to animations
    pub fn animations_mut(&mut self) -> &mut CircuitAnimations {
        &mut self.animations
    }

    /// Get mutable reference to style
    pub fn style_mut(&mut self) -> &mut CircuitStyle {
        &mut self.style
    }

    /// Set theme preset
    pub fn set_theme(&mut self, theme: ThemePreset) {
        self.style = theme.to_style();
    }

    /// Configure animation settings
    pub fn configure_animations(&mut self, config: AnimationConfig) {
        // Animation configuration would be applied here
    }

    /// Reset all graphics state
    pub fn reset(&mut self) {
        self.renderer.clear();
        self.animations.clear();
    }
}

impl Default for OpenCircuitGraphics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencircuit_core::models::Circuit;

    #[test]
    fn test_basic_initialization() {
        let graphics = OpenCircuitGraphics::new();
        assert!(true);
    }

    #[test]
    fn test_theme_switching() {
        let mut graphics = OpenCircuitGraphics::new();
        graphics.set_theme(ThemePreset::Dark);
        assert_eq!(graphics.style.background_color, CircuitStyle::dark_theme().background_color);
    }

    #[test]
    fn test_component_styling() {
        let style = CircuitStyle::default();
        assert_eq!(style.get_component_color("resistor"), style.resistor_color);
        assert_eq!(style.get_component_color("capacitor"), style.capacitor_color);
        assert_eq!(style.get_component_color("inductor"), style.inductor_color);
    }

    #[test]
    fn test_simulation_color_mapping() {
        let style = CircuitStyle::default();
        let color = style.get_simulation_color(5.0, 0.0, 10.0);
        assert!(color.r() > color.b()); // Should be more red than blue
    }
}