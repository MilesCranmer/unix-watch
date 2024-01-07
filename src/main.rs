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
        parse(from_os_str = parse_time_as_s_to_ms),
        help = "Number of seconds between updates. Can be a floating point number, will be rounded to the nearest millisecond.",
        default_value = "1"
    )]
    interval: ParseResult,

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
fn parse_time_as_s_to_ms(ts: impl AsRef<OsStr>) -> ParseResult {
    if let Some(it) = ts.as_ref().to_str() {
        match it.parse::<f64>() {
            Ok(sec) => {
                debug_assert!(sec >= 0.0, "Cannot check at negative intervals.");
                let millis = (sec * 1_000.0).floor() as u64;
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

    let (duration, r#int): (Duration, String) = match opt.interval  {
        Ok(millis) => (Duration::from_millis(millis), format!("{}ms", millis.to_string().as_str())),
        Err(e) => {
            eprintln!("Failed to parse interval: {:?}", e);
            std::process::exit(1);
        }
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
