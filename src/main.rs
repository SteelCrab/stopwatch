use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{self, Result, Write, stdout};
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
enum State {
    Running,
    Stopped,
}

struct Stopwatch {
    state: State,
    elapsed: Duration,
    start_time: Option<Instant>,
    counter: u32,
}

impl Stopwatch {
    fn new() -> Self {
        Stopwatch {
            state: State::Stopped,
            elapsed: Duration::ZERO,
            start_time: None,
            counter: 0,
        }
    }

    fn current_time(&self) -> Duration {
        match self.state {
            State::Running => self.elapsed + self.start_time.unwrap().elapsed(),
            State::Stopped => self.elapsed,
        }
    }

    fn counter_increment(&mut self) {
        self.counter += 1;
    }

    fn reset_counter(&mut self) {
        self.counter = 0;
    }

    fn reset_time(&mut self) {
        self.elapsed = Duration::ZERO;
    }

    fn toggle(&mut self) {
        self.state = match self.state {
            State::Running => {
                self.elapsed += self.start_time.unwrap().elapsed();
                println!("\r⏸️  정지 | ⛳️ {}\r", self.counter);
                stdout().flush().unwrap();
                State::Stopped
            }
            State::Stopped => {
                self.start_time = Some(Instant::now());
                println!("▶️  시작\r");
                stdout().flush().unwrap();
                self.counter_increment();
                State::Running
            }
        };
    }

    fn display(&self) {
        let time = self.current_time();
        print!(
            "\r⏱️ {:02}:{:02}:{:02}.{:02}  ",
            time.as_secs() / 3600,
            time.as_secs() / 60,
            time.as_secs() % 60,
            time.subsec_millis() / 10,
        );
        stdout().flush().unwrap();
    }

    fn is_lid_closed(&self) -> Result<bool> {
        let cmd = Command::new("ioreg")
            .args(["-r", "-k", "AppleClamshellState", "-d", "4"])
            .output()?;
        let cmdout = String::from_utf8_lossy(&cmd.stdout);
        Ok(cmdout.contains("AppleClamshellState\" = Yes"))
    }
    fn run(&mut self) -> Result<()> {
        println!("\r\n🕐 스톱워치\r");
        println!("\r[Enter] 시작/정지  [r] 리셋  [Esc] 종료\r\n");
        stdout().flush()?;

        loop {
            if self.state == State::Running {
                self.display();
                if self.is_lid_closed()? {
                    println!("\r\n💤 잠금 감지 - 정지\r");
                    self.toggle();
                }
            }

            if event::poll(Duration::from_millis(10))? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind != KeyEventKind::Press {
                        continue;
                    }

                    match key_event.code {
                        KeyCode::Enter => {
                            println!();
                            self.toggle();
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            self.reset_counter();
                            self.reset_time();
                            self.state = State::Stopped;
                            println!("\r\n🔄 리셋\r");
                            stdout().flush()?;
                        }
                        KeyCode::Esc => {
                            println!("\r\n🚪 종료\r");
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    enable_raw_mode().unwrap();

    let mut stopwatch = Stopwatch::new();
    let result = stopwatch.run();

    disable_raw_mode()?;

    result
}
