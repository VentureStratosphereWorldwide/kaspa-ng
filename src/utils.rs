use crate::imports::*;
use qrcode::render::svg;
use qrcode::*;

pub fn render_qrcode(text: &str, width: usize, height: usize) -> String {
    let code = QrCode::with_version(text, Version::Normal(4), EcLevel::L).unwrap();

    // let _theme = theme();

    code.render::<svg::Color<'_>>()
        .min_dimensions(width as u32, height as u32)
        .dark_color(svg::Color("#ffffff"))
        .light_color(svg::Color("#00000000"))
        .build()
        .to_string()
}
// #[macro_use]
#[macro_export]
macro_rules! spawn {
    ($args: expr) => {{
        let id = concat!(file!(), ":", line!());
        println!("#### spawn ID: {}", id);
        let payload = Payload::new(id);
        if !payload.is_pending() {
            println!("spawning...");
            spawn_with_result(&payload, $args);
        }
        payload.take()
    }};
}

pub use spawn;

/* */
pub fn icon_with_text(ui: &Ui, icon: &str, color: Color32, text: &str) -> LayoutJob {
    // ui.horizontal(|ui| {
    //     ui.add(crate::theme::icon(icon));
    //     ui.add(crate::theme::text(text));
    // });

    // let size = ui.ctx().style().text_styles.entry(TextStyle::Button).or_default().size;
    let text_color = ui.ctx().style().visuals.widgets.inactive.text_color();//.text_color();
    let text_size = ui.ctx().style().text_styles.get(&TextStyle::Button).unwrap().size;

    let theme = theme();

    let mut job = LayoutJob {
        halign: Align::Min,
        // justify: true,
        ..Default::default()
    };

    job.append(
        icon,
        0.0,
        TextFormat {
            // font_id: FontId::new(text_size + 4., FontFamily::Name("phosphor".into())),
            font_id: FontId::new(text_size + 4., FontFamily::Proportional),
            color,
            ..Default::default()
        },
    );
    //  job.append(text, leading_space, format)
    job.append(
        text,
        2.0,
        TextFormat {
            font_id: FontId::new(text_size, FontFamily::Proportional),
            color: text_color,
            ..Default::default()
        },
    );
    // job.append(
    //     wallet.filename.clone().as_str(),
    //     0.0,
    //     TextFormat {
    //         font_id: FontId::new(12.0, FontFamily::Proportional),
    //         color: ui.ctx().style().visuals.text_color(),
    //         ..Default::default()
    //     },
    // );

    job

}