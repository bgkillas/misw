//for touch maybe only support left click, click numbers to remove sorrounding space besides first touch, click unknown for flag, maybe * for flag
//make menu for deciding starting conditions
//timer
//100 by 80 max, maybe calculate max
use crossterm::{
    cursor, event,
    event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind},
    terminal, ExecutableCommand,
};
use std::{
    env::args,
    io::{stdout, Write},
    process::exit,
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
    let xb = if args.is_empty()
    {
        16
    }
    else
    {
        args[0].parse::<usize>().unwrap_or(16)
    };
    let yb = if args.len() <= 1
    {
        16
    }
    else
    {
        args[1].parse::<usize>().unwrap_or(16)
    };
    let bombs = (xb * yb)
        / if args.len() <= 2
        {
            4
        }
        else
        {
            args[2].parse::<usize>().unwrap_or(4)
        };
    let min = u64::MAX / ((xb * yb) as u64 / bombs as u64);
    'main: loop
    {
        print!("\x1b[H\x1b[J");
        let mut board = vec![vec![Point::Close; yb]; xb];
        let mut xcount = 0;
        let mut even = false;
        for row in board.iter_mut()
        {
            for point in row.iter_mut()
            {
                if min > fastrand::u64(..)
                {
                    *point = Point::Bomb;
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
        stdout.flush().unwrap();
        loop
        {
            let (x, y, _, restart) = read_input();
            if restart
            {
                continue 'main;
            }
            if x < xb && y < yb
            {
                for x in x.saturating_sub(1)..=(x + 1).min(xb - 1)
                {
                    for y in y.saturating_sub(1)..=(y + 1).min(yb - 1)
                    {
                        board[x][y] = Point::Open;
                    }
                }
                clear(&mut board, x, y, xb, yb, &mut Vec::new());
                stdout.flush().unwrap();
                break;
            }
        }
        loop
        {
            let (x, y, mb, restart) = read_input();
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
                            flag(x, y);
                            stdout.flush().unwrap();
                        }
                        Point::Close =>
                        {
                            board[x][y] = Point::Flag;
                            flag(x, y);
                            stdout.flush().unwrap();
                        }
                        Point::Flag =>
                        {
                            board[x][y] = Point::Close;
                            unflag(x, y);
                            stdout.flush().unwrap();
                        }
                        Point::BombFlag =>
                        {
                            board[x][y] = Point::Bomb;
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
                        _ =>
                        {}
                    },
                    _ =>
                    {}
                }
            }
        }
    }
}
fn flag(x: usize, y: usize)
{
    print!(
        "\x1b[H{}{}\x1b[41m   \x1b[0m",
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
fn read_input() -> (usize, usize, MouseButton, bool)
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