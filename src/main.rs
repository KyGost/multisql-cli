use cli_table::{print_stdout, Cell, Table};
use multisql::{Cast, Payload};
use {
	dialoguer::{theme::ColorfulTheme, Input},
	indicatif::{ProgressBar, ProgressStyle},
	lazy_static::lazy_static,
	multisql::Glue,
};
lazy_static! {
	pub(crate) static ref PROGRESS_STYLE: ProgressStyle = ProgressStyle::default_spinner()
		.template("{spinner:.magenta} {elapsed:3.red} {msg:.green}")
		.tick_chars("|/—\\*");
}

fn main() {
	let mut glue = Glue::new_multi(vec![]);
	while prompt(&mut glue) {}
}

fn prompt(glue: &mut Glue) -> bool {
	let query: String = Input::with_theme(&ColorfulTheme::default())
		.with_prompt("Query")
		.interact()
		.unwrap();

	let progress = ProgressBar::new_spinner()
		.with_message(format!("Running Query"))
		.with_style(PROGRESS_STYLE.clone());
	progress.enable_steady_tick(200);
	let result = glue.execute(&query);
	progress.finish();

	match result {
		Err(err) => println!("{:?}", err),
		Ok(Payload::Select { labels, rows }) => {
			let table = rows
				.into_iter()
				.map(|row| {
					row.0
						.into_iter()
						.map(|value| {
							let string: String = value.cast().unwrap();
							string.cell()
						})
						.collect()
				})
				.collect::<Vec<Vec<_>>>()
				.table()
				.title(labels.into_iter().map(|label| label.cell()).collect::<Vec<_>>());
			print_stdout(table).unwrap()
		}
		Ok(payload) => println!("{:?}", payload),
	};

	true
}
