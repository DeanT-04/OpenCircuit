//! Basic drawing primitives for circuit components
//! 
//! Provides fundamental shapes and drawing functions for electronic components,
//! wires, and other circuit elements used in schematic rendering.

use egui::{Color32, Pos2, Rect, Rounding, Stroke, Ui, Vec2};

/// Basic drawing primitives for circuit components
pub struct CircuitPrimitives;

impl CircuitPrimitives {
    /// Draw a resistor symbol
    pub fn draw_resistor(ui: &mut Ui, center: Pos2, size: f32, color: Color32) -> Rect {
        let rect = Rect::from_center_size(center, Vec2::splat(size));
        let painter = ui.painter();
        
        // Draw resistor body
        let body_width = size * 0.8;
        let body_height = size * 0.3;
        let body_rect = Rect::from_center_size(center, Vec2::new(body_width, body_height));
        
        painter.rect_filled(body_rect, Rounding::none(), color);
        
        // Draw connection lines
        let left_line = [Pos2::new(center.x - size * 0.5, center.y), Pos2::new(center.x - body_width * 0.5, center.y)];
        let right_line = [Pos2::new(center.x + body_width * 0.5, center.y), Pos2::new(center.x + size * 0.5, center.y)];
        
        painter.line_segment(left_line, Stroke::new(2.0, color));
        painter.line_segment(right_line, Stroke::new(2.0, color));
        
        rect
    }

    /// Draw a capacitor symbol
    pub fn draw_capacitor(ui: &mut Ui, center: Pos2, size: f32, color: Color32) -> Rect {
        let rect = Rect::from_center_size(center, Vec2::splat(size));
        let painter = ui.painter();
        
        // Draw capacitor plates
        let plate_width = size * 0.1;
        let plate_height = size * 0.8;
        let gap = size * 0.3;
        
        let left_plate = Rect::from_center_size(
            Pos2::new(center.x - gap * 0.5, center.y),
            Vec2::new(plate_width, plate_height)
        );
        let right_plate = Rect::from_center_size(
            Pos2::new(center.x + gap * 0.5, center.y),
            Vec2::new(plate_width, plate_height)
        );
        
        painter.rect_filled(left_plate, Rounding::none(), color);
        painter.rect_filled(right_plate, Rounding::none(), color);
        
        // Draw connection lines
        let left_line = [Pos2::new(center.x - size * 0.5, center.y), Pos2::new(left_plate.left(), center.y)];
        let right_line = [Pos2::new(right_plate.right(), center.y), Pos2::new(center.x + size * 0.5, center.y)];
        
        painter.line_segment(left_line, Stroke::new(2.0, color));
        painter.line_segment(right_line, Stroke::new(2.0, color));
        
        rect
    }

    /// Draw an inductor symbol
    pub fn draw_inductor(ui: &mut Ui, center: Pos2, size: f32, color: Color32) -> Rect {
        let rect = Rect::from_center_size(center, Vec2::splat(size));
        let painter = ui.painter();
        
        // Draw inductor coils
        let coil_radius = size * 0.15;
        let num_coils = 4;
        let coil_spacing = size * 0.15;
        
        let start_x = center.x - size * 0.4;
        
        // Draw connection lines
        let left_line = [Pos2::new(center.x - size * 0.5, center.y), Pos2::new(start_x, center.y)];
        let right_line = [Pos2::new(center.x + size * 0.5, center.y), Pos2::new(center.x + size * 0.4, center.y)];
        
        painter.line_segment(left_line, Stroke::new(2.0, color));
        painter.line_segment(right_line, Stroke::new(2.0, color));
        
        // Draw coils
        for i in 0..num_coils {
            let x = start_x + i as f32 * coil_spacing;
            let coil_rect = Rect::from_center_size(
                Pos2::new(x + coil_spacing * 0.5, center.y),
                Vec2::new(coil_spacing, coil_radius * 2.0)
            );
            painter.rect_stroke(coil_rect, Rounding::none(), Stroke::new(2.0, color));
        }
        
        rect
    }

