use eframe::egui::{Color32, Pos2, Rect, Rounding, Sense, Ui, Vec2};

pub fn show(ui: &mut Ui, downloaded: usize, total: usize, cols: usize) {
    if total == 0 { return; }

    let rows = (total + cols - 1) / cols;
    let block_size = 6.0;
    let gap = 1.0;
    let total_w = cols as f32 * (block_size + gap);
    let total_h = rows as f32 * (block_size + gap);

    let (resp, painter) = ui.allocate_painter(
        Vec2::new(total_w, total_h.min(150.0)),
        Sense::hover(),
    );

    let clip_rect = resp.rect;
    let origin = clip_rect.left_top();

    let filled_color = Color32::from_rgb(0, 180, 80);
    let empty_color = Color32::from_rgb(220, 220, 220);

    let visible_rows = (clip_rect.height() / (block_size + gap)) as usize + 1;
    let start_row = 0usize;
    let end_row = (start_row + visible_rows).min(rows);

    for row in start_row..end_row {
        for col in 0..cols {
            let idx = row * cols + col;
            if idx >= total { break; }

            let x = origin.x + col as f32 * (block_size + gap);
            let y = origin.y + row as f32 * (block_size + gap);
            let rect = Rect::from_min_size(
                Pos2::new(x, y),
                Vec2::splat(block_size),
            );

            let color = if idx < downloaded { filled_color } else { empty_color };
            painter.rect_filled(rect, Rounding::ZERO, color);
        }
    }
}
