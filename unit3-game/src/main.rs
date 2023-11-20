// TODO: use AABB instead of Rect for centered box, so collision checking doesn't have to offset by half size

use engine::Key;
use engine_simple as engine;
use engine_simple::wgpu;
use engine_simple::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::Rng;
use std::path::Path;
const W: f32 = 320.0;
const H: f32 = 240.0;
const GUY_HORZ_SPEED: f32 = 4.0;
const SPRITE_MAX: usize = 128;
const CATCH_DISTANCE: f32 = 16.0;
const COLLISION_STEPS: usize = 3;
const GRAVITY: f32 = 1.0;
const NO_COLLISION: u16 = 9;

const TILE_SIZE: u16 = 256;
const TILE_SHEET_W: u16 = 6 * TILE_SIZE;
const TILE_SHEET_H: u16 = 5 * TILE_SIZE;

struct Guy {
    pos: Vec2,
    vel: Vec2,
    grounded: bool,
}

impl Guy {

    fn doGravity(&mut self, ) {
        if self.vel.y >= -10.0 {            
            self.vel.y -= GRAVITY;
        }
        
    }

    fn setHorzVel(&mut self, direction:f32) {
        self.vel.x = direction * GUY_HORZ_SPEED;
    }

    fn handle_jump(&mut self, vert_dir: f32) {
        if vert_dir > 0.0 && self.grounded {
            self.vel.y = 10.0;
            self.grounded = false;
        }
        
    }

    fn moveGuy(&mut self, horz_dir: f32, vert_dir: f32) {
        
        //Handle velocities
        self.setHorzVel(horz_dir);
        self.handle_jump(vert_dir);
        self.doGravity();
        
        // Update positon
        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;
    }

}

struct Apple {
    pos: Vec2,
    vel: Vec2,
}

struct SpriteTile {
    collision: AABB,
    tex_coord: (u16, u16),
}

struct Game {
    camera: engine::Camera,
    collision_objects: Vec<SpriteTile>,
    guy: Guy,
    apples: Vec<Apple>,
    apple_timer: u32,
    score: u32,
    font: engine_simple::BitFont,
}



fn newSpriteGroup(sprite_path: &str, engine: &mut Engine, camera_ref: &Camera) {
    
    let camera = camera_ref.clone();

    let sprite_img = image::open(sprite_path).unwrap().into_rgba8();
    
    let sprite_tex = engine.renderer.gpu.create_texture(
        &sprite_img,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        sprite_img.dimensions(),
        Some(sprite_path),  // Some string or something
    );

    engine.renderer.sprites.add_sprite_group(
        &engine.renderer.gpu,
        &sprite_tex,
        vec![Transform::zeroed(); SPRITE_MAX], //bg, three walls, guy, a few apples
        vec![SheetRegion::zeroed(); SPRITE_MAX],
        camera,
    );

}

fn newSpriteTile_Square(pos_x: f32, pos_y: f32, size: f32, tex_x: u16, tex_y: u16) -> SpriteTile {
    SpriteTile {
        collision: AABB::new(pos_x, pos_y, size, size),
        tex_coord: (tex_x, tex_y),
    }
}

fn newSpriteTile_Rect(pos_x: f32, pos_y: f32, width: f32, height: f32, tex_x: u16, tex_y: u16) -> SpriteTile {
    SpriteTile {
        collision: AABB::new(pos_x, pos_y, width, height),
        tex_coord: (tex_x, tex_y),
    }
}

 fn addSpriteTileRow(col_objs: &mut Vec<SpriteTile>, sprite_idxs: Vec<(u16, u16)>, y_pos: f32, size: f32) {
    
    let mut x_pos: f32 = 16.0;
    let x_incr: f32 = 32.0;

    for s_idx in sprite_idxs.iter() {
        col_objs.push(newSpriteTile_Square(x_pos, y_pos, size, s_idx.0, s_idx.1));
        x_pos += x_incr;
    }
 }


// Meant to get from a uniform grid
fn getSpriteFromSheet(sheet_num: u16, x: u16, y: u16, depth: u16, sprite_size: u16) -> SheetRegion {
    SheetRegion::new(sheet_num, x*sprite_size, y*sprite_size, depth, sprite_size, sprite_size)
}

// Meant to just get it directly based on data
fn getSpriteFromSheet_Demo(sheet_num: u16, x: u16, y: u16, depth: u16, w: u16, h: u16) -> SheetRegion {
    SheetRegion::new(sheet_num, x, y, depth, w, h)
}

impl engine::Game for Game {

