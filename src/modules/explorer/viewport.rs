use egui::{Color32, Pos2, Vec2, emath::TSTransform};

use super::grid;
use super::{ExplorerModule, ExplorerPoint};

fn zoom_to_fit(points: &[ExplorerPoint], canvas_size: Vec2) -> (Vec2, f32) {
    if points.is_empty() {
        return (Vec2::ZERO, 1.0);
    }

    let min_time = points.iter().map(|p| p.time).min().unwrap();
    let max_time = points.iter().map(|p| p.time).max().unwrap();
    let min_tick = points.iter().map(|p| p.tick).min().unwrap();
    let max_tick = points.iter().map(|p| p.tick).max().unwrap();

    let data_width = (max_time - min_time) as f32;
    let data_height = (max_tick - min_tick) as f32;

    let zoom_x = if data_width > 0.0 {
        canvas_size.x / data_width
    } else {
        f32::INFINITY
    };
    let zoom_y = if data_height > 0.0 {
        canvas_size.y / data_height
    } else {
        f32::INFINITY
    };

    let zoom = zoom_x.min(zoom_y);
    if !zoom.is_finite() {
        return (Vec2::ZERO, 1.0);
    }

    let center_data = Vec2::new(
        (min_time + max_time) as f32 / 2.0,
        -((min_tick + max_tick) as f32 / 2.0),
    );

    let pan = canvas_size * 0.5 - center_data * zoom;

    (pan, zoom)
}

fn draw_point(painter: &egui::Painter, point: Pos2, selected: bool) {
    if selected {
        painter.circle_filled(point, 6.5, Color32::from_rgb(255, 100, 100));
    }
    painter.circle_filled(point, 5.0, Color32::from_white_alpha(255));
    painter.circle_filled(point, 3.5, Color32::from_black_alpha(255));
}

impl ExplorerModule {
    pub(super) fn explorer_viewer(&mut self, ui: &mut egui::Ui) {
        let fit_requested = ui.input(|i| i.key_pressed(egui::Key::F)) || self.fit_view;
        self.fit_view = false;
        self.handle_keybinds(ui.ctx());
        egui::Frame::canvas(ui.style())
            .fill(Color32::from_rgb(20, 20, 24))
            .corner_radius(8.0)
            .show(ui, |ui| {
                let (response, painter) =
                    ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
                if fit_requested {
                    let (pan, zoom) = zoom_to_fit(&self.points, response.rect.size());
                    self.pan = pan;
                    self.zoom = zoom;
                }
                self.handle_zoom(&response, ui);
                let to_screen = self.to_screen(&response.rect);
                self.handle_pointer(&response, ui, to_screen.inverse(), to_screen);
                self.update_edge_scroll(&response, ui);
                self.paint(&painter, &response.rect, self.to_screen(&response.rect));
            });
    }

    pub(super) fn clear_selection(&mut self) {
        for p in &mut self.points {
            p.selected = false;
        }
    }

    fn points_in_bbox(&self, start: Pos2, end: Pos2) -> Vec<usize> {
        let world_rect = egui::Rect::from_two_pos(start, end);
        self.points
            .iter()
            .enumerate()
            .filter_map(|(i, p)| world_rect.contains(p.world_pos()).then_some(i))
            .collect()
    }