    /// Draw a voltage source symbol
    pub fn draw_voltage_source(ui: &mut Ui, center: Pos2, size: f32, color: Color32) -> Rect {
        let rect = Rect::from_center_size(center, Vec2::splat(size));
        let painter = ui.painter();
        
        // Draw circle for voltage source
        let circle_radius = size * 0.3;
        painter.circle(center, circle_radius, Color32::TRANSPARENT, Stroke::new(2.0, color));
        
        // Draw polarity symbols
        let plus_pos = Pos2::new(center.x, center.y - circle_radius * 0.5);
        let minus_pos = Pos2::new(center.x, center.y + circle_radius * 0.5);
        
        // Plus sign
        let plus_size = size * 0.1;
        let plus_h = [Pos2::new(plus_pos.x - plus_size, plus_pos.y), Pos2::new(plus_pos.x + plus_size, plus_pos.y)];
        let plus_v = [Pos2::new(plus_pos.x, plus_pos.y - plus_size), Pos2::new(plus_pos.x, plus_pos.y + plus_size)];
        painter.line_segment(plus_h, Stroke::new(2.0, color));
        painter.line_segment(plus_v, Stroke::new(2.0, color));
        
        // Minus sign
        let minus_size = size * 0.1;
        let minus_line = [Pos2::new(minus_pos.x - minus_size, minus_pos.y), Pos2::new(minus_pos.x + minus_size, minus_pos.y)];
        painter.line_segment(minus_line, Stroke::new(2.0, color));
        
        // Draw connection lines
        let top_line = [Pos2::new(center.x, center.y - size * 0.5), Pos2::new(center.x, center.y - circle_radius)];
        let bottom_line = [Pos2::new(center.x, center.y + circle_radius), Pos2::new(center.x, center.y + size * 0.5)];
        
        painter.line_segment(top_line, Stroke::new(2.0, color));
        painter.line_segment(bottom_line, Stroke::new(2.0, color));
        
        rect
    }

    /// Draw a current source symbol
    pub fn draw_current_source(ui: &mut Ui, center: Pos2, size: f32, color: Color32) -> Rect {
        let rect = Rect::from_center_size(center, Vec2::splat(size));
        let painter = ui.painter();
        
        // Draw circle for current source
        let circle_radius = size * 0.3;
        painter.circle(center, circle_radius, Color32::TRANSPARENT, Stroke::new(2.0, color));
        
        // Draw arrow for current direction
        let arrow_start = Pos2::new(center.x, center.y - circle_radius * 0.5);
        let arrow_end = Pos2::new(center.x, center.y + circle_radius * 0.5);
        
        let arrow_head_size = size * 0.1;
        let arrow_head_left = Pos2::new(center.x - arrow_head_size, center.y + circle_radius * 0.3);
        let arrow_head_right = Pos2::new(center.x + arrow_head_size, center.y + circle_radius * 0.3);
        
        painter.line_segment([arrow_start, arrow_end], Stroke::new(2.0, color));
        painter.line_segment([arrow_end, arrow_head_left], Stroke::new(2.0, color));
        painter.line_segment([arrow_end, arrow_head_right], Stroke::new(2.0, color));
        
        // Draw connection lines
        let top_line = [Pos2::new(center.x, center.y - size * 0.5), Pos2::new(center.x, center.y - circle_radius)];
        let bottom_line = [Pos2::new(center.x, center.y + circle_radius), Pos2::new(center.x, center.y + size * 0.5)];
        
        painter.line_segment(top_line, Stroke::new(2.0, color));
        painter.line_segment(bottom_line, Stroke::new(2.0, color));
        
        rect
    }

    /// Draw a ground symbol
    pub fn draw_ground(ui: &mut Ui, center: Pos2, size: f32, color: Color32) -> Rect {
        let rect = Rect::from_center_size(center, Vec2::splat(size));
        let painter = ui.painter();
        
        // Draw ground symbol (three horizontal lines)
        let line_width = size * 0.6;
        let line_height = size * 0.05;
        let spacing = size * 0.1;
        
        for i in 0..3 {
            let y = center.y + i as f32 * spacing;
            let width = line_width * (1.0 - i as f32 * 0.3);
            let line_start = Pos2::new(center.x - width * 0.5, y);
            let line_end = Pos2::new(center.x + width * 0.5, y);
            painter.line_segment([line_start, line_end], Stroke::new(line_height * 10.0, color));
        }
        
        // Draw connection line
        let top_line = [Pos2::new(center.x, center.y - size * 0.5), Pos2::new(center.x, center.y)];
        painter.line_segment(top_line, Stroke::new(2.0, color));
        
        rect
    }

