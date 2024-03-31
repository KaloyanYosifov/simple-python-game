use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style,
    terminal::{self, Clear},
    QueueableCommand,
};
use std::{io::Stdout, panic};
use std::{
    io::{stdout, Write},
    time::Duration,
};

#[derive(Debug)]
struct GameObject {
    x: u16,
    y: u16,
}

struct Movement {
    x: i16,
    y: i16,
}

type Snake = Vec<GameObject>;

struct TerminalSize {
    w: u16,
    h: u16,
}

fn create_apple(game_time: u64, size: &TerminalSize) -> GameObject {
    let random_x = game_time % size.w as u64;
    let random_y = game_time % size.h as u64;

    GameObject {
        x: random_x as u16,
        y: random_y as u16,
    }
}

fn render_snake(
    snake: &mut Snake,
    stdout: &mut Stdout,
    movement: &Movement,
    game_time: u64,
    terminal_size: &TerminalSize,
) {
    let mut snake_head_indicator = '>';

    if game_time % 5 == 0 {
        let mut prev_x = snake.get(0).unwrap().x;
        let mut prev_y = snake.get(0).unwrap().y;

        for i in 1..snake.len() {
            let current_el = snake.get_mut(i).unwrap();
            (current_el.x, current_el.y, prev_x, prev_y) =
                (prev_x, prev_y, current_el.x, current_el.y);
        }

        let snake_head = snake.get_mut(0).unwrap();

        if snake_head.x == 0 && movement.x < 0 {
            snake_head.x = terminal_size.w;
        } else if snake_head.x >= terminal_size.w && movement.x > 0 {
            snake_head.x = 0;
        } else {
            if movement.x < 0 {
                snake_head.x -= u16::MAX - movement.x as u16 + 1;
            } else {
                snake_head.x += movement.x as u16;
            }
        }

        if snake_head.y == 0 && movement.y < 0 {
            snake_head.y = terminal_size.h;
        } else if snake_head.y >= terminal_size.h && movement.y > 0 {
            snake_head.y = 0;
        } else {
            if movement.y < 0 {
                snake_head.y -= u16::MAX - movement.y as u16 + 1;
            } else {
                snake_head.y += movement.y as u16;
            }
        }
    }

    if movement.y < 0 {
        snake_head_indicator = '^';
    } else if movement.y > 0 {
        snake_head_indicator = ',';
    } else if movement.x > 0 {
        snake_head_indicator = '>';
    } else if movement.x < 0 {
        snake_head_indicator = '<';
    }

    for (index, obj) in snake.iter().enumerate() {
        stdout.queue(cursor::MoveTo(obj.x, obj.y)).unwrap();

        if index == 0 {
            stdout.write(&[snake_head_indicator as u8]).unwrap();

            continue;
        }

        stdout.write("-".as_bytes()).unwrap();
    }
}

fn main() {
    terminal::enable_raw_mode().unwrap();
    let (width, height) = terminal::size().unwrap();
    let terminal_size = TerminalSize {
        w: width,
        h: height,
    };
    let mut stdout = stdout();
    let mut should_quit = false;
    let mut snake = vec![GameObject { x: 0, y: 0 }];
    let mut game_time: u64 = 0;
    let mut apple = create_apple(320, &terminal_size);
    let mut movement = Movement { x: 1, y: 1 };

    // Return to cooked mode when app panics
    panic::set_hook(Box::new(|e| {
        // Reenter canonical mode
        // TODO: exit raw mode
        terminal::disable_raw_mode().unwrap();
        // Print panic info
        eprintln!("{e}");
    }));

    while !should_quit {
        while event::poll(Duration::ZERO).unwrap() {
            match event::read().unwrap() {
                Event::Key(event) => match event.code {
                    KeyCode::Char(c) => {
                        if c == 'c' && event.modifiers.contains(event::KeyModifiers::CONTROL) {
                            should_quit = true;
                        }
                    }
                    KeyCode::Up => {
                        movement.y = -1;
                        movement.x = 0;
                    }
                    KeyCode::Down => {
                        movement.y = 1;
                        movement.x = 0;
                    }
                    KeyCode::Left => {
                        movement.x = -1;
                        movement.y = 0;
                    }
                    KeyCode::Right => {
                        movement.x = 1;
                        movement.y = 0;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        stdout.queue(Clear(terminal::ClearType::All)).unwrap();

        render_snake(
            &mut snake,
            &mut stdout,
            &movement,
            game_time,
            &terminal_size,
        );
        let snake_head = snake.get(0).unwrap();

        if snake_head.x == apple.x && snake_head.y == apple.y {
            apple = create_apple(game_time, &terminal_size);
            let snake_tail = snake.last().unwrap();
            let (mut new_tail_x, mut new_tail_y) = (snake_tail.x, snake_tail.y);

            if movement.x > 0 {
                new_tail_x -= 1;
            } else if movement.x < 0 {
                new_tail_x += 1;
            } else if movement.y > 0 {
                new_tail_y -= 1;
            } else if movement.y < 0 {
                new_tail_y += 1;
            }

            snake.push(GameObject {
                x: new_tail_x,
                y: new_tail_y,
            });
        }

        stdout.queue(cursor::MoveTo(apple.x, apple.y)).unwrap();
        stdout.write("@".as_bytes()).unwrap();

        stdout.flush().unwrap();
        std::thread::sleep(Duration::from_millis(16)); // 60 FPS = 16 millis sleep
        game_time += 16;
    }

    terminal::disable_raw_mode().unwrap();
}
