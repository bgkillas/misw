use crossterm::{
    cursor, event,
    event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind},
    terminal, ExecutableCommand,
};
use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};
use std::{
    env::args,
    io::{stdout, Write},
    process::exit,
    time::Instant,
};
#[derive(Clone, Copy, PartialEq)]
enum Point
{
    Bomb,
    Flag,
    BombFlag,
    Open,
    Close,
}
fn main()
{
    let mut stdout = stdout().lock();
    terminal::enable_raw_mode().unwrap();
    stdout.execute(cursor::Hide).unwrap();
    stdout.execute(event::EnableMouseCapture).unwrap();
    stdout.execute(terminal::EnterAlternateScreen).unwrap();
    let mut args = args().collect::<Vec<String>>();
    args.remove(0);
    let touch = !args.is_empty() && args[0] == "touch";
    if touch
    {
        args.remove(0);
    }
    let xb = if args.is_empty()
    {
        16
    }
    else if args[0] == "max"
    {
        get_terminal_dimensions().0 / 3
    }
    else
    {
        args[0].parse::<usize>().unwrap_or(16)
    };
    let yb = if !args.is_empty() && args[0] == "max"
    {
        get_terminal_dimensions().1 - 1
    }
    else if args.len() <= 1
    {
        16
    }
    else
    {
        args[1].parse::<usize>().unwrap_or(16)
    };
    let bombs = (xb * yb)
        / if args.len() > 1 && args[0] == "max"
        {
            args[1].parse::<usize>().unwrap_or(4)
        }
        else if args.len() <= 2
        {
            4
        }
        else
        {
            args[2].parse::<usize>().unwrap_or(4)
        };
    println!("{} {} {}", xb, yb, bombs);
    let min = u64::MAX / ((xb * yb) as u64 / bombs as u64);
    'main: loop
    {
        print!("\x1b[H\x1b[J");
        let mut board = vec![vec![Point::Close; yb]; xb];
        let mut xcount = 0;
        let mut even = false;
        let mut rbombs = 0;
        let mut flags = 0;
        for row in board.iter_mut()
        {
            for point in row.iter_mut()
            {
                if min > fastrand::u64(..)
                {
                    *point = Point::Bomb;
                    rbombs += 1;
                }
                if even
                {
                    even = false;
                    print!("\x1b[46m   \x1b[0m");
                }
                else
                {
                    even = true;
                    print!("\x1b[106m   \x1b[0m");
                }
                xcount += 1;
                if xcount == xb
                {
                    if xb % 2 == 0
                    {
                        even = !even;
                    }
                    print!("\x1b[B\x1b[G");
                    xcount = 0;
                }
            }
        }
        let timer;
        stdout.flush().unwrap();
        loop
        {
            let (x, y, _, _) = read_input(touch);
            if x < xb && y < yb
            {
                timer = Instant::now();
                for x in x.saturating_sub(1)..=(x + 1).min(xb - 1)
                {
                    for y in y.saturating_sub(1)..=(y + 1).min(yb - 1)
                    {
                        if board[x][y] == Point::Bomb
                        {
                            board[x][y] = Point::Open;
                            rbombs -= 1;
                        }
                    }
                }
                clear(&mut board, x, y, xb, yb, &mut Vec::new());
                stdout.flush().unwrap();
                break;
            }
        }
        print_info(timer, flags, rbombs);
        stdout.flush().unwrap();
        loop
        {
            let (x, y, mb, restart) = read_input(touch);
            if restart
            {
                continue 'main;
            }
            if x < xb && y < yb
            {
                match mb
                {
                    MouseButton::Middle if board[x][y] == Point::Open =>
                    {
                        let mut flags = 0;
                        let mut bombs = 0;
                        for board in &board[x.saturating_sub(1)..=(x + 1).min(xb - 1)]
                        {
                            for i in &board[y.saturating_sub(1)..=(y + 1).min(yb - 1)]
                            {
                                if *i == Point::Bomb
                                {
                                    bombs += 1;
                                }
                                else if *i == Point::Flag
                                {
                                    flags += 1;
                                }
                            }
                        }
                        if flags == bombs
                        {
                            if bombs > 0
                            {
                                break;
                            }
                            for x in x.saturating_sub(1)..=(x + 1).min(xb - 1)
                            {
                                for y in y.saturating_sub(1)..=(y + 1).min(yb - 1)
                                {
                                    if board[x][y] != Point::BombFlag
                                    {
                                        clear(&mut board, x, y, xb, yb, &mut Vec::new())
                                    }
                                }
                            }
                        }
                        stdout.flush().unwrap();
                    }
                    MouseButton::Right => match board[x][y]
                    {
                        Point::Bomb =>
                        {
                            board[x][y] = Point::BombFlag;
                            flags += 1;
                            flag(x, y);
                            stdout.flush().unwrap();
                        }
                        Point::Close =>
                        {
                            board[x][y] = Point::Flag;
                            flags += 1;
                            flag(x, y);
                            stdout.flush().unwrap();
                        }
                        Point::Flag =>
                        {
                            board[x][y] = Point::Close;
                            flags -= 1;
                            unflag(x, y);
                            stdout.flush().unwrap();
                        }
                        Point::BombFlag =>
                        {
                            board[x][y] = Point::Bomb;
                            flags -= 1;
                            unflag(x, y);
                            stdout.flush().unwrap();
                        }
                        _ =>
                        {}
                    },
                    MouseButton::Left => match board[x][y]
                    {
                        Point::Bomb =>
                        {
                            break;
                        }
                        Point::Close =>
                        {
                            clear(&mut board, x, y, xb, yb, &mut Vec::new());
                            stdout.flush().unwrap()
                        }
                        Point::Open =>
                        {
                            let mut flags = 0;
                            let mut bombs = 0;
                            for board in &board[x.saturating_sub(1)..=(x + 1).min(xb - 1)]
                            {
                                for i in &board[y.saturating_sub(1)..=(y + 1).min(yb - 1)]
                                {
                                    if *i == Point::Bomb
                                    {
                                        bombs += 1;
                                    }
                                    else if *i == Point::Flag
                                    {
                                        flags += 1;
                                    }
                                }
                            }
                            if flags == bombs
                            {
                                if bombs > 0
                                {
                                    break;
                                }
                                for x in x.saturating_sub(1)..=(x + 1).min(xb - 1)
                                {
                                    for y in y.saturating_sub(1)..=(y + 1).min(yb - 1)
                                    {
                                        if board[x][y] != Point::BombFlag
                                        {
                                            clear(&mut board, x, y, xb, yb, &mut Vec::new())
                                        }
                                    }
                                }
                            }
                            stdout.flush().unwrap();
                        }
                        _ =>
                        {}
                    },
                    _ =>
                    {}
                }
            }
            print_info(timer, flags, rbombs);
            stdout.flush().unwrap();
        }
    }
}
fn flag(x: usize, y: usize)
{
    print!(
        "\x1b[H{}{}{}",
        if x == 0
        {
            String::new()
        }
        else
        {
            "\x1b[".to_owned() + &(x * 3).to_string() + "C"
        },
        if y == 0
        {
            String::new()
        }
        else
        {
            "\x1b[".to_owned() + &y.to_string() + "B"
        },
        if (y % 2 == 0) == (x % 2 == 0)
        {
            "\x1b[101m   \x1b[0m"
        }
        else
        {
            "\x1b[41m   \x1b[0m"
        }
    );
}
fn unflag(x: usize, y: usize)
{
    print!(
        "\x1b[H{}{}{}",
        if x == 0
        {
            String::new()
        }
        else
        {
            "\x1b[".to_owned() + &(x * 3).to_string() + "C"
        },
        if y == 0
        {
            String::new()
        }
        else
        {
            "\x1b[".to_owned() + &y.to_string() + "B"
        },
        if (y % 2 == 0) == (x % 2 == 0)
        {
            "\x1b[106m   \x1b[0m"
        }
        else
        {
            "\x1b[46m   \x1b[0m"
        }
    );
}
fn clear(
    board: &mut Vec<Vec<Point>>,
    x: usize,
    y: usize,
    xb: usize,
    yb: usize,
    blacklist: &mut Vec<(usize, usize)>,
)
{
    board[x][y] = Point::Open;
    let mut sum = 0;
    for board in &board[x.saturating_sub(1)..=(x + 1).min(xb - 1)]
    {
        for i in &board[y.saturating_sub(1)..=(y + 1).min(yb - 1)]
        {
            if *i == Point::Bomb || *i == Point::BombFlag
            {
                sum += 1;
            }
        }
    }
    if sum == 0
    {
        print!(
            "\x1b[H{}{}{}",
            if x == 0
            {
                String::new()
            }
            else
            {
                "\x1b[".to_owned() + &(x * 3).to_string() + "C"
            },
            if y == 0
            {
                String::new()
            }
            else
            {
                "\x1b[".to_owned() + &y.to_string() + "B"
            },
            if (y % 2 == 0) == (x % 2 == 0)
            {
                "\x1b[105m   \x1b[0m"
            }
            else
            {
                "\x1b[45m   \x1b[0m"
            }
        );
        blacklist.push((x, y));
        for x in x.saturating_sub(1)..=(x + 1).min(xb - 1)
        {
            for y in y.saturating_sub(1)..=(y + 1).min(yb - 1)
            {
                if !blacklist.iter().any(|b| b.0 == x && b.1 == y)
                {
                    clear(board, x, y, xb, yb, blacklist);
                }
            }
        }
    }
    else
    {
        print!(
            "\x1b[H{}{}{} {} \x1b[0m",
            if x == 0
            {
                String::new()
            }
            else
            {
                "\x1b[".to_owned() + &(x * 3).to_string() + "C"
            },
            if y == 0
            {
                String::new()
            }
            else
            {
                "\x1b[".to_owned() + &y.to_string() + "B"
            },
            if (y % 2 == 0) == (x % 2 == 0)
            {
                "\x1b[105m"
            }
            else
            {
                "\x1b[45m"
            },
            sum
        );
    }
}
fn print_info(timer: Instant, flags: usize, bombs: usize)
{
    print!(
        "\x1b[{}B\x1b[G{:02}:{:02}\t{}/{}",
        get_terminal_dimensions().1,
        timer.elapsed().as_secs() / 60,
        timer.elapsed().as_secs() % 60,
        flags,
        bombs
    );
}
fn read_input(touch: bool) -> (usize, usize, MouseButton, bool)
{
    if touch
    {
        loop
        {
            match event::read().unwrap()
            {
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(_),
                    ..
                })
                | Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Drag(_),
                    ..
                }) =>
                {
                    let timer = Instant::now();
                    loop
                    {
                        match event::read().unwrap()
                        {
                            Event::Mouse(MouseEvent {
                                kind: MouseEventKind::Up(_),
                                column,
                                row,
                                ..
                            }) =>
                            {
                                return (
                                    column as usize / 3,
                                    row as usize,
                                    if timer.elapsed().as_millis() > 100
                                    {
                                        MouseButton::Right
                                    }
                                    else
                                    {
                                        MouseButton::Left
                                    },
                                    false,
                                );
                            }
                            _ =>
                            {}
                        }
                    }
                }
                _ =>
                {}
            }
        }
    }
    else
    {
        loop
        {
            match event::read().unwrap()
            {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) =>
                {
                    terminal::disable_raw_mode().unwrap();
                    stdout().execute(cursor::Show).unwrap();
                    stdout().execute(event::DisableMouseCapture).unwrap();
                    stdout().execute(terminal::LeaveAlternateScreen).unwrap();
                    exit(0);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('r'),
                    ..
                }) => return (0, 0, MouseButton::Left, true),
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(kind),
                    column,
                    row,
                    ..
                }) =>
                {
                    return (column as usize / 3, row as usize, kind, false);
                }
                _ =>
                {}
            }
        }
    }
}
pub fn get_terminal_dimensions() -> (usize, usize)
{
    unsafe {
        let mut size: winsize = std::mem::zeroed();
        if ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size) == 0 && size.ws_col != 0
        {
            (size.ws_col as usize, size.ws_row as usize)
        }
        else
        {
            (80, 80)
        }
    }
}