use macroquad::prelude::*;
// randの名前衝突を防ぐため、外部クレートであることを明示してインポート
use ::rand::Rng;

// --- ゲームの状態定義 ---
#[derive(PartialEq)]
enum State {
    Title,
    GameSwim,
    GameDive,
    Clear,
    GameOver,
}

// --- 難易度の定義 ---
#[derive(PartialEq, Clone, Copy)]
enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    fn to_string(&self) -> &str {
        match self {
            Difficulty::Easy => "EASY",
            Difficulty::Normal => "NORMAL",
            Difficulty::Hard => "HARD",
        }
    }
}

// --- エフェクト用構造体（波紋） ---
struct Ripple {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    alpha: f32,
}

impl Ripple {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            width: 20.0,
            height: 10.0,
            alpha: 0.8,
        }
    }

    fn update(&mut self) {
        self.width += 1.5;
        self.height += 0.8;
        self.alpha -= 0.03;
    }
}

// --- ゲーム全体のデータを保持する構造体 ---
struct Game {
    current_state: State,
    difficulty: Difficulty, // 難易度を保持
    screen_width: f32,
    screen_height: f32,
    anim_frame: f64,

    // 水泳用変数
    player_x: f32,
    cpu_x: f32,
    goal_x: f32,
    stamina: f32,
    is_exhausted: bool,
    ripples: Vec<Ripple>,

    // 飛び込み用変数
    player_y: f32,
    velocity_y: f32,
    rotations: i32,
    gravity: f32,
    space_pressed: bool,
}

impl Game {
    fn new() -> Self {
        Self {
            current_state: State::Title,
            difficulty: Difficulty::Normal, // デフォルトはNormal
            screen_width: 800.0,
            screen_height: 600.0,
            anim_frame: 0.0,
            player_x: 50.0,
            cpu_x: 50.0,
            goal_x: 700.0,
            stamina: 100.0,
            is_exhausted: false,
            ripples: Vec::new(),
            player_y: 100.0,
            velocity_y: 0.0,
            rotations: 0,
            gravity: 0.5,
            space_pressed: false,
        }
    }

    fn reset_swimming(&mut self) {
        self.player_x = 50.0;
        self.cpu_x = 50.0;
        self.stamina = 100.0;
        self.is_exhausted = false;
        self.ripples.clear();
        self.current_state = State::GameSwim;
    }

    fn reset_diving(&mut self) {
        self.player_y = 100.0;
        self.velocity_y = 0.0;
        self.rotations = 0;
        self.space_pressed = false;
        
        // 難易度によって重力（落下速度）を変更
        self.gravity = match self.difficulty {
            Difficulty::Easy => 0.05,
            Difficulty::Normal => 0.1,
            Difficulty::Hard => 0.15, // 早く落ちるので連打が難しくなる
        };
        
        self.current_state = State::GameDive;
    }

    // --- ロジック更新処理 ---
    fn update(&mut self) {
        self.anim_frame += 0.15;

        if is_key_released(KeyCode::Space) {
            self.space_pressed = false;
        }

        match self.current_state {
            State::Title => {
                // 難易度切り替えの入力
                if is_key_pressed(KeyCode::E) { self.difficulty = Difficulty::Easy; }
                if is_key_pressed(KeyCode::N) { self.difficulty = Difficulty::Normal; }
                if is_key_pressed(KeyCode::H) { self.difficulty = Difficulty::Hard; }

                // ゲーム開始の入力
                if is_key_pressed(KeyCode::Key1) {
                    self.reset_swimming();
                }
                if is_key_pressed(KeyCode::Key2) {
                    self.reset_diving();
                }
            }
            State::GameSwim => {
                if !self.is_exhausted {
                    self.stamina += 0.3;
                }
                if self.stamina <= 0.0 {
                    self.is_exhausted = true;
                    self.stamina = 0.0;
                }
                if self.is_exhausted && self.stamina > 20.0 {
                    self.is_exhausted = false;
                }

                if is_key_pressed(KeyCode::Space) && !self.is_exhausted {
                    self.player_x += 15.0;
                    self.stamina -= 5.0;
                }

                // 難易度によってCPUの最高速度（乱数の範囲）を変更
                let mut rng = ::rand::thread_rng();
                let cpu_speed = match self.difficulty {
                    Difficulty::Easy => rng.gen_range(1..2) as f32,   // 遅い
                    Difficulty::Normal => rng.gen_range(1..3) as f32, // 普通
                    Difficulty::Hard => rng.gen_range(1..4) as f32,   // 早い！
                };
                self.cpu_x += cpu_speed;

                if (self.player_x as i32) % 25 < 5 {
                    self.ripples.push(Ripple::new(self.player_x, 280.0));
                }

                for ripple in &mut self.ripples {
                    ripple.update();
                }
                self.ripples.retain(|r| r.alpha > 0.0);

                if self.player_x >= self.goal_x {
                    self.current_state = State::Clear;
                } else if self.cpu_x >= self.goal_x {
                    self.current_state = State::GameOver;
                }
            }
            State::GameDive => {
                self.velocity_y += self.gravity;
                self.player_y += self.velocity_y;

                if is_key_pressed(KeyCode::Space) && !self.space_pressed {
                    self.rotations += 1;
                    self.space_pressed = true;
                }

                if self.player_y >= 500.0 {
                    if self.rotations >= 3 {
                        self.current_state = State::Clear;
                    } else {
                        self.current_state = State::GameOver;
                    }
                }
            }
            State::Clear | State::GameOver => {
                if is_key_pressed(KeyCode::Space) {
                    self.current_state = State::Title;
                }
            }
        }
    }