    fn new(engine: &mut Engine) -> Self {
        let camera = Camera {
            screen_pos: [0.0, 0.0],
            screen_size: [W, H],
        };
        #[cfg(target_arch = "wasm32")]
        let sprite_img = {
            let img_bytes = include_bytes!("content/demo.png");
            image::load_from_memory_with_format(&img_bytes, image::ImageFormat::Png)
                .map_err(|e| e.to_string())
                .unwrap()
                .into_rgba8()
        };
        #[cfg(not(target_arch = "wasm32"))]

        

        newSpriteGroup("content/demo.png", engine, &camera); // 0
        newSpriteGroup("content/Tiles/tile_sheet.png", engine, &camera); // 1

        // let sprite_img = image::open("content/demo.png").unwrap().into_rgba8();
        // let sprite_tex = engine.renderer.gpu.create_texture(
        //     &sprite_img,
        //     wgpu::TextureFormat::Rgba8UnormSrgb,
        //     sprite_img.dimensions(),
        //     Some("spr-demo.png"),
        // );
        // engine.renderer.sprites.add_sprite_group(
        //     &engine.renderer.gpu,
        //     &sprite_tex,
        //     vec![Transform::zeroed(); SPRITE_MAX], //bg, three walls, guy, a few apples
        //     vec![SheetRegion::zeroed(); SPRITE_MAX],
        //     camera,
        // );

        let guy = Guy {
            pos: Vec2 {
                x: W / 2.0,
                y: H / 2.0,
            },
            vel: Vec2 {
                x: 0.0,
                y: 0.0,
            },
            grounded: false,
        };



        //              size_x
        //            --------------
        //   size_y   | c_xy x     |
        //            --------------

        let mut collision_objects: Vec<SpriteTile> = Vec::default(); 
        
        let row4_sprite_idxs = vec![(9,9),(9,9),(0,3),(1,3),(2,3),(9,9),(9,9),(3,4),(9,9),(9,9),];
        let row3_sprite_idxs = vec![(1,4),(9,9),(9,9),(9,9),(9,9),(9,9),(9,9),(9,9),(9,9),(9,9),];
        let row2_sprite_idxs = vec![(3,4),(0,4),(0,4),(0,4),(0,4),(0,4),(0,4),(0,0),(0,0),(0,4),];
        let row1_sprite_idxs = vec![(3,4),(3,4),(3,4),(3,4),(3,4),(3,4),(3,4),(1,0),(1,0),(3,4),];

        addSpriteTileRow(&mut collision_objects, row1_sprite_idxs, 16.0, 32.0);
        addSpriteTileRow(&mut collision_objects, row2_sprite_idxs, 48.0, 32.0);
        addSpriteTileRow(&mut collision_objects, row3_sprite_idxs, 80.0, 32.0);
        addSpriteTileRow(&mut collision_objects, row4_sprite_idxs, 112.0, 32.0);

        // let full_tile0 = newSpriteTile_Square(16.0, 16.0, 32.0, 1, 1); //AABB::new(16.0, 16.0, 32.0, 32.0);
        // let full_tile1 = newSpriteTile_Square(48.0, 16.0, 32.0, 1, 1);
        // let full_tile2 = newSpriteTile_Square(80.0, 16.0, 32.0, 1, 1);
        // let full_tile3 = newSpriteTile_Square(112.0, 16.0, 32.0, 1, 1);
        // let full_tile4 = newSpriteTile_Square(144.0, 16.0, 32.0, 1, 1);
        // let full_tile5 = newSpriteTile_Square(176.0, 16.0, 32.0, 1, 1);
        // let full_tile6 = newSpriteTile_Square(208.0, 16.0, 32.0, 1, 1);
        // let full_tile7 = newSpriteTile_Square(240.0, 16.0, 32.0, 1, 1);
        // let full_tile8 = newSpriteTile_Square(272.0, 16.0, 32.0, 1, 1);
        // let full_tile9 = newSpriteTile_Square(304.0, 16.0, 32.0, 1, 1);


        // let floor = newSpriteTile_Rect(W / 2.0, 8.0, W / 3.0, 32.0, 1, 1);
        

        // let floor2 = newSpriteTile_Rect(W / 4.0, 128.0, 32.0, 16.0, 1, 1);
        // collision_objects.push(floor2);

        // let test_wall = AABB::new(32.0, 75.0, 160.0, 50.0); 
        // let test_wall = newSpriteTile_Rect(W / 3.0, 100.0, 32.0, 64.0, 1, 1);
        // collision_objects.push(test_wall);

        let left_wall = newSpriteTile_Rect(8.0, H / 2.0, 16.0, H, 1, 1);
        collision_objects.push(left_wall);

        let right_wall = newSpriteTile_Rect(W - 8.0, H /2.0, 16.0, H, 1, 1);
        collision_objects.push(right_wall);

        let font = engine::BitFont::with_sheet_region(
            '0'..='9',
            SheetRegion::new(0, 0, 512, 0, 80, 8),
            10,
        );
        //vec![full_tile0, full_tile1, full_tile2, full_tile3, full_tile4, full_tile5, full_tile6, full_tile7, full_tile8, full_tile9, left_wall, right_wall, test_wall, floor, floor2]
        Game {
            camera,
            guy,
            collision_objects,
            apples: Vec::with_capacity(16),
            apple_timer: 0,
            score: 0,
            font,
        }
    }
    fn update(&mut self, engine: &mut Engine) {

        // Character movement ------------------------------------------------------------------------
        let dir_x = engine.input.key_axis(engine::Key::Left, engine::Key::Right);
        let dir_y = engine.input.key_axis(engine::Key::Down, engine::Key::Up);

        self.guy.moveGuy(dir_x, dir_y);
        // Character movement ------------------------------------------------------------------------


        if engine.input.is_key_pressed(engine::Key::R) {
            self.guy.pos = Vec2 {
                x: W / 2.0,
                y: 24.0,
            }
        }






        // Collision ------------------------------------------------------------------------
        let mut contacts = Vec::with_capacity(self.collision_objects.len());
        // TODO: for multiple guys this might be better as flags on the guy for what side he's currently colliding with stuff on
        for _iter in 0..COLLISION_STEPS {
            let guy_aabb = AABB {
                center: self.guy.pos,
                size: Vec2 { x: 16.0, y: 16.0 },
            };
            contacts.clear();
            // TODO: to generalize to multiple guys, need to iterate over guys first and have guy_index, rect_index, displacement in a contact tuple
            contacts.extend(
                self.collision_objects
                    .iter()
                    .filter(|tile| tile.tex_coord.0 != NO_COLLISION)
                    .enumerate()
                    .filter_map(|(ri, w)| w.collision.displacement(guy_aabb).map(|d| (ri, d))),
            );
            if contacts.is_empty() {
                break;
            }
            // This part stays mostly the same for multiple guys, except the shape of contacts is different
            contacts.sort_by(|(_r1i, d1), (_r2i, d2)| {
                d2.length_squared()
                    .partial_cmp(&d1.length_squared())
                    .unwrap()
            });
        


            for (wall_idx, _disp) in contacts.iter() {
                // TODO: for multiple guys should access self.guys[guy_idx].
                let guy_aabb = AABB {
                    center: self.guy.pos,
                    size: Vec2 { x: 16.0, y: 16.0 },
                };
                let wall = self.collision_objects[*wall_idx].collision;
                let mut disp = wall.displacement(guy_aabb).unwrap_or(Vec2::ZERO);
                
                // We got to a basically zero collision amount
                if disp.x.abs() < std::f32::EPSILON || disp.y.abs() < std::f32::EPSILON {
                    break;
                }
                

                // Guy is below wall, push down
                if self.guy.pos.y < wall.center.y {
                    disp.y *= -1.0;
                }else if self.guy.pos.y > wall.center.y {
                    disp.y *= 1.0;
                }

                // Guy is left of wall, push left
                if self.guy.pos.x < wall.center.x {
                    disp.x *= -1.0;

                // Guy is right of wall, push left
                } else if self.guy.pos.x > wall.center.x {
                    disp.x *= 1.0;
                }

                
                if disp.y.abs() <= disp.x.abs(){
                    
                    // Guy is above wall, push up

                    self.guy.pos.y += disp.y;
                    self.guy.vel.y = 0.0;

                    if(self.guy.vel.y <= 0.0) {
                        self.guy.grounded = true;
                    }

                    
                    // so far it seems resolved; for multiple guys this should probably set a flag on the guy
                }else if  disp.x.abs() <= disp.y.abs() {
                    self.guy.pos.x += disp.x;
                    self.guy.vel.x = 0.0;
                    
                    // so far it seems resolved; for multiple guys this should probably set a flag on the guy
                }

                println!("x vel: {}, y vel: {}", self.guy.vel.x , self.guy.vel.y );
                

                
            }
        }
        // Collision ------------------------------------------------------------------------





        
        // Regerate apples ------------------------------------------------------------------------
        let mut rng = rand::thread_rng();
        if self.apple_timer > 0 {
            self.apple_timer -= 1;
        } else if self.apples.len() < 8 {
            self.apples.push(Apple {
                pos: Vec2 {
                    x: rng.gen_range(8.0..(W - 8.0)),
                    y: H + 8.0,
                },
                vel: Vec2 {
                    x: 0.0,
                    y: rng.gen_range((-4.0)..(-1.0)),
                },
            });
            self.apple_timer = rng.gen_range(30..90);
        }
        for apple in self.apples.iter_mut() {
            apple.pos += apple.vel;
        }
        if let Some(idx) = self
            .apples
            .iter()
            .position(|apple| apple.pos.distance(self.guy.pos) <= CATCH_DISTANCE)
        {
            self.apples.swap_remove(idx);
            self.score += 1;
        }
        self.apples.retain(|apple| apple.pos.y > -8.0)
        // Regerate apples ------------------------------------------------------------------------
    }
    
    
    
