use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use rand::seq::SliceRandom;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

const WORD_POOL: &[&str] = &[
    "the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog",
    "pack", "my", "box", "with", "five", "dozen", "liquor", "jugs",
    "rust", "is", "a", "systems", "programming", "language", "focused",
    "on", "safety", "and", "speed", "terminal", "applications", "are",
    "surprisingly", "fun", "to", "build", "from", "scratch", "every",
    "keystroke", "brings", "you", "closer", "mastery", "keyboard",
    "precision", "twin", "virtues", "expert", "typist", "best", "way",
    "learn", "typing", "practice", "open", "source", "software", "powers",
    "modern", "world", "countless", "ways", "fast", "thinks", "words",
    "not", "individual", "letters", "linux", "gives", "freedom",
    "customize", "everything", "see", "command", "line", "most",
    "powerful", "interface", "ever", "created", "great", "written",
    "one", "careful", "code", "time", "into", "new", "old", "deep",
    "light", "dark", "high", "low", "each", "last", "next", "first",
    "long", "short", "hard", "easy", "more", "less", "keep", "find",
    "make", "take", "give", "come", "know", "think", "look", "want",
    "use", "work", "call", "try", "ask", "need", "feel", "become",
    "leave", "put", "mean", "keep", "let", "begin", "show", "hear",
    "play", "run", "move", "live", "believe", "hold", "bring", "happen",
    "write", "provide", "stand", "lose", "pay", "meet", "include",
    "continue", "set", "change", "lead", "understand", "watch", "follow",
    "stop", "create", "speak", "read", "spend", "grow", "open", "walk",
    "win", "offer", "remember", "love", "consider", "appear", "buy",
    "wait", "serve", "send", "expect", "build", "stay", "fall", "cut",
    "reach", "kill", "remain", "suggest", "raise", "pass", "sell",
    "require", "report", "decide", "pull", "break", "act", "point",
];

fn generate_prompt(word_count: usize, rng: &mut impl rand::Rng) -> String {
    let mut words: Vec<&str> = WORD_POOL.choose_multiple(rng, word_count).cloned().collect();

    words.shuffle(rng);
    words.join(" ")
}

struct State {
    prompt: Vec<char>,
    typed: Vec<char>,
    start_time: Option<Instant>,
    done: bool,
    word_count: usize,
}

impl State {
    fn new(prompt: String, word_count: usize) -> Self {
        Self {
            prompt: prompt.chars().collect(),
            typed: Vec::new(),
            start_time: None,
            done: false,
            word_count,
        }
    }

    fn wpm(&self) -> f64 {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs_f64() / 60.0;
            if elapsed > 0.0 {
                let words = self.typed.len() as f64 / 5.0;
                return words / elapsed;
            }
        }
        0.0
    }

    fn accuracy(&self) -> f64 {
        if self.typed.is_empty() {
            return 100.0;
        }
        let correct = self
            .typed
            .iter()
            .zip(self.prompt.iter())
            .filter(|(t, p)| t == p)
            .count();
        (correct as f64 / self.typed.len() as f64) * 100.0
    }
}


fn wrap_prompt(chars: &[char], width: usize) -> Vec<Vec<char>> {
    let text: String = chars.iter().collect();
    let mut lines: Vec<Vec<char>> = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line.chars().collect());
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line.chars().collect());
    }
    lines
}

fn draw_menu(selected: usize, custom_input: &str, custom_mode: bool) -> std::io::Result<()> {
    let mut out = stdout();
    let options = ["10 words", "25 words", "50 words", "custom"];

    queue!(out, terminal::Clear(ClearType::All))?;

    queue!(out, cursor::MoveTo(2, 1), SetForegroundColor(Color::Cyan),
        Print("+-----------------------------------------+"))?;
    queue!(out, cursor::MoveTo(2, 2),
        Print("|           typer -- wpm tester           |"))?;
    queue!(out, cursor::MoveTo(2, 3),
        Print("+-----------------------------------------+"), ResetColor)?;

    queue!(out, cursor::MoveTo(2, 5),
        SetForegroundColor(Color::DarkGrey),
        Print("select word count:"),
        ResetColor)?;

    for (i, opt) in options.iter().enumerate() {
        let row = 7 + i as u16;
        if i == selected {
            queue!(out, cursor::MoveTo(2, row),
                SetForegroundColor(Color::Yellow),
                Print(format!("> {}", opt)),
                ResetColor)?;
        } else {
            queue!(out, cursor::MoveTo(2, row),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("  {}", opt)),
                ResetColor)?;
        }
    }

    if custom_mode {
        queue!(out, cursor::MoveTo(2, 12),
            SetForegroundColor(Color::Cyan),
            Print("enter word count: "),
            SetForegroundColor(Color::White),
            Print(custom_input),
            Print("_"),
            ResetColor)?;
        queue!(out, cursor::MoveTo(2, 14),
            SetForegroundColor(Color::DarkGrey),
            Print("enter to confirm  |  esc to cancel"),
            ResetColor)?;
    } else {
        queue!(out, cursor::MoveTo(2, 12),
            SetForegroundColor(Color::DarkGrey),
            Print("up/down to select  |  enter to start  |  q to quit"),
            ResetColor)?;
    }

    out.flush()?;
    Ok(())
}

