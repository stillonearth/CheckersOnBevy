use std::cell::{RefCell, RefMut};
use std::sync::Arc;

fn main() {
    let game: Arc<RefCell<Game>> = Arc::new(RefCell::<Game>::new(Game { value: 5 }));

    let app = Arc::new(RefCell::<App>::new(App {
        game: Arc::clone(&game),
    }));

    let env = Env {
        app: Arc::clone(&app),
        game: Arc::clone(&game),
    };

    // Here's what I want
    // // 1. A game loop starts
    // // 2. User interacts with bevy app
    // // 3. App invokes self.game.action() which changes &game state
    // {
    //     app.game_loop();
    // }
    // // At the same time
    // // Game state is changed from outside
    // {
    //     env.game.borrow_mut().action();
    // }

    app.borrow_mut().game_loop();
    println!("{}", game.borrow().value);

    env.game.borrow_mut().action();
    env.game.borrow_mut().action();
    env.game.borrow_mut().action();

    println!("{}", game.borrow().value);

    println!("hej");
}

#[derive(Debug, Copy, Clone)]
struct Game {
    value: i8,
}

impl Game {
    pub fn action(&mut self) {
        self.value += 1;
    }
}

struct App {
    game: Arc<RefCell<Game>>,
}

impl App {
    pub fn game_loop(&mut self) {
        // something happens here
        self.game.borrow_mut().action();
    }
}

struct Env {
    pub game: Arc<RefCell<Game>>,
    pub app: Arc<RefCell<App>>,
}