    fn render(&mut self, engine: &mut Engine) {
        
        let DEMO_SPRITE_GROUP = 0;
        let TILE_SPRITE_GROUP = 1;
        

        // [idx 0, idx 1..guy_idx, guy_idx, apple_start..)]
        // [Background, walls..., guy, apples...]


        // set bg image
        let (trfs1, uvs1) = engine.renderer.sprites.get_sprites_mut(TILE_SPRITE_GROUP);
        trfs1[0] = AABB::new(W /2.0, H /2.0, W, H).into();  // Create a non-collision AABB for use in the background
        // AABB {
        //     center: Vec2 {
        //         x: W / 2.0,
        //         y: H / 2.0,
        //     },
        //     size: Vec2 { x: W, y: H },
        // }
        // .into();
        // Get the sprite from tiles at coords (1,2)
        uvs1[0] = getSpriteFromSheet(TILE_SPRITE_GROUP as u16, 1, 2, 16, TILE_SIZE);
    
        // SheetRegion::new(TILE_SPRITE_GROUP as u16, 1*TILE_SIZE, 2*TILE_SIZE, 16, TILE_SIZE, TILE_SIZE);
        
        


        // let (trfs2, uvs2) = engine.renderer.sprites.get_sprites_mut(TILE_SPRITE_GROUP);

        // set walls
        const WALL_START: usize = 1;
        let guy_idx = WALL_START + self.collision_objects.len();
        for (wall, (trf, uv)) in self.collision_objects.iter().zip(
            trfs1[WALL_START..guy_idx]
                .iter_mut()
                .zip(uvs1[WALL_START..guy_idx].iter_mut()),
        ) {
            *trf = (wall.collision).into();
            // *uv = getSpriteFromSheet_Demo(DEMO_SPRITE_GROUP as u16, 0, 480, 12, 8, 8);
            *uv = getSpriteFromSheet(TILE_SPRITE_GROUP as u16, wall.tex_coord.0, wall.tex_coord.1, 12, TILE_SIZE);
            // SheetRegion::new(0, 0, 480, 12, 8, 8);
        }

        let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(DEMO_SPRITE_GROUP);





        // set guy
        trfs[guy_idx] = AABB {
            center: self.guy.pos,
            size: Vec2 { x: 16.0, y: 16.0 },
        }
        .into();
        // TODO animation frame
        uvs[guy_idx] = getSpriteFromSheet_Demo(DEMO_SPRITE_GROUP as u16, 16, 480, 8, 16, 16);
        // SheetRegion::new(0, 16, 480, 8, 16, 16);
        // set apple
        let apple_start = guy_idx + 1;
        for (apple, (trf, uv)) in self.apples.iter().zip(
            trfs[apple_start..]
                .iter_mut()
                .zip(uvs[apple_start..].iter_mut()),
        ) {
            *trf = AABB {
                center: apple.pos,
                size: Vec2 { x: 16.0, y: 16.0 },
            }
            .into();
            *uv = getSpriteFromSheet_Demo(DEMO_SPRITE_GROUP as u16, 0, 496, 4, 16, 16);
            //SheetRegion::new(0, 0, 496, 4, 16, 16);
        }
        let sprite_count = apple_start + self.apples.len();
        let score_str = self.score.to_string();
        let text_len = score_str.len();
        engine.renderer.sprites.resize_sprite_group(
            &engine.renderer.gpu,
            0,
            sprite_count + text_len,
        );
        self.font.draw_text(
            &mut engine.renderer.sprites,
            0,
            sprite_count,
            &score_str,
            Vec2 {
                x: 16.0,
                y: H - 16.0,
            }
            .into(),
            16.0,
        );


        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, DEMO_SPRITE_GROUP, 0..sprite_count + text_len);
        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, TILE_SPRITE_GROUP, 0..self.collision_objects.len() + 1);
        engine
            .renderer
            .sprites
            .set_camera_all(&engine.renderer.gpu, self.camera);
    }
}
fn main() {
    Engine::new(winit::window::WindowBuilder::new()).run::<Game>();
}