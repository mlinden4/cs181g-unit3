// TODO: use AABB instead of Rect for centered box, so collision checking doesn't have to offset by half size

use engine_simple as engine;
use engine_simple::wgpu;
use engine_simple::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::Rng;
use std::path::Path;
const W: f32 = 320.0;
const H: f32 = 240.0;
const GUY_HORZ_SPEED: f32 = 4.0;
const SPRITE_MAX: usize = 16;
const CATCH_DISTANCE: f32 = 16.0;
const COLLISION_STEPS: usize = 3;
const GRAVITY: f32 = 1.0;

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
        if(self.vel.y >= -15.0) {            
            self.vel.y -= GRAVITY;
        }
        
    }

    fn setHorzVel(&mut self, direction:f32) {
        self.vel.x = direction * GUY_HORZ_SPEED;
    }

    fn handle_jump(&mut self, vert_dir: f32) {
        if(vert_dir > 0.0 && self.grounded){
            self.vel.y = 15.0;
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

struct Game {
    camera: engine::Camera,
    collision_objects: Vec<AABB>,
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

        newSpriteGroup("content/demo.png", engine, &camera);
        newSpriteGroup("content/Tiles/tile_sheet.png", engine, &camera);

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
                y: 24.0,
            },
            vel: Vec2 {
                x: 0.0,
                y: 0.0,
            },
            grounded: true,
        };



        //              size_x
        //            --------------
        //   size_y   | c_xy x     |
        //            --------------
        let floor = AABB::new(W / 2.0, 8.0, 128.0, 16.0);

        let floor2 = AABB::new(W / 4.0, 64.0, 32.0, 16.0);

        // let test_wall = AABB::new(32.0, 75.0, 160.0, 50.0); 
        let test_wall = AABB::new(W / 3.0, 128.0, 32.0, 64.0);
        
        let left_wall = AABB::new(8.0, H / 2.0, 16.0, H);
      
        let right_wall = AABB::new(W - 8.0, H /2.0, 16.0, H); 

        let font = engine::BitFont::with_sheet_region(
            '0'..='9',
            SheetRegion::new(0, 0, 512, 0, 80, 8),
            10,
        );

        Game {
            camera,
            guy,
            collision_objects: vec![left_wall, right_wall, test_wall, floor, floor2],
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
                    .enumerate()
                    .filter_map(|(ri, w)| w.displacement(guy_aabb).map(|d| (ri, d))),
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
                let wall = self.collision_objects[*wall_idx];
                let mut disp = wall.displacement(guy_aabb).unwrap_or(Vec2::ZERO);
                // We got to a basically zero collision amount
                if disp.x.abs() < std::f32::EPSILON || disp.y.abs() < std::f32::EPSILON {
                    break;
                }
                // Guy is left of wall, push left
                if self.guy.pos.x < wall.center.x {
                    disp.x *= -1.0;
                }
                // Guy is below wall, push down
                if self.guy.pos.y < wall.center.y {
                    disp.y *= -1.0;
                }
                if disp.x.abs() <= disp.y.abs() {
                    self.guy.pos.x += disp.x;
                    // so far it seems resolved; for multiple guys this should probably set a flag on the guy
                } else if disp.y.abs() <= disp.x.abs() {
                    
                    // self.guy.pos.y += disp.y;
                    if(self.guy.vel.y < 0.0) {
                        self.guy.grounded = true;
                    }

                    self.guy.pos.y += disp.y;
                    self.guy.vel.y = 0.0;
                    
                    // so far it seems resolved; for multiple guys this should probably set a flag on the guy
                }
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
            *trf = (*wall).into();
            // *uv = getSpriteFromSheet_Demo(DEMO_SPRITE_GROUP as u16, 0, 480, 12, 8, 8);
            *uv = getSpriteFromSheet(TILE_SPRITE_GROUP as u16, 4, 0, 12, TILE_SIZE);
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
            .upload_sprites(&engine.renderer.gpu, TILE_SPRITE_GROUP, 0..6);
        engine
            .renderer
            .sprites
            .set_camera_all(&engine.renderer.gpu, self.camera);
    }
}
fn main() {
    Engine::new(winit::window::WindowBuilder::new()).run::<Game>();
}