use {
	console::Style,
	dialoguer::{theme::ColorfulTheme, Confirm, Editor, Input, Select},
	indicatif::{ProgressBar, ProgressStyle},
	lazy_static::lazy_static,
	multisql::{CSVStorage, Glue, SledStorage, Storage},
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

const PROMPT_ACTION: [&str; 2] = ["Connect", "Query"];
const PROMPT_KIND: [&str; 2] = ["Sled", "CSV"];
const QUERY_KIND: [&str; 3] = ["Small", "Big", "File"];

fn prompt(glue: &mut Glue) -> bool {
	let mut input_theme = ColorfulTheme::default();
	input_theme.active_item_prefix = Style::new().green().apply_to(String::from("•-"));

	match Select::with_theme(&input_theme)
		.items(&PROMPT_ACTION)
		.default(0)
		.interact()
		.unwrap()
	{
		0 => {
			let name = Input::with_theme(&input_theme)
				.with_prompt("Name")
				.interact()
				.unwrap();
			let new_storage = match Select::with_theme(&input_theme)
				.items(&PROMPT_KIND)
				.default(0)
				.interact()
				.unwrap()
			{
				0 => {
					let path: String = Input::with_theme(&input_theme)
						.with_prompt("Path")
						.interact()
						.unwrap();
					Storage::new_sled(SledStorage::new(&path).unwrap())
				}
				1 => {
					let path: String = Input::with_theme(&input_theme)
						.with_prompt("Path")
						.interact()
						.unwrap();
					Storage::new_csv(CSVStorage::new(&path).unwrap())
				}
				_ => unreachable!(),
			};
			glue.extend(vec![Glue::new(name, new_storage)]);
		}
		1 => {
			let query = match Select::with_theme(&input_theme)
				.items(&QUERY_KIND)
				.default(0)
				.interact()
				.unwrap()
			{
				0 => Input::with_theme(&input_theme)
					.with_prompt("Query")
					.interact()
					.unwrap(),
				1 => {
					let text = Editor::new()
						.extension("sql")
						.require_save(false)
						.edit("")
						.unwrap()
						.unwrap();
					println!("{}", text);
					Confirm::with_theme(&input_theme)
						.with_prompt("Run")
						.default(true)
						.interact()
						.unwrap();
					text
				}
				2 => unimplemented!(),
				_ => unreachable!(),
			};

			let progress = ProgressBar::new_spinner()
				.with_message(format!("Running Query"))
				.with_style(PROGRESS_STYLE.clone());
			progress.enable_steady_tick(200);

			glue.execute(&query).unwrap();

			progress.finish();
		}
		_ => unreachable!(),
	}

	true
}
