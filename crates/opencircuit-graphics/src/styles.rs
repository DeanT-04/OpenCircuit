//! Visual styling system for circuit graphics
//! 
//! Defines colors, fonts, and visual appearance for circuit components,
//! wires, and UI elements used in the schematic renderer.

use egui::{Color32, FontId, Stroke, Style, Visuals};

/// Circuit styling configuration
#[derive(Debug, Clone, Copy)]
pub struct CircuitStyle {
    /// Color for resistors
    pub resistor_color: Color32,
    /// Color for capacitors
    pub capacitor_color: Color32,
    /// Color for inductors
    pub inductor_color: Color32,
    /// Color for voltage sources
    pub voltage_source_color: Color32,
    /// Color for current sources
    pub current_source_color: Color32,
    /// Color for ground symbols
    pub ground_color: Color32,
    /// Color for wires
    pub wire_color: Color32,
    /// Color for selected components
    pub selection_color: Color32,
    /// Color for highlighted components
    pub highlight_color: Color32,
    /// Color for the grid
    pub grid_color: Color32,
    /// Color for text labels
    pub text_color: Color32,
    /// Color for background
    pub background_color: Color32,
    /// Color for junction points
    pub junction_color: Color32,
    /// Color for measurement probes
    pub probe_color: Color32,
    /// Default font size
    pub font_size: f32,
    /// Wire thickness
    pub wire_thickness: f32,
    /// Grid spacing
    pub grid_spacing: f32,
    /// Selection handle size
    pub handle_size: f32,
    /// Animation speed for simulation
    pub animation_speed: f32,
}

impl Default for CircuitStyle {
    fn default() -> Self {
        Self {
            resistor_color: Color32::from_rgb(100, 100, 100),
            capacitor_color: Color32::from_rgb(200, 100, 100),
            inductor_color: Color32::from_rgb(100, 200, 100),
            voltage_source_color: Color32::from_rgb(100, 100, 200),
            current_source_color: Color32::from_rgb(200, 100, 200),
            ground_color: Color32::from_rgb(150, 150, 150),
            wire_color: Color32::from_rgb(50, 50, 50),
            selection_color: Color32::from_rgb(255, 165, 0),
            highlight_color: Color32::from_rgb(255, 255, 0),
            grid_color: Color32::from_rgb(220, 220, 220),
            text_color: Color32::from_rgb(0, 0, 0),
            background_color: Color32::from_rgb(250, 250, 250),
            junction_color: Color32::from_rgb(0, 0, 0),
            probe_color: Color32::from_rgb(255, 0, 0),
            font_size: 12.0,
            wire_thickness: 2.0,
            grid_spacing: 20.0,
            handle_size: 4.0,
            animation_speed: 1.0,
        }
    }
}

impl CircuitStyle {
    /// Create a dark theme style
    pub fn dark_theme() -> Self {
        Self {
            resistor_color: Color32::from_rgb(200, 200, 200),
            capacitor_color: Color32::from_rgb(255, 150, 150),
            inductor_color: Color32::from_rgb(150, 255, 150),
            voltage_source_color: Color32::from_rgb(150, 150, 255),
            current_source_color: Color32::from_rgb(255, 150, 255),
            ground_color: Color32::from_rgb(200, 200, 200),
            wire_color: Color32::from_rgb(200, 200, 200),
            selection_color: Color32::from_rgb(255, 165, 0),
            highlight_color: Color32::from_rgb(255, 255, 0),
            grid_color: Color32::from_rgb(80, 80, 80),
            text_color: Color32::from_rgb(255, 255, 255),
            background_color: Color32::from_rgb(30, 30, 30),
            junction_color: Color32::from_rgb(255, 255, 255),
            probe_color: Color32::from_rgb(255, 100, 100),
            ..Self::default()
        }
    }

