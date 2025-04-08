use std::collections::HashMap;

use macroquad::prelude::*;
use robotics_lib::world::tile::TileType;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]

///**ALL** possible sprites
pub(crate) enum SpritesType {
    Rock,
    Tree,
    Garbage,
    Fire,
    Coin,
    Bin,
    Crate,
    Bank,
    Water,
    Market,
    Fish,
    Building,
    Bush,
    JollyBlock,
    Scarecrow,
    DeepWater,
    ShallowWater,
    Sand,
    Grass,
    Street,
    Hill,
    Mountain,
    Snow,
    Lava,
    Teleport,
    Wall,
    Robot,
}

/// Holds custom data needed to draw the tiles
pub(crate) struct TileSprites {
    tile_img: Texture2D,
    color: Color,
}

impl TileSprites {
    ///Creates a new [TileSprites] based on the [TileType] given
    pub(crate) async fn new(tile: TileType) -> Self {
        match tile {
            TileType::DeepWater => Self {
                tile_img: load_texture("src/visualizer_2/assets/tiles/water.png")
                    .await
                    .unwrap(),
                color: BLUE,
            },
            TileType::ShallowWater => Self {
                tile_img: load_texture("src/visualizer_2/assets/tiles/water.png")
                    .await
                    .unwrap(),
                color: LIGHTGRAY,
            },
            TileType::Sand => Self {
                tile_img: load_texture("src/visualizer_2/assets/tiles/sand.png")
                    .await
                    .unwrap(),
                color: YELLOW,
            },
            TileType::Grass => Self {
                tile_img: load_texture("src/visualizer_2/assets/tiles/grass.png")
                    .await
                    .unwrap(),
                color: WHITE,
            },
            TileType::Street => Self {
                tile_img: load_texture("src/visualizer_2/assets/tiles/road.png")
                    .await
                    .unwrap(),
                color: GRAY,
            },
            TileType::Hill => Self {
                tile_img: load_texture("src/visualizer_2/assets/tiles/hill.png")
                    .await
                    .unwrap(),
                color: WHITE,
            },
            TileType::Mountain => Self {
                tile_img: load_texture("src/visualizer_2/assets/tiles/mountain.png")
                    .await
                    .unwrap(),
                color: GRAY,
            },
            TileType::Snow => Self {
                tile_img: load_texture("src/visualizer_2/assets/tiles/snow.png")
                    .await
                    .unwrap(),
                color: WHITE,
            },
            TileType::Lava => Self {
                tile_img: load_texture("src/visualizer_2/assets/tiles/lava.png")
                    .await
                    .unwrap(),
                color: WHITE,
            },
            TileType::Teleport(_) => Self {
                tile_img: load_texture("src/visualizer_2/assets/tiles/teleport.png")
                    .await
                    .unwrap(),
                color: WHITE,
            },
            TileType::Wall => Self {
                tile_img: load_texture("src/visualizer_2/assets/tiles/wall.png")
                    .await
                    .unwrap(),
                color: WHITE,
            },
        }
    }

    /// Displayes single tile
    pub(crate) fn show(&self, x: usize, y: usize) {
        draw_texture_ex(
            &self.tile_img,
            x as f32,
            y as f32,
            self.color,
            DrawTextureParams {
                dest_size: Some(vec2(1., 1.)),
                ..Default::default()
            },
        );
    }
}

/// Holds custom data needed to draw the contents
pub(crate) struct ContentSprites {
    texture_front: Texture2D,
    texture_side: Texture2D,
}

impl ContentSprites {
    ///Creates a new [ContentSprites] based on the [SpritesType] given
    pub(crate) async fn new(sprite: &SpritesType) -> Self {
        match sprite {
            SpritesType::Bank => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/bank.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/bankSide.png")
                    .await
                    .unwrap(),
            },
            SpritesType::Tree => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/tree.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/treeSide.png")
                    .await
                    .unwrap(),
            },

            SpritesType::Rock => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/rock.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/rockSide.png")
                    .await
                    .unwrap(),
            },
            SpritesType::Robot => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/robot.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/robotSide.png")
                    .await
                    .unwrap(),
            },
            SpritesType::Coin => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/coin.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/coinSide.png")
                    .await
                    .unwrap(),
            },
            SpritesType::Fish => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/fish.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/fishSide.png")
                    .await
                    .unwrap(),
            },
            SpritesType::Garbage => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/garbage.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/garbageSide.png")
                    .await
                    .unwrap(),
            },
            SpritesType::Fire => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/fire.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/fireSide.png")
                    .await
                    .unwrap(),
            },
            SpritesType::Bin => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/bin.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/binSide.png")
                    .await
                    .unwrap(),
            },
            SpritesType::Crate => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/crate.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/crateSide.png")
                    .await
                    .unwrap(),
            },
            SpritesType::Water => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/water.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/waterSide.png")
                    .await
                    .unwrap(),
            },
            SpritesType::Market => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/market.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/marketSide.png")
                    .await
                    .unwrap(),
            },
            SpritesType::Building => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/building.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/buildingSide.png")
                    .await
                    .unwrap(),
            },
            SpritesType::Bush => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/bush.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/bushSide.png")
                    .await
                    .unwrap(),
            },
            SpritesType::JollyBlock => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/jollyblock.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/jollyblockSide.png")
                    .await
                    .unwrap(),
            },
            SpritesType::Scarecrow => Self {
                texture_front: load_texture("src/visualizer_2/assets/contents/scarecrow.png")
                    .await
                    .unwrap(),
                texture_side: load_texture("src/visualizer_2/assets/contents/scarecrowSide.png")
                    .await
                    .unwrap(),
            },
            _ => panic!("TileType tried to be shwon inside ContentSprite"),
        }
    }

    /// Displayes single content
    pub(crate) fn show(&self, x: usize, y: usize, facing_front: bool) {
        let x = x as f32;
        let y = y as f32;
        let texture = if facing_front {
            &self.texture_front
        } else {
            &self.texture_side
        };
        let size = if facing_front {
            vec3(1., 0., 1.)
        } else {
            vec3(0., 1., 1.)
        };

        //needs to be a flat cube to be able to stand vertically. Aka to be flat on the Z-axis
        draw_cube(vec3(x + 0.5, y + 0.5, 0.5), size, Some(texture), WHITE);
    }
}

