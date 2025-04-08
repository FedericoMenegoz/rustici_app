use macroquad::{
    color::{BROWN, DARKBROWN, WHITE},
    shapes::{draw_line, draw_rectangle},
    text::draw_text,
    texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D},
    window::{screen_height, screen_width},
};

/// List of possible backpack contens
pub(crate) enum BackpackContent {
    Tree,
    Rock,
    Coin,
    Garbage,
    Fish,
}

/// Everything that is needed to manage the backpack
#[derive(Debug)]
pub(crate) struct Backpack {
    tree: usize,
    tree_sprites: Texture2D,
    rock: usize,
    rock_sprites: Texture2D,
    coin: usize,
    coin_sprites: Texture2D,
    garbage: usize,
    garbage_sprites: Texture2D,
    fish: usize,
    fish_sprites: Texture2D,
}

impl Backpack {
    /// Creates an empty backpack. Can not be the Default method because it needs to be async
    pub(crate) async fn new() -> Self {
        Self {
            tree: 0,
            rock: 0,
            coin: 0,
            garbage: 0,
            fish: 0,
            tree_sprites: load_texture("./src/visualizer_2/assets/contents/tree.png")
                .await
                .unwrap(),
            rock_sprites: load_texture("./src/visualizer_2/assets/contents/rock.png")
                .await
                .unwrap(),
            coin_sprites: load_texture("./src/visualizer_2/assets/contents/coin.png")
                .await
                .unwrap(),
            garbage_sprites: load_texture("./src/visualizer_2/assets/contents/garbage.png")
                .await
                .unwrap(),
            fish_sprites: load_texture("./src/visualizer_2/assets/contents/fish.png")
                .await
                .unwrap(),
        }
    }
}
impl Backpack {
    /// Show backpack contents
    pub(crate) async fn show(&self) {
        let width = screen_width() / 4.;
        let height = screen_height() / 10.;
        let mut x = screen_width() - width;
        draw_rectangle(x - 5., 0., width + 5., height + 5., DARKBROWN);
        draw_rectangle(x, 0., width, height, BROWN);
        let size = width / 5.;

        show_item(&self.tree_sprites, self.tree, &mut x, height, size).await;
        show_item(&self.fish_sprites, self.fish, &mut x, height, size).await;
        show_item(&self.rock_sprites, self.rock, &mut x, height, size).await;
        show_item(&self.garbage_sprites, self.garbage, &mut x, height, size).await;
        show_item(&self.coin_sprites, self.coin, &mut x, height, size).await;
    }
    /// Adds a quantity of the content to the backpack
    pub(crate) fn add(&mut self, content: BackpackContent, quantity: usize) {
        match content {
            BackpackContent::Tree => self.tree += quantity,
            BackpackContent::Rock => self.rock += quantity,
            BackpackContent::Coin => self.coin += quantity,
            BackpackContent::Garbage => self.garbage += quantity,
            BackpackContent::Fish => self.fish += quantity,
        }
    }
    /// Removes a quantity of the content to the backpack
    pub(crate) fn remove(&mut self, content: BackpackContent, quantity: usize) {
        match content {
            BackpackContent::Tree => self.tree -= quantity,
            BackpackContent::Rock => self.rock -= quantity,
            BackpackContent::Coin => self.coin -= quantity,
            BackpackContent::Garbage => self.garbage -= quantity,
            BackpackContent::Fish => self.fish -= quantity,
        }
    }
}

///Wrapper for showing contents inside the backpack
async fn show_item(texture: &Texture2D, amount: usize, x: &mut f32, height: f32, size: f32) {
    draw_line(*x + size, 0., *x + size, height, 3., DARKBROWN);
    draw_texture_ex(
        texture,
        *x + 5.,
        10.,
        WHITE,
        DrawTextureParams {
            dest_size: Some(macroquad::math::vec2(size - 10., size - 10.)),
            ..Default::default()
        },
    );

    draw_text(&format!("{}", amount), *x, 30., 50., WHITE);
    *x = *x + size + 3.;
}