fn draw(state: &State) -> std::io::Result<()> {
    let mut out = stdout();
    let prompt_width = 45usize;
    let lines = wrap_prompt(&state.prompt, prompt_width);


    let typed_len = state.typed.len();
    let mut char_index = 0usize;
    let mut cursor_line = 0usize;
    let mut cursor_col = 0usize;

    'outer: for (li, line) in lines.iter().enumerate() {
        for (ci, _) in line.iter().enumerate() {
            if char_index == typed_len {
                cursor_line = li;
                cursor_col = ci;
                break 'outer;
            }
            char_index += 1;
        }

        if char_index == typed_len {
            cursor_line = li + 1;
            cursor_col = 0;
            break;
        }

        char_index += 1;
    }
    let _ = (cursor_line, cursor_col);

    queue!(out, terminal::Clear(ClearType::All))?;

    queue!(out, cursor::MoveTo(2, 1), SetForegroundColor(Color::Cyan),
        Print("+-----------------------------------------+"))?;
    queue!(out, cursor::MoveTo(2, 2),
        Print(format!("|        typer -- {} words{:>16}|",
            state.word_count, " ")))?;
    queue!(out, cursor::MoveTo(2, 3),
        Print("+-----------------------------------------+"), ResetColor)?;


    let mut global_idx = 0usize;
    for (li, line) in lines.iter().enumerate() {
        queue!(out, cursor::MoveTo(2, 5 + li as u16))?;
        for ch in line.iter() {
            if global_idx < state.typed.len() {
                if state.typed[global_idx] == *ch {
                    queue!(out, SetForegroundColor(Color::Green), Print(ch), ResetColor)?;
                } else {
                    queue!(out, SetForegroundColor(Color::Red), Print(ch), ResetColor)?;
                }
            } else if global_idx == state.typed.len() {
                queue!(out, SetForegroundColor(Color::Yellow), Print(ch), ResetColor)?;
            } else {
                queue!(out, SetForegroundColor(Color::DarkGrey), Print(ch), ResetColor)?;
            }
            global_idx += 1;
        }
        global_idx += 1;
    }

    let stats_row = 5 + lines.len() as u16 + 1;

    let wpm = state.wpm();
    let acc = state.accuracy();
    let progress = if state.prompt.is_empty() {
        0
    } else {
        (state.typed.len() * 20 / state.prompt.len()).min(20)
    };
    let bar: String = "#".repeat(progress) + &".".repeat(20 - progress);

    queue!(out, cursor::MoveTo(2, stats_row),
        SetForegroundColor(Color::Cyan), Print("wpm: "),
        SetForegroundColor(Color::White), Print(format!("{:5.1}  ", wpm)),
        SetForegroundColor(Color::Cyan), Print("acc: "),
        SetForegroundColor(Color::White), Print(format!("{:5.1}%  ", acc)),
        SetForegroundColor(Color::DarkGrey), Print(format!("[{}]", bar)),
        ResetColor)?;

    queue!(out, cursor::MoveTo(2, stats_row + 2),
        SetForegroundColor(Color::DarkGrey),
        Print("ctrl+c to quit  |  ctrl+r to restart"),
        ResetColor)?;

    out.flush()?;
    Ok(())
}

fn draw_results(state: &State) -> std::io::Result<()> {
    let mut out = stdout();
    let wpm = state.wpm();
    let acc = state.accuracy();

    let correct: usize = state
        .typed
        .iter()
        .zip(state.prompt.iter())
        .filter(|(t, p)| t == p)
        .count();
    let errors = state.typed.len().saturating_sub(correct);

    let elapsed = state.start_time.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0);

    queue!(out, terminal::Clear(ClearType::All))?;

    queue!(out, cursor::MoveTo(2, 1), SetForegroundColor(Color::Cyan),
        Print("+-----------------------------------------+"))?;
    queue!(out, cursor::MoveTo(2, 2),
        Print("|                 results                 |"))?;
    queue!(out, cursor::MoveTo(2, 3),
        Print("+-----------------------------------------+"), ResetColor)?;

    let wpm_color = if wpm >= 60.0 { Color::Green } else if wpm >= 35.0 { Color::Yellow } else { Color::Red };
    let acc_color = if acc >= 95.0 { Color::Green } else if acc >= 80.0 { Color::Yellow } else { Color::Red };

    queue!(out, cursor::MoveTo(2, 5),
        SetForegroundColor(Color::DarkGrey), Print("speed      "),
        SetForegroundColor(wpm_color), Print(format!("{:.1} wpm", wpm)))?;

    queue!(out, cursor::MoveTo(2, 7),
        SetForegroundColor(Color::DarkGrey), Print("accuracy   "),
        SetForegroundColor(acc_color), Print(format!("{:.1}%", acc)))?;

    queue!(out, cursor::MoveTo(2, 9),
        SetForegroundColor(Color::DarkGrey), Print("errors     "),
        SetForegroundColor(Color::White), Print(format!("{}", errors)))?;

    queue!(out, cursor::MoveTo(2, 11),
        SetForegroundColor(Color::DarkGrey), Print("time       "),
        SetForegroundColor(Color::White), Print(format!("{:.1}s", elapsed)))?;

    queue!(out, cursor::MoveTo(2, 13),
        SetForegroundColor(Color::DarkGrey),
        Print("r = retry  |  n = new prompt  |  m = menu  |  q = quit"),
        ResetColor)?;

    out.flush()?;
    Ok(())
}

