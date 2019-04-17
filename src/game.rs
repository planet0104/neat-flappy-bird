use std::f32::consts::PI;
pub static GAME_WIDTH: i32 = 500;
pub static GAME_HEIGHT: i32 = 512;
pub static IMG_BACKGROUND_WIDTH: f64 = 288.;
pub static IMG_PIPETOP_HEIGHT: f64 = 512.;
use crate::neat::ga::GA;
use crate::neat::phenotype::RunType;
use quicksilver::{
    geom::{Shape, Transform},
    graphics::{Background::Img, Color, Font, FontStyle, Image},
    lifecycle::{Asset, Window},
    Result,
};

pub static POP_SIZE: i32 = 60;

pub struct Bird {
    x: f64,
    y: f64,
    width: f64,
    height: f64,

    alive: bool,
    gravity: f64,
    velocity: f64,
    jump: f64,
}

impl Bird {
    pub fn flap(&mut self) {
        self.gravity = self.jump;
    }
    pub fn update(&mut self) {
        self.gravity += self.velocity;
        self.y += self.gravity;
    }

    fn is_dead(&self, height: f64, pipes: &Vec<Pipe>) -> bool {
        if self.y >= height || self.y + self.height <= 0.0 {
            return true;
        }
        for i in 0..pipes.len() {
            if !(self.x > pipes[i].x + pipes[i].width
                || self.x + self.width < pipes[i].x
                || self.y > pipes[i].y + pipes[i].height
                || self.y + self.height < pipes[i].y)
            {
                return true;
            }
        }
        false
    }
}

impl Default for Bird {
    fn default() -> Self {
        Bird {
            x: 80.0,
            y: 250.0,
            width: 40.0,
            height: 30.0,

            alive: true,
            gravity: 0.0,
            velocity: 0.3,
            jump: -6.0,
        }
    }
}

struct Pipe {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    speed: f64,
    pass: bool,
}
impl Pipe {
    pub fn update(&mut self) {
        self.x -= self.speed;
    }
    pub fn is_out(&self) -> bool {
        self.x + self.width < 0.0
    }
}
impl Default for Pipe {
    fn default() -> Self {
        Pipe {
            pass: false,
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 40.0,
            speed: 3.0,
        }
    }
}

pub struct Game {
    font: Asset<Font>,
    font_style: FontStyle,
    img_bird: Asset<Image>,
    img_background: Asset<Image>,
    img_pipebottom: Asset<Image>,
    img_pipetop: Asset<Image>,
    pipes: Vec<Pipe>,
    birds: Vec<Bird>,
    score: i32,
    frame_count: i32,
    width: f64,
    height: f64,
    spawn_interval: i32,
    interval: i32,
    ga: GA,
    alives: usize,
    generation: i32,
    background_speed: f64,
    backgroundx: f64,
    max_score: i32,
    status_texture: Option<(StatusInfo, Image)>,
    pub key_ctrl_pressed: bool,
}
impl Default for Game {
    fn default() -> Self {
        Game {
            font: Asset::new(Font::load("FZFSJW.TTF")),
            font_style: FontStyle::new(18.0, Color::WHITE),
            img_bird: Asset::new(Image::load("bird.png")),
            img_background: Asset::new(Image::load("background.png")),
            img_pipebottom: Asset::new(Image::load("pipebottom.png")),
            img_pipetop: Asset::new(Image::load("pipetop.png")),
            pipes: vec![],
            birds: vec![],
            score: 0,
            frame_count: 0,
            width: GAME_WIDTH as f64,
            height: GAME_HEIGHT as f64,
            spawn_interval: 90,
            interval: 0,
            ga: GA::new(POP_SIZE, 2, 1),
            alives: 0,
            generation: 0,
            background_speed: 0.5,
            backgroundx: 0.0,
            max_score: 0,
            status_texture: None,
            key_ctrl_pressed: false,
        }
    }
}

struct StatusInfo {
    score: i32,
    max_score: i32,
    generation: i32,
    alives: usize,
}

impl Game {
    pub fn start(&mut self) {
        self.interval = 0;
        self.frame_count = 0;
        self.score = 0;
        self.pipes = vec![];
        self.birds = vec![];

        //下一代
        self.ga.epoch();
        for _ in 0..self.ga.pop_size() {
            self.birds.push(Bird::default());
        }
        self.generation += 1;
        self.alives = self.birds.len();
    }

    fn update_status_texture(&mut self) -> Result<()> {
        let texture = &mut self.status_texture;
        let font_style = &self.font_style;

        let score = self.score;
        let max_score = self.max_score;
        let generation = self.generation;
        let alives = self.alives;

        self.font.execute(|font| {
            *texture = Some((StatusInfo{
            score,
            max_score,
            generation,
            alives
        }, font.render(&format!("Score : {}\nMax Score : {}\nGeneration : {}\nAlive : {} / {}\nCtrl+1~5：x1~x5\nCtrl+M：MAX\nF5：RESET",
            score,
            max_score,
            generation,
            alives,
            POP_SIZE
            ), font_style).unwrap()));
            Ok(())
        })
    }

    pub fn reset(&mut self) {
        self.ga = GA::new(POP_SIZE, 2, 1);
        self.max_score = 0;
        self.generation = 0;
        self.start();
    }