    /// Draw a generic component symbol
    pub fn draw_generic_component(ui: &mut Ui, center: Pos2, size: f32, color: Color32, label: &str) -> Rect {
        let rect = Rect::from_center_size(center, Vec2::splat(size));
        let painter = ui.painter();
        
        // Draw rectangle for generic component
        let component_rect = Rect::from_center_size(center, Vec2::new(size * 0.8, size * 0.5));
        painter.rect_stroke(component_rect, Rounding::none(), Stroke::new(2.0, color));
        
        // Draw connection lines
        let top_line = [Pos2::new(center.x, center.y - size * 0.5), Pos2::new(center.x, component_rect.top())];
        let bottom_line = [Pos2::new(center.x, component_rect.bottom()), Pos2::new(center.x, center.y + size * 0.5)];
        
        painter.line_segment(top_line, Stroke::new(2.0, color));
        painter.line_segment(bottom_line, Stroke::new(2.0, color));
        
        // Draw label
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(size * 0.3),
            color,
        );
        
        rect
    }

    /// Draw a wire between two points
    pub fn draw_wire(ui: &mut Ui, start: Pos2, end: Pos2, color: Color32, thickness: f32) -> Rect {
        let painter = ui.painter();
        
        // Draw wire line
        painter.line_segment([start, end], Stroke::new(thickness, color));
        
        // Calculate bounding rect
        let min_x = start.x.min(end.x);
        let max_x = start.x.max(end.x);
        let min_y = start.y.min(end.y);
        let max_y = start.y.max(end.y);
        
        Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y))
    }

    /// Draw a junction point
    pub fn draw_junction(ui: &mut Ui, center: Pos2, radius: f32, color: Color32) -> Rect {
        let rect = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
        let painter = ui.painter();
        
        painter.circle(center, radius, color, Stroke::new(1.0, color));
        
        rect
    }

    /// Draw a grid
    pub fn draw_grid(ui: &mut Ui, rect: &Rect, spacing: f32, color: Color32) {
        let painter = ui.painter();
        
        // Vertical lines
        let mut x = rect.left();
        while x <= rect.right() {
            let line = [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())];
            painter.line_segment(line, Stroke::new(1.0, color));
            x += spacing;
        }
        
        // Horizontal lines
        let mut y = rect.top();
        while y <= rect.bottom() {
            let line = [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)];
            painter.line_segment(line, Stroke::new(1.0, color));
            y += spacing;
        }
    }

    /// Draw a selection rectangle
    pub fn draw_selection_rect(ui: &mut Ui, rect: &Rect, color: Color32) {
        let painter = ui.painter();
        
        painter.rect_stroke(
            *rect,
            Rounding::none(),
            Stroke::new(2.0, color),
        );
        
        // Draw selection handles
        let handles = [
            rect.left_top(),
            rect.right_top(),
            rect.left_bottom(),
            rect.right_bottom(),
            rect.center_top(),
            rect.center_bottom(),
            rect.left_center(),
            rect.right_center(),
        ];
        
        for handle in handles {
            painter.circle(handle, 4.0, color, Stroke::new(1.0, color));
        }
    }

    /// Draw a measurement probe
    pub fn draw_probe(ui: &mut Ui, center: Pos2, size: f32, color: Color32, value: &str) {
        let painter = ui.painter();
        
        // Draw probe circle
        let circle_radius = size * 0.3;
        painter.circle(center, circle_radius, Color32::from_rgba_premultiplied(255, 255, 0, 100), Stroke::new(2.0, color));
        
        // Draw probe tip
        let tip = Pos2::new(center.x, center.y + circle_radius);
        let bottom = Pos2::new(center.x, center.y + size * 0.5);
        painter.line_segment([tip, bottom], Stroke::new(2.0, color));
        
        // Draw value label
        painter.text(
            Pos2::new(center.x, center.y - circle_radius - 5.0),
            egui::Align2::CENTER_BOTTOM,
            value,
            egui::FontId::proportional(size * 0.25),
            color,
        );
    }
}