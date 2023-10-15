use crate::{emit, input::InputOpt, manager::Manager, tasks::Tasks};

impl Manager {
	pub fn quit(&self, tasks: &Tasks, no_cwd_file: bool) -> bool {
		let tasks = tasks.len();
		if tasks == 0 {
			emit!(Quit(no_cwd_file));
			return false;
		}

		tokio::spawn(async move {
			let mut result = emit!(Input(InputOpt::top(format!(
				"There are {tasks} tasks running, sure to quit? (y/N)"
			))));

			if let Some(Ok(choice)) = result.recv().await {
				if choice == "y" || choice == "Y" {
					emit!(Quit(no_cwd_file));
				}
			}
		});
		false
	}
}
