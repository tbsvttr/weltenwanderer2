//! Entity relationship graph view with force-directed layout.
//!
//! Supports zoom (mouse wheel), pan (drag), and hover to show full node name.

use macroquad::prelude::*;

use crate::app::AppState;
use crate::theme::font::{draw_pixel_text, measure_text_width};
use crate::theme::palette;
use crate::theme::{CANVAS_H, CANVAS_W, kind_color, mouse_canvas_position};
use crate::widget::Rect2;

use super::{Screen, ScreenId, Transition};

/// Maximum characters shown in a node label before truncating.
const LABEL_MAX: usize = 14;

/// A node in the relationship graph.
struct GraphNode {
    /// Entity name.
    name: String,
    /// Entity kind (for coloring).
    kind: String,
    /// X position in world space.
    x: f32,
    /// Y position in world space.
    y: f32,
    /// X velocity for force simulation.
    vx: f32,
    /// Y velocity for force simulation.
    vy: f32,
}

/// An edge between two graph nodes.
struct GraphEdge {
    /// Source node index.
    from: usize,
    /// Target node index.
    to: usize,
    /// Relationship label.
    _label: String,
}

/// Graph view screen state.
pub struct GraphScreen {
    /// Graph nodes.
    nodes: Vec<GraphNode>,
    /// Graph edges.
    edges: Vec<GraphEdge>,
    /// Whether the graph has been initialized.
    initialized: bool,
    /// Simulation tick counter.
    ticks: u32,
    /// Camera offset X (pan).
    cam_x: f32,
    /// Camera offset Y (pan).
    cam_y: f32,
    /// Zoom level (1.0 = 100%).
    zoom: f32,
    /// Whether the user is dragging to pan.
    dragging: bool,
    /// Last mouse position during drag.
    drag_start: (f32, f32),
    /// Index of the hovered node (if any).
    hovered_node: Option<usize>,
}

impl Default for GraphScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphScreen {
    /// Create a new graph screen.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            initialized: false,
            ticks: 0,
            cam_x: 0.0,
            cam_y: 0.0,
            zoom: 1.0,
            dragging: false,
            drag_start: (0.0, 0.0),
            hovered_node: None,
        }
    }

    fn initialize(&mut self, app: &AppState) {
        self.nodes.clear();
        self.edges.clear();

        let Some(world) = &app.world else {
            return;
        };

        // Build node list — layout in a larger virtual space
        let entities: Vec<_> = world.all_entities().collect();
        let mut name_to_idx = std::collections::HashMap::new();
        let n = entities.len().max(1) as f32;

        // Use a larger radius so nodes have room
        let radius = 40.0 + n * 8.0;
        let cx = CANVAS_W / 2.0;
        let cy = CANVAS_H / 2.0;

        for (i, entity) in entities.iter().enumerate() {
            let kind_str = format!("{}", entity.kind);
            let angle = i as f32 * std::f32::consts::TAU / n;
            self.nodes.push(GraphNode {
                name: entity.name.clone(),
                kind: kind_str,
                x: cx + angle.cos() * radius,
                y: cy + angle.sin() * radius,
                vx: 0.0,
                vy: 0.0,
            });
            name_to_idx.insert(entity.id, i);
        }

        // Build edge list (deduplicated)
        let mut seen = std::collections::HashSet::new();
        for entity in &entities {
            for rel in world.relationships_from(entity.id) {
                let from = entity.id;
                let to = rel.target;
                let key = if from.0 < to.0 {
                    (from, to)
                } else {
                    (to, from)
                };
                if seen.insert(key)
                    && let (Some(&fi), Some(&ti)) = (name_to_idx.get(&from), name_to_idx.get(&to))
                {
                    self.edges.push(GraphEdge {
                        from: fi,
                        to: ti,
                        _label: rel
                            .label
                            .clone()
                            .or_else(|| Some(format!("{}", rel.kind)))
                            .unwrap_or_default(),
                    });
                }
            }
        }

        self.initialized = true;
        self.ticks = 0;
    }

    fn simulate_step(&mut self) {
        if self.nodes.len() < 2 {
            return;
        }

        // Stronger repulsion, moderate attraction for better spacing
        let repulsion = 3000.0;
        let attraction = 0.005;
        let damping = 0.85;

        // Repulsion between all pairs
        for i in 0..self.nodes.len() {
            for j in (i + 1)..self.nodes.len() {
                let dx = self.nodes[i].x - self.nodes[j].x;
                let dy = self.nodes[i].y - self.nodes[j].y;
                let dist_sq = dx * dx + dy * dy + 1.0;
                let force = repulsion / dist_sq;
                let fx = dx / dist_sq.sqrt() * force;
                let fy = dy / dist_sq.sqrt() * force;
                self.nodes[i].vx += fx;
                self.nodes[i].vy += fy;
                self.nodes[j].vx -= fx;
                self.nodes[j].vy -= fy;
            }
        }

        // Attraction along edges
        for edge in &self.edges {
            let dx = self.nodes[edge.to].x - self.nodes[edge.from].x;
            let dy = self.nodes[edge.to].y - self.nodes[edge.from].y;
            let fx = dx * attraction;
            let fy = dy * attraction;
            self.nodes[edge.from].vx += fx;
            self.nodes[edge.from].vy += fy;
            self.nodes[edge.to].vx -= fx;
            self.nodes[edge.to].vy -= fy;
        }

        // Gentle centering force to prevent drift
        let cx = CANVAS_W / 2.0;
        let cy = CANVAS_H / 2.0;
        for node in &mut self.nodes {
            node.vx += (cx - node.x) * 0.0005;
            node.vy += (cy - node.y) * 0.0005;
        }

        // Apply velocities with damping (no bounds clamping — user can pan)
        for node in &mut self.nodes {
            node.vx *= damping;
            node.vy *= damping;
            node.x += node.vx;
            node.y += node.vy;
        }
    }

    /// Convert world coordinates to screen coordinates.
    fn world_to_screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        let sx = (wx - self.cam_x) * self.zoom + CANVAS_W / 2.0;
        let sy = (wy - self.cam_y) * self.zoom + CANVAS_H / 2.0;
        (sx, sy)
    }

    /// Convert screen coordinates to world coordinates.
    fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        let wx = (sx - CANVAS_W / 2.0) / self.zoom + self.cam_x;
        let wy = (sy - CANVAS_H / 2.0) / self.zoom + self.cam_y;
        (wx, wy)
    }
}

