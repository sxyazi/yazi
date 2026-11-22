//! Build on Debian with `rustc --edition=2024 -O scripts/perf.rs -o target/yazi-perf`.
//! Run `target/yazi-perf BASELINE [CANDIDATE]` with release binaries.
//! Use `target/yazi-perf --editor BUILD [BUILD]` for builds with the native editor.

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
compile_error!("This benchmark currently targets Linux x86_64");

use std::{env, error::Error, ffi::{c_char, c_int, c_short, c_ulong, c_void}, fs::{self, DirBuilder, File}, io::{self, Read, Write}, os::{fd::{AsRawFd, FromRawFd}, unix::{fs::DirBuilderExt, process::CommandExt}}, path::{Path, PathBuf}, process::{self, Child, Command, Stdio}, ptr, thread, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const WIDTH: u16 = 120;
const HEIGHT: u16 = 32;
const RUNS: usize = 7;
const IDLE: Duration = Duration::from_secs(5);
const TIMEOUT: Duration = Duration::from_secs(12);
const ENTRIES: usize = 1_000;
const READY: &[u8] = b"bench-enter";
const TOP: &[u8] = b"aa-bench-first";
const BOTTOM: &[u8] = b"zz-bench-last";
const END_FRAME: &[u8] = b"\x1b[?2026l";
const DA_QUERY: &[u8] = b"\x1b[0c";
const DA_REPLY: &[u8] = b"\x1b[?1;2c";

const F_SETFD: c_int = 2;
const FD_CLOEXEC: c_int = 1;
const POLLIN: c_short = 0x0001;
const TIOCSCTTY: c_ulong = 0x540e;

#[repr(C)]
struct WinSize {
	rows:     u16,
	cols:     u16,
	x_pixels: u16,
	y_pixels: u16,
}

#[repr(C)]
struct PollFd {
	fd:      c_int,
	events:  c_short,
	revents: c_short,
}

#[link(name = "util")]
unsafe extern "C" {
	fn openpty(
		master: *mut c_int,
		slave: *mut c_int,
		name: *mut c_char,
		termios: *const c_void,
		size: *const WinSize,
	) -> c_int;
	fn fcntl(fd: c_int, cmd: c_int, ...) -> c_int;
	fn poll(fds: *mut PollFd, count: c_ulong, timeout_ms: c_int) -> c_int;
	fn setsid() -> c_int;
	fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
}

fn main() -> Result<()> {
	let mut args = env::args_os().skip(1).peekable();
	let editor = args.next_if(|arg| arg == "--editor").is_some();
	let usage = "Usage: yazi-perf [--editor] BASELINE [CANDIDATE]";
	let baseline = fs::canonicalize(args.next().ok_or(usage)?)?;
	let candidate = args.next().map(fs::canonicalize).transpose()?;
	if args.next().is_some() {
		return Err(usage.into());
	}

	let mut programs = vec![Program::new("baseline", baseline)];
	if let Some(candidate) = candidate {
		programs.push(Program::new("candidate", candidate));
	}
	for program in &programs {
		println!("{}: {}", program.label, program.path.display());
		let version = Command::new(&program.path).arg("--version").output()?;
		if !version.status.success() {
			return Err(
				format!("{} --version failed: {}", program.path.display(), version.status).into(),
			);
		}
		print!("{}", String::from_utf8_lossy(&version.stdout));
	}

	if editor {
		return editor_bench(&programs);
	}

	let ticks = Command::new("getconf").arg("CLK_TCK").output()?;
	if !ticks.status.success() {
		return Err("getconf CLK_TCK failed".into());
	}
	let ticks_per_second: f64 = std::str::from_utf8(&ticks.stdout)?.trim().parse()?;
	let temp = TempDir::new()?;
	let fixture = temp.0.join("bench-fixture");
	let directory = fixture.join(std::str::from_utf8(READY)?);
	fs::create_dir_all(&directory)?;
	fs::write(directory.join("aa-bench-first.txt"), "Yazi benchmark fixture\n")?;
	fs::write(directory.join("zz-bench-last.txt"), "Yazi benchmark fixture\n")?;
	for i in 0..ENTRIES - 2 {
		File::create_new(directory.join(format!("entry-{i:04}.txt")))?;
	}

	println!("{WIDTH}x{HEIGHT} cells, {ENTRIES} entries, {} s idle per run", IDLE.as_secs());
	for program in &programs {
		let state = temp.0.join(format!("{}-warmup", program.label));
		trial(program, &fixture, &state, Duration::from_millis(200), ticks_per_second)?;
	}

	println!(
		"run  build       startup_ms  idle_cpu_%  rss_mib  pss_mib  enter_ms  bottom_ms  top_ms"
	);
	for run in 0..RUNS {
		// Alternate order so either build gets to run first in each pair.
		for i in 0..programs.len() {
			let index = (run + i) % programs.len();
			let program = &mut programs[index];
			let state = temp.0.join(format!("{}-{run}", program.label));
			let sample = trial(program, &fixture, &state, IDLE, ticks_per_second)?;
			print!("{:>3}  {:<10}", run + 1, program.label);
			for value in sample {
				print!(" {:>9.2}", value);
			}
			println!();
			program.samples.push(sample);
		}
	}

	for program in &programs {
		println!("{} (median and observed range)", program.label);
		for (i, metric) in
			["startup_ms", "idle_cpu_%", "rss_mib", "pss_mib", "enter_ms", "bottom_ms", "top_ms"]
				.into_iter()
				.enumerate()
		{
			let mut values: Vec<_> = program.samples.iter().map(|sample| sample[i]).collect();
			values.sort_unstable_by(f64::total_cmp);
			println!(
				"  {metric:<12} {:>8.2}  [{:.2} .. {:.2}]",
				values[RUNS / 2],
				values[0],
				values[RUNS - 1]
			);
		}
	}
	Ok(())
}

fn editor_bench(programs: &[Program]) -> Result<()> {
	const METRICS: [&str; 4] = ["open_ms", "type_ms", "undo_ms", "save_ms"];
	let temp = TempDir::new()?;
	for target in [4 * 1024, 10 * 1024 * 1024] {
		let mut text = String::from("undo-witness\n");
		let line = "sample text \t示例内容 編集\n";
		while text.len() < target {
			text.push_str(line);
		}
		println!("{WIDTH}x{HEIGHT} cells, {} bytes, 48 typed keys", text.len());
		println!("run  build         open_ms   type_ms   undo_ms   save_ms");

		let mut samples = vec![Vec::with_capacity(RUNS); programs.len()];
		for run in 0..=RUNS {
			for i in 0..programs.len() {
				let index = (run + i) % programs.len();
				let program = &programs[index];
				let state = temp.0.join(format!("{target}-{}-{run}", program.label));
				let result = trial_editor(program, &state, &text)?;
				if run > 0 {
					print!("{run:>3}  {:<10}", program.label);
					for value in result {
						print!(" {value:>9.2}");
					}
					println!();
					samples[index].push(result);
				}
			}
		}
		for (program, samples) in programs.iter().zip(samples) {
			println!("{} (median and observed range)", program.label);
			for (index, metric) in METRICS.iter().enumerate() {
				let mut values: Vec<_> = samples.iter().map(|s| s[index]).collect();
				values.sort_unstable_by(f64::total_cmp);
				println!(
					"  {metric:<12} {:>8.2}  [{:.2} .. {:.2}]",
					values[RUNS / 2],
					values[0],
					values[RUNS - 1]
				);
			}
		}
	}
	Ok(())
}

fn trial_editor(program: &Program, state: &Path, text: &str) -> Result<[f64; 4]> {
	let fixture = state.join("fixture");
	fs::create_dir_all(&fixture)?;
	let path = fixture.join("sample.txt");
	fs::write(&path, text)?;

	// Use the same save action for builds with either default keymap.
	let config = state.join("config");
	fs::create_dir_all(&config)?;
	fs::write(
		config.join("keymap.toml"),
		b"[view]\nprepend_keymap = [{ on = \"<C-s>\", run = \"save\" }]\n",
	)?;

	let mut terminal = Terminal::start(&program.path, &fixture, state)?;
	terminal.wait_frame("editor fixture", &[b"sample.txt"])?;
	// Older builds put the mode and file name in a single title.
	let open = terminal.step(b"\r", "open native editor", &[b"| sample.txt", b"NORMAL"])?;
	terminal.step(b"i", "enter insert mode", &[b"Insert", b"INSERT"])?;
	let mut keys = [b'x'; 48];
	keys[47] = b'Z';
	let typing = terminal.step(&keys, "type 48 characters", &[b"Z"])?;
	terminal.step(b"\x1b", "leave insert mode", &[b"Normal", b"NORMAL"])?;
	let undo = terminal.step(b"u", "undo typing", &[b"undo-witness"])?;
	if fs::read_to_string(&path)? != text {
		return Err("Typing or undo changed the file before save".into());
	}

	terminal.step(b"i", "enter insert mode again", &[b"Insert", b"INSERT"])?;
	terminal.step(b"Q", "type save marker", &[b"Q"])?;
	terminal.step(b"\x1b", "leave insert mode", &[b"Normal", b"NORMAL"])?;
	let expected_len = text.len() as u64 + 1;
	let started = Instant::now();
	terminal.master.write_all(b"\x13")?;
	// Measure until the replacement is visible, then verify its contents outside the timer.
	while fs::metadata(&path)?.len() != expected_len {
		if started.elapsed() >= TIMEOUT {
			return Err("Timed out waiting for the editor to save sample.txt".into());
		}
		thread::sleep(Duration::from_millis(1));
	}
	let save = started.elapsed();
	let saved = fs::read_to_string(&path)?;
	if saved.strip_prefix('Q') != Some(text) {
		return Err("Saved bytes differ from the expected editor contents".into());
	}
	terminal.drain(Duration::from_millis(100))?;
	terminal.master.write_all(b"\x1b")?;
	terminal.drain(Duration::from_millis(150))?;
	terminal.quit()?;

	Ok([open, typing, undo, save].map(|duration| duration.as_secs_f64() * 1_000.0))
}

struct Program {
	label:   &'static str,
	path:    PathBuf,
	samples: Vec<[f64; 7]>,
}

impl Program {
	fn new(label: &'static str, path: PathBuf) -> Self {
		Self { label, path, samples: Vec::with_capacity(RUNS) }
	}
}

fn trial(
	program: &Program,
	fixture: &Path,
	state: &Path,
	idle: Duration,
	ticks_per_second: f64,
) -> Result<[f64; 7]> {
	let mut terminal = Terminal::start(&program.path, fixture, state)?;
	let startup = terminal.wait_frame("initial directory list", &[READY])? - terminal.started;
	terminal.drain(Duration::from_millis(800))?;

	let proc = PathBuf::from("/proc").join(terminal.child.id().to_string());
	let ticks_before = cpu_ticks(&proc)?;
	let idle_start = Instant::now();
	terminal.drain(idle)?;
	let ticks = cpu_ticks(&proc)?.checked_sub(ticks_before).ok_or("CPU counter went backwards")?;
	let cpu = 100.0 * (ticks as f64) / ticks_per_second / idle_start.elapsed().as_secs_f64();
	let (rss, pss) = memory_mib(&proc)?;

	let enter = terminal.step(b"l", "enter directory", &[TOP])?;
	let bottom = terminal.step(b"G", "jump to bottom", &[BOTTOM])?;
	let top = terminal.step(b"gg", "jump to top", &[TOP])?;
	terminal.quit()?;

	let ms = |duration: Duration| duration.as_secs_f64() * 1_000.0;
	Ok([ms(startup), cpu, rss, pss, ms(enter), ms(bottom), ms(top)])
}

fn cpu_ticks(proc: &Path) -> Result<u64> {
	let stat = fs::read_to_string(proc.join("stat"))?;
	let (_, tail) = stat.rsplit_once(')').ok_or("missing process name in /proc/PID/stat")?;
	let mut fields = tail.split_whitespace().skip(11);
	let user: u64 = fields.next().ok_or("missing utime")?.parse()?;
	let system: u64 = fields.next().ok_or("missing stime")?.parse()?;
	Ok(user + system)
}

fn memory_mib(proc: &Path) -> Result<(f64, f64)> {
	let stats = fs::read_to_string(proc.join("smaps_rollup"))?;
	let read = |label: &str| -> Result<f64> {
		let value = stats
			.lines()
			.find_map(|line| line.strip_prefix(label))
			.and_then(|rest| rest.split_whitespace().next())
			.ok_or("missing RSS or PSS")?;
		Ok(value.parse::<f64>()? / 1024.0)
	};
	Ok((read("Rss:")?, read("Pss:")?))
}

struct TempDir(PathBuf);

impl TempDir {
	fn new() -> Result<Self> {
		let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
		let path =
			fs::canonicalize(env::temp_dir())?.join(format!("yazi-perf-{}-{now}", process::id()));
		DirBuilder::new().mode(0o700).create(&path)?;
		Ok(Self(path))
	}
}

impl Drop for TempDir {
	fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}

struct Terminal {
	child:        Child,
	master:       File,
	started:      Instant,
	bytes:        Vec<u8>,
	query_cursor: usize,
}

impl Terminal {
	fn start(binary: &Path, fixture: &Path, state: &Path) -> Result<Self> {
		let mut command = Command::new(binary);
		command.arg(fixture).env("TERM", "xterm-256color").env_remove("TERM_PROGRAM");
		for (name, _) in env::vars_os() {
			let text = name.to_string_lossy();
			if ["YAZI_", "KITTY_", "TMUX", "STY"].iter().any(|prefix| text.starts_with(prefix)) {
				command.env_remove(name);
			}
		}
		for (name, dir) in [
			("YAZI_CONFIG_HOME", "config"),
			("XDG_STATE_HOME", "state"),
			("XDG_CACHE_HOME", "cache"),
			("XDG_RUNTIME_DIR", "runtime"),
			("TMPDIR", "tmp"),
		] {
			let path = state.join(dir);
			DirBuilder::new().recursive(true).mode(0o700).create(&path)?;
			command.env(name, path);
		}

		let (mut master, mut slave) = (-1, -1);
		let size = WinSize { rows: HEIGHT, cols: WIDTH, x_pixels: 0, y_pixels: 0 };
		// openpty initializes both descriptors on success. Null pointers select default settings.
		if unsafe { openpty(&mut master, &mut slave, ptr::null_mut(), ptr::null(), &size) } == -1 {
			return Err(io::Error::last_os_error().into());
		}
		// These are fresh descriptors and each is transferred into one owner.
		let (master, slave) = unsafe { (File::from_raw_fd(master), File::from_raw_fd(slave)) };
		for file in [&master, &slave] {
			// openpty does not guarantee close-on-exec. The child receives only stdio.
			if unsafe { fcntl(file.as_raw_fd(), F_SETFD, FD_CLOEXEC) } == -1 {
				return Err(io::Error::last_os_error().into());
			}
		}
		command
			.stdin(Stdio::from(slave.try_clone()?))
			.stdout(Stdio::from(slave.try_clone()?))
			.stderr(Stdio::from(slave));
		// The child owns a new session. Stdin points at its PTY slave.
		unsafe {
			command.pre_exec(|| {
				if setsid() == -1 || ioctl(0, TIOCSCTTY, 0_i32) == -1 {
					return Err(io::Error::last_os_error());
				}
				Ok(())
			});
		}
		let started = Instant::now();
		let child = command.spawn()?;
		Ok(Self { child, master, started, bytes: Vec::new(), query_cursor: 0 })
	}

	fn read_until(&mut self, deadline: Instant) -> Result<Option<Instant>> {
		let Some(left) = deadline.checked_duration_since(Instant::now()) else { return Ok(None) };
		let mut fd = PollFd { fd: self.master.as_raw_fd(), events: POLLIN, revents: 0 };
		// poll reads exactly one initialized descriptor and does not retain its address.
		let result = unsafe { poll(&mut fd, 1, left.as_millis().max(1) as c_int) };
		if result == 0 {
			return Ok(None);
		}
		if result == -1 {
			return Err(io::Error::last_os_error().into());
		}

		let mut chunk = [0; 4096];
		let len = self.master.read(&mut chunk)?;
		if len == 0 {
			return Err("PTY closed before the expected output".into());
		}
		let received = Instant::now();
		self.bytes.extend_from_slice(&chunk[..len]);
		while let Some(found) = find(&self.bytes[self.query_cursor..], DA_QUERY) {
			self.query_cursor += found + DA_QUERY.len();
			self.master.write_all(DA_REPLY)?;
		}
		self.query_cursor = self.query_cursor.max(self.bytes.len().saturating_sub(DA_QUERY.len() - 1));
		Ok(Some(received))
	}

	fn wait_frame(&mut self, action: &str, markers: &[&[u8]]) -> Result<Instant> {
		let deadline = Instant::now() + TIMEOUT;
		let mut cursor = 0;
		while let Some(received) = self.read_until(deadline)? {
			while let Some(end) = find(&self.bytes[cursor..], END_FRAME) {
				let end = cursor + end;
				if markers.iter().any(|marker| find(&self.bytes[cursor..end], marker).is_some()) {
					return Ok(received);
				}
				cursor = end + END_FRAME.len();
			}
			// Retain the incomplete frame and any partial terminal query.
			let consumed = cursor.min(self.query_cursor);
			if consumed > 0 {
				self.bytes.drain(..consumed);
				self.query_cursor -= consumed;
				cursor -= consumed;
			}
			if self.bytes.len() > 1024 * 1024 {
				return Err(
					format!("Over 1 MiB of output without completing a frame during {action}").into(),
				);
			}
		}
		let expected: Vec<_> = markers.iter().map(|marker| String::from_utf8_lossy(marker)).collect();
		Err(format!("Timed out waiting for {action} ({})", expected.join(" or ")).into())
	}

	fn clear(&mut self) {
		self.bytes.clear();
		self.query_cursor = 0;
	}

	fn drain(&mut self, duration: Duration) -> Result<()> {
		let deadline = Instant::now() + duration;
		while self.read_until(deadline)?.is_some() {
			// Keep the partial query at the end even when no frame is being measured.
			self.bytes.drain(..self.query_cursor);
			self.query_cursor = 0;
		}
		self.clear();
		Ok(())
	}

	fn step(&mut self, key: &[u8], action: &str, markers: &[&[u8]]) -> Result<Duration> {
		self.clear();
		let started = Instant::now();
		self.master.write_all(key)?;
		Ok(self.wait_frame(action, markers)? - started)
	}

	fn quit(&mut self) -> Result<()> {
		self.master.write_all(b"q")?;
		let deadline = Instant::now() + Duration::from_secs(5);
		while Instant::now() < deadline {
			if let Some(status) = self.child.try_wait()? {
				return if status.success() {
					Ok(())
				} else {
					Err(format!("Yazi exited with {status}").into())
				};
			}
			thread::sleep(Duration::from_millis(10));
		}
		Err("Yazi did not exit after q".into())
	}
}

impl Drop for Terminal {
	fn drop(&mut self) {
		if matches!(self.child.try_wait(), Ok(None)) {
			let _ = self.child.kill();
			let _ = self.child.wait();
		}
	}
}

fn find(bytes: &[u8], needle: &[u8]) -> Option<usize> {
	bytes.windows(needle.len()).position(|window| window == needle)
}
