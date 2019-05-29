use std::f64::consts::PI;
pub static GAME_WIDTH: f64 = 500.;
pub static GAME_HEIGHT: f64 = 512.;
use mengine::*;
use neat::ga::GA;
use neat::phenotype::RunType;
use std::collections::HashMap;

pub static POP_SIZE: i32 = 60;

pub const ASSETS_BIRD: &str = "bird.png";
pub const ASSETS_BACKGROUND: &str = "background.png";
pub const ASSETS_PIPE_BOTTOM: &str = "pipebottom.png";
pub const ASSETS_PIPE_TOP: &str = "pipetop.png";

const RESOURCES: &'static [(&'static str, AssetsType); 4] = &[
    (ASSETS_BIRD, AssetsType::Image),
    (ASSETS_BACKGROUND, AssetsType::Image),
    (ASSETS_PIPE_BOTTOM, AssetsType::Image),
    (ASSETS_PIPE_TOP, AssetsType::Image),
];

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

struct Stage {
    img_bird: Image,
    img_pipebottom: Image,
    img_pipetop: Image,
    background: engine::ScrollingBackground,
}

pub struct Game {
    resources: HashMap<String, Assets>,
    stage: Option<Stage>,
    best_net: Option<Image>,
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
    pub fn new(window: &mut Window) -> Game {
        window.load_assets(RESOURCES.to_vec());
        Game {
            resources: HashMap::new(),
            stage: None,
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
            window.load_svg("netimg", net_img);
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

    pub fn update(&mut self, window: &mut Window) {
        let stage = self.stage.as_mut().unwrap();

        stage.background.update();
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
                (random() * (self.height - delta_bord * 2.0 - pipe_holl)).round() + delta_bord;
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

    pub fn draw(&mut self, g: &mut Graphics) {
        let stage = self.stage.as_mut().unwrap();

        stage.background.draw(g);
        for i in 0..self.pipes.len() {
            let pipe_x = self.pipes[i].x;
            if i % 2 == 0 {
                let pipe_y = self.pipes[i].y + self.pipes[i].height - stage.img_pipetop.height();
                g.draw_image_at(None, &stage.img_pipetop, pipe_x, pipe_y);
            } else {
                let pipe_y = self.pipes[i].y;
                g.draw_image_at(None, &stage.img_pipebottom, pipe_x, pipe_y);
            };
        }

        for i in 0..self.birds.len() {
            if self.birds[i].alive {
                let r = PI / 2.0 * self.birds[i].gravity / 20.0;
                g.draw_image_at(
                    Some(Transform {
                        rotate: r,
                        translate: (self.birds[i].x, self.birds[i].y),
                    }),
                    &stage.img_bird,
                    -self.birds[i].width / 2.0,
                    -self.birds[i].height / 2.0,
                );
            }
        }

        let text_color = &[255, 255, 255, 255];
        g.draw_text(
            &format!("Score : {}", self.score),
            10.0,
            20.0,
            text_color,
            18,
        );
        g.draw_text(
            &format!("Max Score : {}", self.max_score),
            10.0,
            40.0,
            text_color,
            18,
        );
        g.draw_text(
            &format!("Generation : {}", self.generation),
            10.0,
            60.0,
            text_color,
            18,
        );
        g.draw_text(
            &format!("Alive : {}/{}", self.alives, POP_SIZE),
            10.0,
            80.0,
            text_color,
            18,
        );
        g.draw_text("Ctrl+1~5：x1~x5", 10.0, 100.0, text_color, 18);
        g.draw_text("Ctrl+M：MAX", 10.0, 120.0, text_color, 18);
        g.draw_text("F5：RESET", 10.0, 140.0, text_color, 18);

        //绘制最好的网络
        if let Some(best_net) = &self.best_net {
            g.draw_image_at(
                None,
                &best_net,
                GAME_WIDTH - best_net.width(),
                GAME_HEIGHT - best_net.height(),
            );
        }
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

impl State for Game {
    fn new(window: &mut Window) -> Self {
        let mut game = Game::new(window);
        game.start(window);
        game
    }

    fn update(&mut self, window: &mut Window) {
        if self.stage.is_none() {
            return;
        }
        self.update(window);
    }

    fn event(&mut self, event: Event, window: &mut Window) {
        if self.stage.is_none() {
            return;
        }
        // log(format!("{:?}", event));
        match event {
            Event::KeyUp(key) => {
                match key.to_lowercase().as_str() {
                    "f5" => self.reset(window),
                    "control" => self.key_ctrl_pressed = false,
                    _ => (),
                };
            }
            Event::KeyDown(key) => {
                match key.to_lowercase().as_str() {
                    "f5" => self.reset(window),
                    "control" => self.key_ctrl_pressed = true,
                    "1" => {
                        if self.key_ctrl_pressed {
                            window.set_update_rate(60);
                        }
                    }
                    "2" => {
                        if self.key_ctrl_pressed {
                            window.set_update_rate(120);
                        }
                    }
                    "3" => {
                        if self.key_ctrl_pressed {
                            window.set_update_rate(180);
                        }
                    }
                    "4" => {
                        if self.key_ctrl_pressed {
                            window.set_update_rate(240);
                        }
                    }
                    "5" => {
                        if self.key_ctrl_pressed {
                            window.set_update_rate(300);
                        }
                    }
                    "m" => {
                        if self.key_ctrl_pressed {
                            window.set_update_rate(1000);
                        }
                    }
                    _ => (),
                };
            }
            _ => (),
        };
    }

    fn on_assets_load(
        &mut self,
        path: &str,
        _: AssetsType,
        assets: std::io::Result<Assets>,
        _window: &mut Window,
    ) {
        if path == "netimg" {
            if let Ok(assets) = assets {
                self.best_net = Some(assets.as_image().unwrap());
            }
            return;
        }
        match assets {
            Ok(assets) => {
                self.resources.insert(path.to_string(), assets);

                if self.resources.len() == RESOURCES.len() {
                    let background = self
                        .resources
                        .get(ASSETS_BACKGROUND)
                        .unwrap()
                        .as_image()
                        .unwrap();
                    let bglayer = engine::BackgroundLayer::new(
                        background,
                        Rect::new(0.0, 0.0, GAME_WIDTH, GAME_HEIGHT),
                        0.5,
                        engine::ScrollDir::Left,
                    );
                    let mut background = engine::ScrollingBackground::new();
                    background.add_layer(bglayer);
                    self.stage = Some(Stage {
                        img_bird: self.resources.get(ASSETS_BIRD).unwrap().as_image().unwrap(),
                        img_pipebottom: self
                            .resources
                            .get(ASSETS_PIPE_BOTTOM)
                            .unwrap()
                            .as_image()
                            .unwrap(),
                        img_pipetop: self
                            .resources
                            .get(ASSETS_PIPE_TOP)
                            .unwrap()
                            .as_image()
                            .unwrap(),
                        background: background,
                    });
                }
            }
            Err(err) => alert(
                "温馨提示",
                &format!("资源文件加载失败:{:?} {:?}", path, err).as_str(),
            ),
        }
    }

    fn draw(&mut self, g: &mut Graphics, _window: &mut Window) {
        g.fill_rect(&[255, 255, 255, 255], 0.0, 0.0, GAME_WIDTH, GAME_HEIGHT);
        if self.stage.is_none() {
            return;
        }
        self.draw(g);
    }
}