impl Screen for GraphScreen {
    fn update(&mut self, app: &mut AppState) -> Transition {
        if !self.initialized {
            self.initialize(app);
            // Center camera on the graph
            self.cam_x = CANVAS_W / 2.0;
            self.cam_y = CANVAS_H / 2.0;
        }

        // Run force simulation for first 300 ticks
        if self.ticks < 300 {
            self.simulate_step();
            self.ticks += 1;
        }

        if is_key_pressed(KeyCode::Escape) {
            return Transition::Pop;
        }
        if crate::input::tab_pressed() {
            return Transition::Replace(ScreenId::Timeline);
        }

        let (mx, my) = mouse_canvas_position();

        // Mouse: tab click
        if is_mouse_button_pressed(MouseButton::Left) && my < 14.0 {
            let tab_idx = (mx / (CANVAS_W / 3.0)) as usize;
            match tab_idx {
                0 => return Transition::Replace(ScreenId::Explorer),
                2 => return Transition::Replace(ScreenId::Timeline),
                _ => {}
            }
        }

        // Zoom with mouse wheel
        let wheel = crate::input::scroll_y();
        if wheel != 0.0 {
            let old_zoom = self.zoom;
            if wheel > 0.0 {
                self.zoom = (self.zoom * 1.15).min(4.0);
            } else {
                self.zoom = (self.zoom / 1.15).max(0.2);
            }
            // Zoom toward mouse position
            let (wx, wy) = self.screen_to_world(mx, my);
            self.cam_x += (wx - self.cam_x) * (1.0 - old_zoom / self.zoom);
            self.cam_y += (wy - self.cam_y) * (1.0 - old_zoom / self.zoom);
        }

        // Find hovered node (before click handling)
        self.hovered_node = None;
        let hover_radius = 8.0;
        for (i, node) in self.nodes.iter().enumerate() {
            let (sx, sy) = self.world_to_screen(node.x, node.y);
            let dx = mx - sx;
            let dy = my - sy;
            if dx * dx + dy * dy < hover_radius * hover_radius {
                self.hovered_node = Some(i);
                break;
            }
        }

        // Click handling (below tab bar)
        if is_mouse_button_pressed(MouseButton::Left) && my > 14.0 {
            if let Some(idx) = self.hovered_node {
                // Click on node — select entity and go to explorer
                let node_name = &self.nodes[idx].name;
                if let Some(world) = &app.world
                    && let Some(entity) = world.find_by_name(node_name)
                {
                    app.selected_entity = Some(entity.id);
                    return Transition::Replace(ScreenId::Explorer);
                }
            } else {
                // Click on empty space — start pan drag
                self.dragging = true;
                self.drag_start = (mx, my);
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            self.dragging = false;
        }
        if self.dragging {
            let dx = mx - self.drag_start.0;
            let dy = my - self.drag_start.1;
            self.cam_x -= dx / self.zoom;
            self.cam_y -= dy / self.zoom;
            self.drag_start = (mx, my);
        }

        // Reset zoom with R key
        if is_key_pressed(KeyCode::R) {
            self.zoom = 1.0;
            self.cam_x = CANVAS_W / 2.0;
            self.cam_y = CANVAS_H / 2.0;
        }

        Transition::None
    }

    fn draw(&self, app: &AppState) {
        let (mx, my) = mouse_canvas_position();

        // Tab bar
        let tab_area = Rect2::new(0.0, 0.0, CANVAS_W, 14.0);
        let tabs = ["Entities", "Graph", "Timeline"];
        crate::widget::tabs::draw_tabs(&app.font, &tabs, 1, &tab_area, mx, my);

        // Draw edges
        for edge in &self.edges {
            let (x1, y1) = self.world_to_screen(self.nodes[edge.from].x, self.nodes[edge.from].y);
            let (x2, y2) = self.world_to_screen(self.nodes[edge.to].x, self.nodes[edge.to].y);
            // Only draw if at least partially visible
            if (x1 > -50.0 || x2 > -50.0)
                && (x1 < CANVAS_W + 50.0 || x2 < CANVAS_W + 50.0)
                && (y1 > 10.0 || y2 > 10.0)
            {
                draw_line(x1, y1, x2, y2, 1.0, palette::DARK_GRAY);
            }
        }

        // Draw nodes
        for (i, node) in self.nodes.iter().enumerate() {
            let (sx, sy) = self.world_to_screen(node.x, node.y);

            // Skip nodes far off-screen
            if !(-60.0..=CANVAS_W + 60.0).contains(&sx) || !(10.0..=CANVAS_H + 10.0).contains(&sy) {
                continue;
            }

            let color = kind_color(&node.kind);
            let is_hovered = self.hovered_node == Some(i);
            let r = if is_hovered { 5.0 } else { 3.0 };
            draw_circle(sx, sy, r, color);

            if is_hovered {
                // Hovered: show full name with background
                let name_w = measure_text_width(&node.name);
                let label_x = (sx + 8.0).min(CANVAS_W - name_w - 4.0).max(2.0);
                let label_y = (sy - 12.0).max(16.0);
                draw_rectangle(
                    label_x - 2.0,
                    label_y - 1.0,
                    name_w + 4.0,
                    10.0,
                    palette::BLACK,
                );
                draw_pixel_text(&app.font, &node.name, label_x, label_y, palette::WHITE);
            } else {
                // Normal: show truncated label
                let label = if node.name.len() > LABEL_MAX {
                    let mut s = node.name[..LABEL_MAX - 2].to_string();
                    s.push_str("..");
                    s
                } else {
                    node.name.clone()
                };
                draw_pixel_text(&app.font, &label, sx + 6.0, sy - 3.0, palette::LIGHT_GRAY);
            }
        }

        // Status bar
        draw_rectangle(0.0, CANVAS_H - 12.0, CANVAS_W, 12.0, palette::BLACK);
        let status = if let Some(idx) = self.hovered_node {
            let node = &self.nodes[idx];
            format!(
                "{} ({}) | Scroll:zoom | Drag:pan | R:reset",
                node.name, node.kind
            )
        } else {
            format!(
                "Scroll:zoom | Drag:pan | R:reset | Tab:timeline | Esc:back (x{:.1})",
                self.zoom
            )
        };
        // Truncate status if needed
        let max_chars = ((CANVAS_W - 4.0) / 8.0) as usize;
        let display = if status.len() > max_chars {
            &status[..max_chars]
        } else {
            &status
        };
        draw_pixel_text(&app.font, display, 2.0, CANVAS_H - 10.0, palette::DARK_GRAY);
    }
}