    // --- 描画処理 ---
    fn draw(&self) {
        clear_background(Color::from_rgba(200, 230, 255, 255));

        match self.current_state {
            State::Title => self.draw_title(),
            State::GameSwim => self.draw_swimming_game(),
            State::GameDive => self.draw_diving_game(),
            State::Clear => self.draw_result("CLEAR! GREAT JOB!", Color::from_rgba(50, 200, 50, 255)),
            State::GameOver => self.draw_result("GAME OVER... TRY AGAIN", Color::from_rgba(220, 50, 50, 255)),
        }
    }

    fn draw_title(&self) {
        draw_text("AQUA SPORTS SIM", 120.0, 160.0, 60.0, Color::from_rgba(0, 50, 150, 255));
        
        // 現在の難易度表示
        let diff_color = match self.difficulty {
            Difficulty::Easy => GREEN,
            Difficulty::Normal => ORANGE,
            Difficulty::Hard => RED,
        };
        draw_text(&format!("DIFFICULTY: {}", self.difficulty.to_string()), 150.0, 240.0, 30.0, diff_color);
        draw_text("Press [E] Easy / [N] Normal / [H] Hard", 150.0, 280.0, 20.0, DARKGRAY);

        // ゲーム選択
        draw_text("Press [1] for Swimming (Tap Space)", 150.0, 380.0, 30.0, BLACK);
        draw_text("Press [2] for Diving (Timing Space)", 150.0, 430.0, 30.0, BLACK);
    }

    fn draw_swimming_game(&self) {
        draw_rectangle(0.0, 200.0, self.screen_width, 200.0, Color::from_rgba(0, 105, 148, 255));
        draw_line(0.0, 300.0, self.screen_width, 300.0, 2.0, WHITE);
        draw_rectangle(self.goal_x, 200.0, 10.0, 200.0, RED);

        for r in &self.ripples {
            let color = Color::from_rgba(204, 230, 255, (r.alpha * 255.0) as u8);
            draw_circle_lines(r.x, r.y, r.width / 2.0, 2.0, color);
        }

        let player_color = if self.is_exhausted { GRAY } else { RED };
        self.draw_swimmer(self.player_x, 280.0, player_color, self.anim_frame);
        self.draw_swimmer(self.cpu_x, 340.0, YELLOW, self.anim_frame * 0.8);

        draw_text("STAMINA", 20.0, 50.0, 20.0, BLACK);
        let bar_color = if self.is_exhausted { ORANGE } else { GREEN };
        draw_rectangle(20.0, 60.0, self.stamina * 2.0, 20.0, bar_color);

        // プレイ中も右上に難易度を表示
        draw_text(&format!("MODE: {}", self.difficulty.to_string()), 650.0, 40.0, 20.0, DARKGRAY);
    }

    fn draw_diving_game(&self) {
        draw_rectangle(0.0, 500.0, self.screen_width, 100.0, Color::from_rgba(0, 191, 255, 255));
        draw_rectangle(350.0, 100.0, 100.0, 10.0, DARKGRAY);

        let rotation_rad = (self.rotations as f32 * 90.0).to_radians();
        draw_rectangle_ex(
            400.0,
            self.player_y,
            30.0,
            60.0,
            DrawRectangleParams {
                offset: vec2(0.5, 0.5), 
                rotation: rotation_rad,
                color: RED,
            },
        );

        draw_text(&format!("ROTATIONS: {}", self.rotations), 500.0, 100.0, 30.0, BLACK);
        // 右上に難易度を表示
        draw_text(&format!("MODE: {}", self.difficulty.to_string()), 650.0, 40.0, 20.0, DARKGRAY);
    }

    fn draw_result(&self, text: &str, color: Color) {
        draw_text(text, 50.0, 250.0, 50.0, color);
        draw_text("Press [SPACE] to Title", 250.0, 400.0, 30.0, BLACK);
    }

    fn draw_swimmer(&self, x: f32, y: f32, skin_color: Color, frame: f64) {
        draw_ellipse(x, y, 25.0, 12.5, 0.0, skin_color);

        let arm_y = (frame.sin() * 10.0) as f32;
        let mut darker_color = skin_color;
        darker_color.r *= 0.7;
        darker_color.g *= 0.7;
        darker_color.b *= 0.7;
        draw_ellipse(x + 15.0, y - 10.0 + arm_y, 10.0, 5.0, 0.0, darker_color);
    }
}

#[macroquad::main("Aqua Sports Simulator")]
async fn main() {
    request_new_screen_size(800.0, 600.0);

    let mut game = Game::new();

    loop {
        game.update();
        game.draw();

        next_frame().await
    }
}