    fn point_at(&self, to_screen: TSTransform, screen_pos: Pos2) -> Option<usize> {
        const CLICK_HIT_RADIUS: f32 = 8.0;
        self.points
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                let dist = (to_screen * p.world_pos()).distance(screen_pos);
                (dist <= CLICK_HIT_RADIUS).then_some((i, dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    }

    fn handle_click(&mut self, hit: Option<usize>, shift_held: bool) {
        match hit {
            Some(i) => {
                if shift_held {
                    self.points[i].selected = !self.points[i].selected;
                } else {
                    let was_selected = self.points[i].selected;
                    self.clear_selection();
                    self.points[i].selected = !was_selected;
                }
            }
            None => {
                if !shift_held {
                    self.clear_selection();
                }
            }
        }
    }

    fn apply_box_selection(&mut self, start: Pos2, end: Pos2, shift_held: bool) {
        let hits = self.points_in_bbox(start, end);
        if !shift_held {
            self.clear_selection();
        }
        for i in hits {
            self.points[i].selected = true;
        }
    }

    fn to_screen(&self, canvas_rect: &egui::Rect) -> TSTransform {
        TSTransform::from_translation(canvas_rect.min.to_vec2() + self.pan)
            * TSTransform::from_scaling(self.zoom)
    }

    fn handle_keybinds(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
            self.points.retain(|p| !p.selected);
        }
    }

    fn handle_zoom(&mut self, response: &egui::Response, ui: &egui::Ui) {
        if !response.hovered() {
            return;
        }
        let zoom_delta = ui.input(|i| i.zoom_delta());
        if zoom_delta == 1.0 {
            return;
        }
        let Some(mouse_pos) = response.hover_pos() else {
            return;
        };
        let local_mouse = mouse_pos - response.rect.min;
        self.zoom *= zoom_delta;
        self.zoom = self.zoom.clamp(0.0001, 100.0);
        if self.zoom < 100.0 && self.zoom > 0.0001 {
            self.pan = local_mouse - (local_mouse - self.pan) * zoom_delta;
        }
    }

    fn handle_pointer(
        &mut self,
        response: &egui::Response,
        ui: &egui::Ui,
        from_screen: TSTransform,
        to_screen: TSTransform,
    ) {
        if ui.input(|i| i.modifiers.ctrl) {
            if response.dragged() {
                self.pan += response.drag_delta();
            }
            if response.drag_stopped() {
                self.selection_box = None;
            }
            return;
        }
        if response.clicked()
            && let Some(screen_pos) = response.interact_pointer_pos().or(response.hover_pos())
        {
            let hit = self.point_at(to_screen, screen_pos);
            let shift_held = ui.input(|i| i.modifiers.shift);
            self.handle_click(hit, shift_held);
            return;
        }
        if response.drag_started()
            && let Some(pos) = response.interact_pointer_pos()
        {
            let world = from_screen * pos;
            self.selection_box = Some((world, world));
        } else if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let world = from_screen * pos;
                let (start, _) = self.selection_box.unwrap_or((world, world));
                self.selection_box = Some((start, world));
            }
        } else if response.drag_stopped() {
            if let Some((start, end)) = self.selection_box {
                let shift_held = ui.input(|i| i.modifiers.shift);
                self.apply_box_selection(start, end, shift_held);
            }
            self.selection_box = None;
        }
    }

    fn update_edge_scroll(&mut self, response: &egui::Response, ui: &egui::Ui) {
        if self.selection_box.is_none() {
            return;
        }
        if ui.input(|i| i.modifiers.ctrl) {
            return;
        }
        let Some(pos) = response.interact_pointer_pos() else {
            return;
        };
        const MARGIN: f32 = 28.0;
        const MAX_SPEED: f32 = 800.0;
        let dt = ui.input(|i| i.stable_dt).clamp(1.0 / 240.0, 1.0 / 10.0);
        let ramp = |dist: f32| (1.0 - dist / MARGIN).clamp(0.0, 1.0);
        let mut shift = Vec2::ZERO;
        shift.x -= MAX_SPEED * ramp(response.rect.max.x - pos.x) * dt;
        shift.x += MAX_SPEED * ramp(pos.x - response.rect.min.x) * dt;
        shift.y -= MAX_SPEED * ramp(response.rect.max.y - pos.y) * dt;
        shift.y += MAX_SPEED * ramp(pos.y - response.rect.min.y) * dt;
        if shift == Vec2::ZERO {
            return;
        }
        self.pan += shift;
        let from_screen = self.to_screen(&response.rect).inverse();
        if let Some((start, _)) = self.selection_box {
            self.selection_box = Some((start, from_screen * pos));
        }
        ui.ctx().request_repaint();
    }

    fn paint(&self, painter: &egui::Painter, canvas_rect: &egui::Rect, to_screen: TSTransform) {
        grid::draw_grid(painter, *canvas_rect, to_screen, self.zoom);
        for (p, p2) in self.points.iter().zip(self.points.iter().skip(1)) {
            let pos1 = to_screen * p.world_pos();
            let pos2 = to_screen * p2.world_pos();
            painter.line_segment([pos1, pos2], (1.0, Color32::from_white_alpha(255)));
        }
        for point in &self.points {
            draw_point(painter, to_screen * point.world_pos(), point.selected);
        }
        if let Some((start, end)) = self.selection_box {
            painter.rect_stroke(
                egui::Rect::from_two_pos(to_screen * start, to_screen * end),
                5.0,
                (1.0, Color32::from_white_alpha(255)),
                egui::StrokeKind::Middle,
            );
        }
    }
}
