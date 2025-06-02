use game_match::GameMatch;
use iced::{Element, Subscription, Task};

mod capture;
use capture::capture;

mod game_match;
mod home;
mod ocr;

fn main() {
    let init = || {
        let app = App {
            screen: Screen::Home(home::Home),
        };

        app
    };
    iced::application(init, App::update, App::view)
        .title("voidhunter")
        .subscription(App::subscribtion)
        .window_size((500.0, 500.0))
        .run()
        .unwrap();
}

#[derive(Debug, Clone)]
enum Message {
    Home(home::Message),
    GameMatch(game_match::Message),
}

pub enum Screen {
    Home(home::Home),
    GameMatch(GameMatch),
}

struct App {
    screen: Screen,
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        let task = match message {
            Message::Home(msg) => {
                if let Screen::Home(home) = &mut self.screen {
                    let action = home.update(msg);
                    match action {
                        home::Action::Run(task) => task.map(Message::Home),
                        home::Action::StartGame => {
                            let (screen, task) = GameMatch::new();
                            self.screen = Screen::GameMatch(screen);
                            task.map(Message::GameMatch)
                        }
                        home::Action::None => Task::none(),
                    }
                } else {
                    Task::none()
                }
            }
            Message::GameMatch(msg) => {
                if let Screen::GameMatch(game_match) = &mut self.screen {
                    let action = game_match.update(msg);

                    match action {
                        game_match::Action::Run(task) => task.map(Message::GameMatch),
                        game_match::Action::Home => {
                            self.screen = Screen::Home(home::Home);
                            Task::none()
                        }
                        game_match::Action::None => Task::none(),
                    }
                } else {
                    Task::none()
                }
            }
        };

        task
    }

    pub fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::GameMatch(gm) => gm.view().map(Message::GameMatch),
            Screen::Home(h) => h.view().map(Message::Home),
        }
    }

    pub fn subscribtion(&self) -> Subscription<Message> {
        let component_sub = match &self.screen {
            Screen::GameMatch(gm) => gm.subscribtion().map(Message::GameMatch),
            Screen::Home(_) => Subscription::none(),
        };

        component_sub
    }
}

#[derive(Debug, Clone)]
pub enum Stage {
    Pick,
    Run,
    Timer,
    Finished,
}