///manages sprites
pub(crate) struct SpriteManager {
    //hashmaps to show the a copy of the same sprite in tevery place that is needed.
    //These will always be a fixed size during the entire execution
    content_sprites: HashMap<SpritesType, ContentSprites>,
    tile_sprites: HashMap<SpritesType, TileSprites>,
}
impl SpriteManager {
    /// Initialize the sprite manager with every possible entry
    pub(crate) async fn init() -> Self {
        let mut content_sprites = HashMap::new();

        let bank = SpritesType::Bank;
        let bin = SpritesType::Bin;
        let bush = SpritesType::Bush;
        let building = SpritesType::Building;
        let coin = SpritesType::Coin;
        let crate_spr = SpritesType::Crate;
        let fire = SpritesType::Fire;
        let fish = SpritesType::Fish;
        let garbage = SpritesType::Garbage;
        let jolly_block = SpritesType::JollyBlock;
        let market = SpritesType::Market;
        let rock = SpritesType::Rock;
        let scarecrow = SpritesType::Scarecrow;
        let tree = SpritesType::Tree;
        let water = SpritesType::Water;
        let robot = SpritesType::Robot;

        content_sprites.insert(bank.clone(), ContentSprites::new(&bank).await);
        content_sprites.insert(bin.clone(), ContentSprites::new(&bin).await);
        content_sprites.insert(bush.clone(), ContentSprites::new(&bush).await);
        content_sprites.insert(building.clone(), ContentSprites::new(&building).await);
        content_sprites.insert(coin.clone(), ContentSprites::new(&coin).await);
        content_sprites.insert(crate_spr.clone(), ContentSprites::new(&crate_spr).await);
        content_sprites.insert(fire.clone(), ContentSprites::new(&fire).await);
        content_sprites.insert(fish.clone(), ContentSprites::new(&fish).await);
        content_sprites.insert(garbage.clone(), ContentSprites::new(&garbage).await);
        content_sprites.insert(jolly_block.clone(), ContentSprites::new(&jolly_block).await);
        content_sprites.insert(market.clone(), ContentSprites::new(&market).await);
        content_sprites.insert(rock.clone(), ContentSprites::new(&rock).await);
        content_sprites.insert(scarecrow.clone(), ContentSprites::new(&scarecrow).await);
        content_sprites.insert(tree.clone(), ContentSprites::new(&tree).await);
        content_sprites.insert(water.clone(), ContentSprites::new(&water).await);

        content_sprites.insert(robot.clone(), ContentSprites::new(&robot).await);

        let mut tile_sprites = HashMap::new();
        tile_sprites.insert(SpritesType::Grass, TileSprites::new(TileType::Grass).await);
        tile_sprites.insert(SpritesType::Hill, TileSprites::new(TileType::Hill).await);
        tile_sprites.insert(SpritesType::Lava, TileSprites::new(TileType::Lava).await);
        tile_sprites.insert(
            SpritesType::Mountain,
            TileSprites::new(TileType::Mountain).await,
        );
        tile_sprites.insert(
            SpritesType::Street,
            TileSprites::new(TileType::Street).await,
        );
        tile_sprites.insert(SpritesType::Sand, TileSprites::new(TileType::Sand).await);
        tile_sprites.insert(SpritesType::Snow, TileSprites::new(TileType::Snow).await);
        tile_sprites.insert(
            SpritesType::Teleport,
            TileSprites::new(TileType::Teleport(false)).await,
        );
        tile_sprites.insert(SpritesType::Wall, TileSprites::new(TileType::Wall).await);
        tile_sprites.insert(
            SpritesType::ShallowWater,
            TileSprites::new(TileType::ShallowWater).await,
        );
        tile_sprites.insert(
            SpritesType::DeepWater,
            TileSprites::new(TileType::DeepWater).await,
        );

        return Self {
            content_sprites,
            tile_sprites,
        };
    }

    ///Calls the right function to display the content/tile. Based on the [SpritesType] given
    pub(crate) fn show(&self, sprite: SpritesType, x: usize, y: usize, facing_front: bool) {
        match sprite {
            SpritesType::Grass
            | SpritesType::Hill
            | SpritesType::Lava
            | SpritesType::Mountain
            | SpritesType::Street
            | SpritesType::Sand
            | SpritesType::Snow
            | SpritesType::Teleport
            | SpritesType::Wall
            | SpritesType::ShallowWater
            | SpritesType::DeepWater => {
                self.tile_sprites.get(&sprite).unwrap().show(x, y);
            }
            _ => {
                self.content_sprites
                    .get(&sprite)
                    .unwrap()
                    .show(x, y, facing_front);
            }
        }
    }
}
