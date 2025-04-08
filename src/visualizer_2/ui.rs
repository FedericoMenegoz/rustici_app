use macroquad::{
    color::*,
    math::vec2,
    shapes::{draw_line, draw_rectangle},
    text::{draw_text, get_text_center},
    texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D},
    window::{screen_height, screen_width},
};

///Show instructions' box
pub(crate) async fn show() {
    let width = 500.;
    let height = 330.;
    let x = screen_width() - width + 5.;
    let mut y = screen_height() - height + 10.;
    draw_rectangle(x, y, width, height, WHITE);
    y += 30.;
    draw_text("Esc to quit", x, y, 30., BLACK);
    y += 30.;
    show_tab(x, y).await;
    y += 30.;
    show_time(x, y).await;
    y += 35.;
    show_mode(x, y).await;
    y += 50.;
    draw_line(x, y, screen_width(), y, 3., BLACK);
    let title = "While in free-mode";
    let title_pos = get_text_center(title, None, 25, 1., 0.);
    draw_text(title, x + width / 2. - title_pos.x, y + 15., 25., BLACK);
    y += 20.;
    show_wasd(x, y).await;
    y += 30.;
    write_centered_text("Ctrl/Spacebar to move", x, y);
    y += 30.;
    write_centered_text("Shift to sprint", x, y);
    y += 30.;
    write_centered_text("Mouse to control the camera", x, y);
}

///Shows to move camera
async fn show_wasd(x: f32, y: f32) {
    let w_img = load_texture("src/visualizer_2/assets/keys/W.jpg")
        .await
        .unwrap();
    let a_img = load_texture("src/visualizer_2/assets/keys/A.jpg")
        .await
        .unwrap();
    let s_img = load_texture("src/visualizer_2/assets/keys/S.jpg")
        .await
        .unwrap();
    let d_img = load_texture("src/visualizer_2/assets/keys/D.jpg")
        .await
        .unwrap();
    let size = 40.;
    square_img(&w_img, x, y);
    square_img(&a_img, x + size, y);
    square_img(&s_img, x + 2. * size, y);
    square_img(&d_img, x + 3. * size, y);
    write_centered_text("To move", x + 4. * size + 5., y);
}

///Shows to control the mode
async fn show_mode(x: f32, y: f32) {
    let f_img = load_texture("src/visualizer_2/assets/keys/F.jpg")
        .await
        .unwrap();
    square_img(&f_img, x, y);

    write_centered_text("Change mode", x + 45., y);
}

///Shows how to control time
async fn show_time(x: f32, y: f32) {
    let size_img = 40.;
    let right_arrow = load_texture("src/visualizer_2/assets/keys/right_arrow.jpg")
        .await
        .unwrap();
    let left_arrow = load_texture("src/visualizer_2/assets/keys/left_arrow.jpg")
        .await
        .unwrap();
    square_img(&left_arrow, x, y);
    square_img(&right_arrow, x + size_img, y);
    write_centered_text("Control Speed", x + 2. * size_img, y);
}

///Shows how to hide/show the instructions
async fn show_tab(x: f32, y: f32) {
    let tab_img = load_texture("src/visualizer_2/assets/keys/tab.jpg")
        .await
        .unwrap();
    draw_texture_ex(
        &tab_img,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(40., 20.)),
            ..Default::default()
        },
    );
    write_centered_text("Hide/Show this menu", x + 40., y - 20.);
}

///Shows a square image
fn square_img(texture: &Texture2D, x: f32, y: f32) {
    draw_texture_ex(
        texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(40., 40.)),
            ..Default::default()
        },
    );
}

///Shows a text that is centered compared to the image at the same (x,y)
fn write_centered_text(text: &str, x: f32, y: f32) {
    draw_text(text, x + 5., y + 30., 30., BLACK);
}
