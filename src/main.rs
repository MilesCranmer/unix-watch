#![allow(unused_features)]
use chrono::Local;
use gethostname::gethostname;
use std::ffi::{OsStr, OsString};
use std::os::unix::process::ExitStatusExt;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "watch",
    about = "Execute a command at a regular interval, showing the output fullscreen",
    usage = "watch [options] -- <command> [args...]"
)]
struct Opt {
    // Interval between updates
    #[structopt(
        short = "n",
        long = "interval",
        help = "Number of seconds between updates. Can be a floating point number.",
        default_value = "1"
    )]
    interval: u64,

    // Raw arguments
    #[structopt(
        raw(true),
        parse(from_os_str),
        required = true,
        min_values = 1,
        help = "Command to run"
    )]
    args: Vec<OsString>,
}

// Type which wraps the outcome of a programmatic parsing to
// numerically convert a UTF-8 string into a floating point number of seconds
// into a usable argument for [`Duration::from_millis`]()
type ParseResult = Result<u64, OsString>;

// This function provides a unified wrapper for generating a custom
// newtype over the unified [`Duration::from_secs`]() and [`Duration::from_millis`]()
fn parse_time(ts: impl AsRef<OsStr>) -> ParseResult {
    if let Some(it) = ts.as_ref().to_str() {
        match it.parse::<f32>() {
            Ok(ms) => {
                debug_assert!(ms >= 0.0, "Cannot check at negative intervals.");
                let sec = ms.floor() * 1_000_f32;
                let rem = (ms.fract() * 1_000_f32).floor();
                debug_assert!(rem < 1.0, "Parsing logic is flawed");
                debug_assert!(sec.fract() == rem.fract());
                let (sec, rem): (u64, u64) = (sec as u64, rem as u64);
                let millis = sec + rem;
                ParseResult::Ok(millis)
            }
            Err(e) => ParseResult::Err(format!("{e:?}").into()),
        }
    } else {
        ParseResult::Err("Character set not supported".into())
    }
}

fn main() {
    let opt = Opt::from_args();
    let cmd = opt.args[0].to_str().expect("Failed to parse command");
    let cmd_args = opt.args[1..]
        .iter()
        .map(|arg| arg.to_str().expect("Failed to parse command arguments"))
        .collect::<Vec<&str>>();

    let hostname = gethostname()
        .into_string()
        .unwrap_or_else(|_| "unknown".to_string());

    let (duration, r#int): (Duration, String) = if let Some(ms) = opt.sub_interval {
        (
            Duration::from_millis(ms),
            format!("{}ms", ms.to_string().as_str()),
        )
    } else {
        (
            Duration::from_secs(opt.interval),
            format!("{}s", opt.interval.to_string().as_str()),
        )
    };

    loop {
        let start_time = Instant::now();
        let output = Command::new(cmd)
            .args(&cmd_args)
            .output()
            .expect("Failed to execute command");

        let signal = output.status.stopped_signal();
        let return_code = output.status.code();

        // Clear screen:
        print!("\x1B[2J\x1B[1;1H");

        // Join cmd (str) and cmd_args (Vec<str>):
        let cmd_with_args = cmd_args
            .iter()
            .fold(cmd.to_string(), |acc, arg| acc + " " + arg);

        // Print what command we are running:
        println!(
            "Hostname: {}  Time: {}",
            hostname,
            Local::now().format("%Y-%m-%d %H:%M:%S")
        );
        println!("Every {}: {}", &r#int, cmd_with_args);
        println!();

        // Print output:
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if let Some(return_code) = return_code {
            if return_code != 0 {
                println!("watch command exited with return code: {}", return_code);
            }
        } else if let Some(signal) = signal {
            println!("watch command killed by signal: {}", signal);
        }
        println!("{}", stdout.trim_end());
        eprintln!("{}", stderr.trim_end());

        let elapsed_ms = start_time.elapsed().as_millis();
        if elapsed_ms < duration.as_millis() {
            let remaining = duration.as_millis() - elapsed_ms;
            sleep(Duration::from_millis(remaining as u64));
        }
    }
}
