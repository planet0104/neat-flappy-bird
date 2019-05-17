use std::f64::consts::PI;
pub static GAME_WIDTH: f64 = 500.;
pub static GAME_HEIGHT: f64 = 512.;
use neat::ga::GA;
use neat::phenotype::RunType;
use mengine::*;
use std::rc::Rc;

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
    background: engine::ScrollingBackground,
    img_bird: Rc<Image>,
    img_pipebottom: Rc<Image>,
    img_pipetop: Rc<Image>,
    best_net: Option<Rc<Image>>,
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
    max_score: i32,
    pub key_ctrl_pressed: bool,
}

impl Game {
    pub fn new(loader:&mut Window) -> Game{
        let bglayer = engine::BackgroundLayer::new(loader.load_image("background.png").unwrap(),
                Rect::new(0.0, 0.0, GAME_WIDTH, GAME_HEIGHT), 0.5, engine::ScrollDir::Left);
        let mut background = engine::ScrollingBackground::new();
        background.add_layer(bglayer);
        Game {
            img_bird: loader.load_image("bird.png").unwrap(),
            background,
            img_pipebottom: loader.load_image("pipebottom.png").unwrap(),
            img_pipetop: loader.load_image("pipetop.png").unwrap(),
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
            max_score: 0,
            key_ctrl_pressed: false,
            best_net: None,
        }
    }

    pub fn start(&mut self, window: &mut Window) {
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

        //绘制最好的网络
        let brains: Vec<usize> = self.ga.get_best_phenotypes_from_last_generation();
        if brains.len() > 0 {
            let net_img = self.ga.get_phenotype(brains[0]).draw_net(120, 100, 10);
            self.best_net = Some(window.load_image_alpha(&net_img).unwrap());
        }
        self.generation += 1;
        self.alives = self.birds.len();
    }

    pub fn reset(&mut self, window: &mut Window) {
        self.ga = GA::new(POP_SIZE, 2, 1);
        self.max_score = 0;
        self.generation = 0;
        self.start(window);
    }

    pub fn update(&mut self, window: &mut Window){
        self.background.update();
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
                        self.start(window);
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
                (random() * (self.height - delta_bord * 2.0 - pipe_holl)).round()
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
    }

    pub fn draw(&mut self, g: &mut Graphics) -> Result<(), String> {
        self.background.draw(g)?;

        for i in 0..self.pipes.len() {
            let pipe_x = self.pipes[i].x;
            if i % 2 == 0 {
                let pipe_y =  self.pipes[i].y + self.pipes[i].height - self.img_pipetop.height();
                g.draw_image_at(None, self.img_pipetop.as_ref(), pipe_x, pipe_y)?;
            } else {
                let pipe_y =  self.pipes[i].y;
                g.draw_image_at(None, self.img_pipebottom.as_ref(), pipe_x, pipe_y)?;
            };
        }

        for i in 0..self.birds.len() {
            if self.birds[i].alive {
                let r = PI / 2.0 * self.birds[i].gravity / 20.0;
                g.draw_image_at(Some(Transform{rotate: r, translate:(self.birds[i].x, self.birds[i].y)}), self.img_bird.as_ref(),  - self.birds[i].width / 2.0, - self.birds[i].height / 2.0)?;
            }
        }

        let text_color = &[255, 255, 255, 255];
        g.draw_text(None, &format!("Score : {}", self.score), 10.0, 20.0, text_color, 18)?;
        g.draw_text(None, &format!("Max Score : {}", self.max_score), 10.0, 40.0, text_color, 18)?;
        g.draw_text(None, &format!("Generation : {}", self.generation), 10.0, 60.0, text_color, 18)?;
        g.draw_text(None, &format!("Alive : {}/{}", self.alives, POP_SIZE), 10.0, 80.0, text_color, 18)?;
        g.draw_text(None, "Ctrl+1~5：x1~x5", 10.0, 100.0, text_color, 18)?;
        g.draw_text(None, "Ctrl+M：MAX", 10.0, 120.0, text_color, 18)?;
        g.draw_text(None, "F5：RESET", 10.0, 140.0, text_color, 18)?;

        //绘制最好的网络
        if let Some(best_net) = &self.best_net{
            g.draw_image_at(None, best_net.as_ref(), GAME_WIDTH-best_net.width(), GAME_HEIGHT-best_net.height())?;
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