enum Screen {
    Menu,
    Typing,
    Results,
}

fn run() -> std::io::Result<()> {
    let mut rng = rand::thread_rng();
    let mut screen = Screen::Menu;
    let mut selected = 0usize;
    let mut custom_mode = false;
    let mut custom_input = String::new();
    let mut word_count = 25usize;
    let mut state = State::new(generate_prompt(word_count, &mut rng), word_count);

    terminal::enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, terminal::EnterAlternateScreen, cursor::Hide)?;

    draw_menu(selected, &custom_input, custom_mode)?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {


                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                    break;
                }

                match screen {
                    Screen::Menu => {
                        if custom_mode {
                            match code {
                                KeyCode::Char(c) if c.is_ascii_digit() => {
                                    custom_input.push(c);
                                    draw_menu(selected, &custom_input, custom_mode)?;
                                }
                                KeyCode::Backspace => {
                                    custom_input.pop();
                                    draw_menu(selected, &custom_input, custom_mode)?;
                                }
                                KeyCode::Enter => {
                                    if let Ok(n) = custom_input.trim().parse::<usize>() {
                                        if n > 0 && n <= 200 {
                                            word_count = n;
                                            state = State::new(generate_prompt(word_count, &mut rng), word_count);
                                            screen = Screen::Typing;
                                            draw(&state)?;
                                        }
                                    }
                                    custom_mode = false;
                                    custom_input.clear();
                                }
                                KeyCode::Esc => {
                                    custom_mode = false;
                                    custom_input.clear();
                                    draw_menu(selected, &custom_input, custom_mode)?;
                                }
                                _ => {}
                            }
                        } else {
                            match code {
                                KeyCode::Up => {
                                    if selected > 0 { selected -= 1; }
                                    draw_menu(selected, &custom_input, custom_mode)?;
                                }
                                KeyCode::Down => {
                                    if selected < 3 { selected += 1; }
                                    draw_menu(selected, &custom_input, custom_mode)?;
                                }
                                KeyCode::Enter => {
                                    match selected {
                                        0 => { word_count = 10; }
                                        1 => { word_count = 25; }
                                        2 => { word_count = 50; }
                                        3 => {
                                            custom_mode = true;
                                            draw_menu(selected, &custom_input, custom_mode)?;
                                            continue;
                                        }
                                        _ => {}
                                    }
                                    state = State::new(generate_prompt(word_count, &mut rng), word_count);
                                    screen = Screen::Typing;
                                    draw(&state)?;
                                }
                                KeyCode::Char('q') => break,
                                _ => {}
                            }
                        }
                    }

                    Screen::Typing => {
                        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('r') {
                            state = State::new(generate_prompt(word_count, &mut rng), word_count);
                            draw(&state)?;
                            continue;
                        }
                        match code {
                            KeyCode::Char(c) => {
                                if state.start_time.is_none() {
                                    state.start_time = Some(Instant::now());
                                }
                                state.typed.push(c);
                                if state.typed.len() == state.prompt.len() {
                                    state.done = true;
                                    screen = Screen::Results;
                                    draw_results(&state)?;
                                } else {
                                    draw(&state)?;
                                }
                            }
                            KeyCode::Backspace => {
                                state.typed.pop();
                                draw(&state)?;
                            }
                            _ => {}
                        }
                    }

                    Screen::Results => {
                        match code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('r') => {
                                state = State::new(generate_prompt(word_count, &mut rng), word_count);
                                screen = Screen::Typing;
                                draw(&state)?;
                            }
                            KeyCode::Char('n') => {
                                state = State::new(generate_prompt(word_count, &mut rng), word_count);
                                screen = Screen::Typing;
                                draw(&state)?;
                            }
                            KeyCode::Char('m') => {
                                screen = Screen::Menu;
                                draw_menu(selected, &custom_input, custom_mode)?;
                            }
                            _ => {}
                        }
                    }
                }
            }
        } else if matches!(screen, Screen::Typing) && !state.done && state.start_time.is_some() {
            draw(&state)?;
        }
    }

    execute!(out, terminal::LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
