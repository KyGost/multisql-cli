use cli_table::{print_stdout, Cell, Table};
use multisql::{Cast, Connection, Payload};
use {
	dialoguer::{theme::ColorfulTheme, Input},
	indicatif::{ProgressBar, ProgressStyle},
	lazy_static::lazy_static,
	multisql::Glue,
	std::{
		fs::OpenOptions,
		io::{prelude::*, SeekFrom},
	},
};
lazy_static! {
	pub(crate) static ref PROGRESS_STYLE: ProgressStyle = ProgressStyle::default_spinner()
		.template("{spinner:.magenta} {elapsed:3.red} {msg:.green}")
		.tick_chars("|/—\\*");
}

fn main() {
	let mut connection_file_path = dirs::home_dir().unwrap();
	connection_file_path.push(".multisql-cli.yaml");

	let mut connection_file = OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.open(&connection_file_path)
		.unwrap();
	let mut connection_json = String::new();
	connection_file
		.read_to_string(&mut connection_json)
		.unwrap();
	if connection_json == "" {
		connection_json = String::from("[]")
	};

	let connections: Vec<(String, Connection)> = serde_yaml::from_str(&connection_json).unwrap();
	let databases = connections
		.into_iter()
		.map(|(name, connection)| (name, connection.try_into().unwrap()))
		.collect();

	let mut glue = Glue::new_multi(databases);

	prompt(&mut glue);

	let connection_json = serde_yaml::to_string(&glue.into_connections()).unwrap();
	connection_file.set_len(0).unwrap();
	connection_file.seek(SeekFrom::Start(0)).unwrap();
	connection_file
		.write_all(&connection_json.into_bytes())
		.unwrap();
	connection_file.flush().unwrap();

	main();
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
				.title(
					labels
						.into_iter()
						.map(|label| label.cell())
						.collect::<Vec<_>>(),
				);
			print_stdout(table).unwrap()
		}
		Ok(payload) => println!("{:?}", payload),
	};

	true
}
