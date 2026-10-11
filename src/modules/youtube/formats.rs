use egui_extras::{Column, TableBuilder};

use super::ytdlp;

pub fn render_formats_table(
    selected_format: &mut Option<usize>,
    ui: &mut egui::Ui,
    video_info: &ytdlp::VideoInfo,
) {
    ui.scope(|ui| {
        ui.style_mut().interaction.selectable_labels = false;

        let table = TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .sense(egui::Sense::click())
            .column(Column::auto().at_least(40.0))
            .column(Column::auto().at_least(70.0))
            .column(Column::auto().at_least(110.0))
            .column(Column::auto().at_least(50.0))
            .column(Column::remainder().at_least(40.0));

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Format ID");
                });
                header.col(|ui| {
                    ui.heading("Resolution");
                });
                header.col(|ui| {
                    ui.heading("Codec");
                });
                header.col(|ui| {
                    ui.heading("Extension");
                });
                header.col(|ui| {
                    ui.heading("FPS");
                });
            })
            .body(|mut body| {
                for (index, format) in video_info.formats.iter().enumerate() {
                    body.row(20.0, |mut row| {
                        row.set_selected(*selected_format == Some(index));

                        row.col(|ui| {
                            ui.label(&format.format_id);
                        });
                        row.col(|ui| {
                            ui.label(&format.resolution);
                        });
                        row.col(|ui| {
                            ui.label(&format.vcodec);
                        });
                        row.col(|ui| {
                            ui.label(&format.video_ext);
                        });
                        row.col(|ui| {
                            ui.label(format!("{}", format.fps.unwrap_or(0.0)));
                        });

                        if row.response().clicked() {
                            *selected_format = Some(index);
                        }
                    });
                }
            });
    });
}