    /// Create a high contrast theme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            resistor_color: Color32::from_rgb(255, 255, 255),
            capacitor_color: Color32::from_rgb(255, 255, 255),
            inductor_color: Color32::from_rgb(255, 255, 255),
            voltage_source_color: Color32::from_rgb(255, 255, 255),
            current_source_color: Color32::from_rgb(255, 255, 255),
            ground_color: Color32::from_rgb(255, 255, 255),
            wire_color: Color32::from_rgb(255, 255, 255),
            selection_color: Color32::from_rgb(255, 255, 0),
            highlight_color: Color32::from_rgb(0, 255, 0),
            grid_color: Color32::from_rgb(128, 128, 128),
            text_color: Color32::from_rgb(255, 255, 255),
            background_color: Color32::from_rgb(0, 0, 0),
            junction_color: Color32::from_rgb(255, 255, 255),
            probe_color: Color32::from_rgb(0, 255, 255),
            wire_thickness: 3.0,
            ..Self::default()
        }
    }

    /// Create a colorblind-friendly theme
    pub fn colorblind_friendly() -> Self {
        Self {
            resistor_color: Color32::from_rgb(0, 0, 0),
            capacitor_color: Color32::from_rgb(0, 114, 178),
            inductor_color: Color32::from_rgb(230, 159, 0),
            voltage_source_color: Color32::from_rgb(86, 180, 233),
            current_source_color: Color32::from_rgb(204, 121, 167),
            ground_color: Color32::from_rgb(153, 153, 153),
            wire_color: Color32::from_rgb(0, 0, 0),
            selection_color: Color32::from_rgb(255, 0, 0),
            highlight_color: Color32::from_rgb(0, 158, 115),
            grid_color: Color32::from_rgb(200, 200, 200),
            text_color: Color32::from_rgb(0, 0, 0),
            background_color: Color32::from_rgb(255, 255, 255),
            junction_color: Color32::from_rgb(0, 0, 0),
            probe_color: Color32::from_rgb(213, 94, 0),
            ..Self::default()
        }
    }

    /// Get font ID for labels
    pub fn font_id(&self) -> FontId {
        FontId::proportional(self.font_size)
    }

    /// Get stroke for wires
    pub fn wire_stroke(&self) -> Stroke {
        Stroke::new(self.wire_thickness, self.wire_color)
    }

    /// Get stroke for selection
    pub fn selection_stroke(&self) -> Stroke {
        Stroke::new(2.0, self.selection_color)
    }

    /// Get stroke for grid lines
    pub fn grid_stroke(&self) -> Stroke {
        Stroke::new(1.0, self.grid_color)
    }

    /// Get stroke for junction points
    pub fn junction_stroke(&self) -> Stroke {
        Stroke::new(2.0, self.junction_color)
    }

    /// Create egui style from circuit style
    pub fn to_egui_style(&self) -> Style {
        let mut style = Style::default();
        style.visuals = Visuals::light();
        style.visuals.extreme_bg_color = self.background_color;
        style.visuals.text_color = self.text_color;
        style
    }

    /// Create dark egui style
    pub fn to_dark_egui_style(&self) -> Style {
        let mut style = Style::default();
        style.visuals = Visuals::dark();
        style.visuals.extreme_bg_color = self.background_color;
        style.visuals.text_color = self.text_color;
        style
    }

    /// Get component color by type
    pub fn get_component_color(&self, component_type: &str) -> Color32 {
        match component_type.to_lowercase().as_str() {
            "resistor" => self.resistor_color,
            "capacitor" => self.capacitor_color,
            "inductor" => self.inductor_color,
            "voltage_source" | "vsource" => self.voltage_source_color,
            "current_source" | "csource" => self.current_source_color,
            "ground" => self.ground_color,
            _ => self.wire_color,
        }
    }

    /// Get simulation color based on value
    pub fn get_simulation_color(&self, value: f64, min: f64, max: f64) -> Color32 {
        let normalized = ((value - min) / (max - min)).clamp(0.0, 1.0);
        
        // Red for high values, green for medium, blue for low
        let r = (normalized * 255.0) as u8;
        let g = ((1.0 - (normalized - 0.5).abs() * 2.0) * 255.0) as u8;
        let b = ((1.0 - normalized) * 255.0) as u8;
        
        Color32::from_rgb(r, g, b)
    }

    /// Update style from configuration
    pub fn update_from_config(&mut self, config: &CircuitStyleConfig) {
        self.resistor_color = config.resistor_color.unwrap_or(self.resistor_color);
        self.capacitor_color = config.capacitor_color.unwrap_or(self.capacitor_color);
        self.inductor_color = config.inductor_color.unwrap_or(self.inductor_color);
        self.voltage_source_color = config.voltage_source_color.unwrap_or(self.voltage_source_color);
        self.current_source_color = config.current_source_color.unwrap_or(self.current_source_color);
        self.ground_color = config.ground_color.unwrap_or(self.ground_color);
        self.wire_color = config.wire_color.unwrap_or(self.wire_color);
        self.selection_color = config.selection_color.unwrap_or(self.selection_color);
        self.highlight_color = config.highlight_color.unwrap_or(self.highlight_color);
        self.grid_color = config.grid_color.unwrap_or(self.grid_color);
        self.text_color = config.text_color.unwrap_or(self.text_color);
        self.background_color = config.background_color.unwrap_or(self.background_color);
        self.junction_color = config.junction_color.unwrap_or(self.junction_color);
        self.probe_color = config.probe_color.unwrap_or(self.probe_color);
        self.font_size = config.font_size.unwrap_or(self.font_size);
        self.wire_thickness = config.wire_thickness.unwrap_or(self.wire_thickness);
        self.grid_spacing = config.grid_spacing.unwrap_or(self.grid_spacing);
        self.handle_size = config.handle_size.unwrap_or(self.handle_size);
        self.animation_speed = config.animation_speed.unwrap_or(self.animation_speed);
    }
}

