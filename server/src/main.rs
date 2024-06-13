use clap::Parser;
use log::trace;

use srv_mod_config::RootConfig;
use srv_mod_database::humantime;

use crate::async_main::async_main;
use crate::cli::base::{CliArguments, Commands};
use crate::cli::generate::GenerateSubcommands;
use crate::cli_cmd_generate::dummy_data::make_dummy_data;

mod async_ctx;
mod async_main;
mod cli;
mod cli_cmd_compile;
mod cli_cmd_generate;
mod servers;

fn setup_logging(debug_level: u8) -> anyhow::Result<()> {
	let mut base_config = fern::Dispatch::new()
		.format(|out, message, record| {
			let level_padding = if record.level().to_string().len() < 5 {
				" ".repeat(5 - record.level().to_string().len() + 1)
				   .to_string()
			} else {
				" ".to_string()
			};

			let colors = fern::colors::ColoredLevelConfig::new()
				.info(fern::colors::Color::Green)
				.warn(fern::colors::Color::Yellow)
				.error(fern::colors::Color::Red)
				.debug(fern::colors::Color::Blue)
				.trace(fern::colors::Color::Magenta);

			let additional_info = if record.level() > log::LevelFilter::Debug {
				format!(
					" [{}:{}]",
					record.file().unwrap_or(""),
					record.line().unwrap_or(0)
				)
			} else {
				"".to_string()
			};

			out.finish(format_args!(
				"[{}]{}[{}]{} {}",
				colors.color(record.level()),
				level_padding,
				humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
				additional_info,
				message
			))
		})
		.level(log::LevelFilter::Trace)
		.chain(std::io::stdout());

	base_config = match debug_level {
		0 => base_config.level(log::LevelFilter::Info),
		1 => base_config.level(log::LevelFilter::Debug),
		_ => base_config.level(log::LevelFilter::Trace),
	};

	base_config.apply()?;
	Ok(())
}

fn main() -> anyhow::Result<()> {
	let args = CliArguments::parse();

	setup_logging(args.debug)?;
	trace!("Parsed arguments: {:?}", args);

	match args.command {
		Commands::Compile(compile_args) => match compile_args.command {
			cli::compile::CompileSubcommands::Agent => {
				todo!("Agent compilation not implemented yet");
			}
			cli::compile::CompileSubcommands::Gui => {
				cli_cmd_compile::c2_gui::compile()?;
			}
		},
		Commands::Generate(generate_args) => {
			match generate_args.command {
				GenerateSubcommands::Jwt => {
					cli_cmd_generate::jwt::generate_jwt()?;
				}
				GenerateSubcommands::Operator(generate_args) => {
					let config = RootConfig::load(&args.config)?;

					// requires async context to consume the configuration
					async_ctx::enter(cli_cmd_generate::operator::generate_operator(
						&generate_args,
						config,
					))?;
				}
				GenerateSubcommands::Certificate(generate_args) => {
					cli_cmd_generate::certificate::make_tls(&generate_args)?;
				}
				GenerateSubcommands::DummyData => {
					let config = RootConfig::load(&args.config)?;
					async_ctx::enter(make_dummy_data(config))?;
				}
			}
		}
		Commands::Run(_run_args) => {
			let config = RootConfig::load(&args.config)?;

			async_ctx::enter(async_ctx::init_context(
				args.debug,
				config.clone(),
				async_main(config.clone()),
			))?;
		}
	}

	Ok(())
}