    pub fn update(&mut self, _window: &mut Window) -> Result<()> {
        self.backgroundx += self.background_speed;
        let mut next_holl = 0.0;
        if self.birds.len() > 0 {
            for i in (0..self.pipes.len()).step_by(2) {
                if self.pipes[i].x + self.pipes[i].width > self.birds[0].x {
                    next_holl = self.pipes[i].height / self.height;
                    break;
                }
            }
        }

        for i in 0..self.birds.len() {
            if self.birds[i].alive {
                let inputs = [self.birds[i].y / self.height, next_holl];

                //网络处理
                let phenotype = self.ga.get_phenotype(i);
                let output = phenotype.update(&inputs, RunType::Active);
                if output[0] > 0.5 {
                    self.birds[i].flap();
                }

                self.birds[i].update();
                if self.birds[i].is_dead(self.height, &self.pipes) {
                    self.birds[i].alive = false;
                    self.alives -= 1;
                    //设置网络得分
                    self.ga.fitness_scores()[i] = self.frame_count as f64;
                    if self.is_it_end() {
                        self.start();
                    }
                }
            }
        }

        for pipei in 0..self.pipes.len() {
            let pipe = &mut self.pipes[pipei];
            pipe.update();
            if !pipe.pass && pipe.x + pipe.width / 2.0 < 80.0 {
                pipe.pass = true;
                //加分
                self.score += 1;
                if self.score > self.max_score {
                    self.max_score = self.score;
                }
            }
        }
        self.pipes.retain(|pipe| !pipe.is_out());

        if self.interval == 0 {
            let delta_bord = 50.0;
            let pipe_holl = 120.0;
            let holl_position =
                (rand::random::<f64>() * (self.height - delta_bord * 2.0 - pipe_holl)).round()
                    + delta_bord;
            self.pipes.push(Pipe {
                x: self.width,
                y: 0.0,
                height: holl_position,
                ..Default::default()
            });
            self.pipes.push(Pipe {
                x: self.width,
                y: holl_position + pipe_holl,
                height: self.height,
                ..Default::default()
            });
        }

        self.interval += 1;
        if self.interval == self.spawn_interval {
            self.interval = 0;
        }

        self.frame_count += 1;
        Ok(())
    }

    pub fn draw(&mut self, window: &mut Window) -> Result<()> {
        let backgroundx = self.backgroundx;
        for i in 0..((self.width / IMG_BACKGROUND_WIDTH).ceil() + 1.0) as usize {
            self.img_background.execute(|image| {
                window.draw(
                    &image.area().translate((
                        (i as f64 * IMG_BACKGROUND_WIDTH
                            - (backgroundx % IMG_BACKGROUND_WIDTH).floor())
                            as i32,
                        0,
                    )),
                    Img(&image),
                );
                Ok(())
            })?;
        }

        for i in 0..self.pipes.len() {
            let pipe_x = self.pipes[i].x as i32;
            if i % 2 == 0 {
                let pipe_y = (self.pipes[i].y + self.pipes[i].height - IMG_PIPETOP_HEIGHT) as i32;
                self.img_pipetop.execute(|image| {
                    window.draw(&image.area().translate((pipe_x, pipe_y)), Img(&image));
                    Ok(())
                })?;
            } else {
                let pipe_y = self.pipes[i].y as i32;
                self.img_pipebottom.execute(|image| {
                    window.draw(&image.area().translate((pipe_x, pipe_y)), Img(&image));
                    Ok(())
                })?;
            }
        }

        for i in 0..self.birds.len() {
            if self.birds[i].alive {
                let t = (
                    (self.birds[i].x + self.birds[i].width / 2.0) as i32,
                    (self.birds[i].y + self.birds[i].height / 2.0) as i32,
                );
                let r = (PI / 2.0 * self.birds[i].gravity as f32 / 20.0) * 90.0;
                self.img_bird.execute(|image| {
                    window.draw_ex(
                        &image.area().with_center((0, 0)),
                        Img(&image),
                        Transform::translate(t) * Transform::rotate(r),
                        0,
                    );
                    Ok(())
                })?;
            }
        }

        //需要在draw函数中渲染文字
        if let Some((status, _)) = &self.status_texture {
            if status.score != self.score
                || status.max_score != self.max_score
                || status.alives != self.alives
                || status.generation != self.generation
            {
                self.update_status_texture()?;
            }
        } else {
            self.update_status_texture()?;
        }

        if let Some((_, image)) = &mut self.status_texture {
            window.draw(&image.area().translate((10, 15)), Img(&image));
        }

        window.flush()?;
        //绘制最好的网络
        let brains: Vec<usize> = self.ga.get_best_phenotypes_from_last_generation();
        if brains.len() > 0 {
            self.ga.get_phenotype(brains[0]).draw_net(
                window,
                GAME_WIDTH - 120,
                GAME_HEIGHT - 100,
                GAME_WIDTH,
                GAME_HEIGHT - 10,
            )?;
        }

        Ok(())
    }

    pub fn is_it_end(&self) -> bool {
        for i in 0..self.birds.len() {
            if self.birds[i].alive {
                return false;
            }
        }
        true
    }
}