/// Configuration structure for updating circuit style
#[derive(Debug, Clone, Default)]
pub struct CircuitStyleConfig {
    pub resistor_color: Option<Color32>,
    pub capacitor_color: Option<Color32>,
    pub inductor_color: Option<Color32>,
    pub voltage_source_color: Option<Color32>,
    pub current_source_color: Option<Color32>,
    pub ground_color: Option<Color32>,
    pub wire_color: Option<Color32>,
    pub selection_color: Option<Color32>,
    pub highlight_color: Option<Color32>,
    pub grid_color: Option<Color32>,
    pub text_color: Option<Color32>,
    pub background_color: Option<Color32>,
    pub junction_color: Option<Color32>,
    pub probe_color: Option<Color32>,
    pub font_size: Option<f32>,
    pub wire_thickness: Option<f32>,
    pub grid_spacing: Option<f32>,
    pub handle_size: Option<f32>,
    pub animation_speed: Option<f32>,
}

/// Theme presets
#[derive(Debug, Clone, Copy)]
pub enum ThemePreset {
    Light,
    Dark,
    HighContrast,
    ColorblindFriendly,
}

impl ThemePreset {
    pub fn to_style(&self) -> CircuitStyle {
        match self {
            ThemePreset::Light => CircuitStyle::default(),
            ThemePreset::Dark => CircuitStyle::dark_theme(),
            ThemePreset::HighContrast => CircuitStyle::high_contrast(),
            ThemePreset::ColorblindFriendly => CircuitStyle::colorblind_friendly(),
        }
    }
}

/// Component appearance settings
#[derive(Debug, Clone)]
pub struct ComponentAppearance {
    pub show_labels: bool,
    pub show_values: bool,
    pub show_polarity: bool,
    pub label_font_size: f32,
    pub value_font_size: f32,
    pub label_color: Color32,
    pub value_color: Color32,
}

impl Default for ComponentAppearance {
    fn default() -> Self {
        Self {
            show_labels: true,
            show_values: true,
            show_polarity: true,
            label_font_size: 10.0,
            value_font_size: 8.0,
            label_color: Color32::from_rgb(0, 0, 0),
            value_color: Color32::from_rgb(100, 100, 100),
        }
    }
}

impl ComponentAppearance {
    pub fn dark_theme() -> Self {
        Self {
            label_color: Color32::from_rgb(255, 255, 255),
            value_color: Color32::from_rgb(200, 200, 200),
            ..Self::default()
        }
    }
}