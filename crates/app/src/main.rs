use clap::{
  Parser,
  Subcommand,
};
use ellastic_app::{
  ApplicationConfig,
  ApplicationMode,
  ElasticApplication,
};
use ellastic_errors::Result;
use std::env;
use tokio;

#[derive(Parser)]
#[command(name = "ellastic")]
#[command(about = "Ellastic multimedia databending toolkit")]
#[command(version)]
#[command(author)]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,

  #[arg(short, long, help = "Run in GUI mode")]
  gui: bool,

  #[arg(short, long, help = "Run in CLI mode")]
  cli: bool,

  #[arg(short, long, help = "Run in server mode")]
  server: bool,

  #[arg(short, long, help = "Run in headless mode")]
  headless: bool,

  #[arg(short, long, help = "Configuration file")]
  config: Option<String>,

  #[arg(short, long, help = "Log level")]
  log_level: Option<String>,

  #[arg(short, long, help = "Plugin directory")]
  plugin_dir: Option<String>,

  #[arg(short, long, help = "Project file to open")]
  project: Option<String>,

  #[arg(long, help = "Enable debug mode")]
  debug: bool,

  #[arg(long, help = "Enable verbose logging")]
  verbose: bool,

  #[arg(long, help = "Enable experimental features")]
  experimental: bool,
}

#[derive(Subcommand)]
enum Commands {
  #[command(about = "Create a new project")]
  New {
    #[arg(help = "Project name")]
    name: String,

    #[arg(short, long, help = "Project template")]
    template: Option<String>,

    #[arg(short, long, help = "Project directory")]
    directory: Option<String>,
  },

  #[command(about = "Open an existing project")]
  Open {
    #[arg(help = "Project file or directory")]
    path: String,
  },

  #[command(about = "Import media files")]
  Import {
    #[arg(help = "Files to import")]
    files: Vec<String>,

    #[arg(short, long, help = "Import recursively")]
    recursive: bool,

    #[arg(short, long, help = "Target project")]
    project: Option<String>,
  },

  #[command(about = "Apply effects to media")]
  Apply {
    #[arg(help = "Effect to apply")]
    effect: String,

    #[arg(help = "Files to process")]
    files: Vec<String>,

    #[arg(short, long, help = "Effect intensity")]
    intensity: Option<f64>,

    #[arg(short, long, help = "Output directory")]
    output: Option<String>,
  },

  #[command(about = "Run a pipeline")]
  Run {
    #[arg(help = "Pipeline file")]
    pipeline: String,

    #[arg(short, long, help = "Input files")]
    input: Vec<String>,

    #[arg(short, long, help = "Output directory")]
    output: Option<String>,

    #[arg(short, long, help = "Run in parallel")]
    parallel: bool,
  },

  #[command(about = "Export media")]
  Export {
    #[arg(help = "Files to export")]
    files: Vec<String>,

    #[arg(short, long, help = "Output format")]
    format: Option<String>,

    #[arg(short, long, help = "Output quality")]
    quality: Option<u8>,

    #[arg(short, long, help = "Output directory")]
    output: Option<String>,
  },

  #[command(about = "Show version information")]
  Version,

  #[command(about = "Show help information")]
  Help {
    #[arg(help = "Command to get help for")]
    command: Option<String>,
  },
}

#[tokio::main]
async fn main() -> Result<()> {
  let cli = Cli::parse();

  let mode = determine_mode(&cli);

  let mut config = create_config(&cli, mode)?;

  if let Some(config_file) = &cli.config {
    load_config_file(&mut config, config_file)?;
  }

  let mut app = ElasticApplication::new(config);

  app.initialize().await?;

  match cli.command {
    Some(command) => handle_command(app, command).await?,
    None => app.run().await?,
  }

  app.shutdown().await?;

  Ok(())
}

fn determine_mode(cli: &Cli) -> ApplicationMode {
  if cli.gui {
    ApplicationMode::GUI
  } else if cli.cli {
    ApplicationMode::CLI
  } else if cli.server {
    ApplicationMode::Server
  } else if cli.headless {
    ApplicationMode::Headless
  } else {
    if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
      ApplicationMode::GUI
    } else {
      ApplicationMode::CLI
    }
  }
}

fn create_config(cli: &Cli, mode: ApplicationMode) -> Result<ApplicationConfig> {
  let mut config = ApplicationConfig::default();

  config.mode = mode;

  match mode {
    ApplicationMode::GUI => {
      config.features.enable_ui = true;
      config.features.enable_cli = false;
    }
    ApplicationMode::CLI => {
      config.features.enable_ui = false;
      config.features.enable_cli = true;
    }
    ApplicationMode::Server => {
      config.features.enable_ui = false;
      config.features.enable_cli = false;
      config.features.enable_networking = true;
    }
    ApplicationMode::Headless => {
      config.features.enable_ui = false;
      config.features.enable_cli = false;
    }
    ApplicationMode::Embedded => {
      config.features.enable_ui = false;
      config.features.enable_cli = false;
    }
  }

  if let Some(log_level) = &cli.log_level {
    config.logging.level = log_level.clone();
  }

  if cli.debug {
    config.logging.level = "debug".to_string();
    config.advanced.debug_mode = true;
    config.advanced.verbose_logging = true;
  }

  if cli.verbose {
    config.logging.level = "trace".to_string();
    config.advanced.verbose_logging = true;
  }

  if cli.experimental {
    config.advanced.experimental_features = true;
  }

  if let Some(plugin_dir) = &cli.plugin_dir {
    config.services.plugin_service.plugin_directory = plugin_dir.clone();
  }

  Ok(config)
}

fn load_config_file(config: &mut ApplicationConfig, config_file: &str) -> Result<()> {
  println!("Loading config from: {}", config_file);
  Ok(())
}

async fn handle_command(mut app: ElasticApplication, command: Commands) -> Result<()> {
  match command {
    Commands::New {
      name,
      template,
      directory,
    } => {
      println!("Creating new project: {}", name);
      if let Some(template) = template {
        println!("Using template: {}", template);
      }
      if let Some(directory) = directory {
        println!("In directory: {}", directory);
      }
    }

    Commands::Open { path } => {
      println!("Opening project: {}", path);
    }

    Commands::Import {
      files,
      recursive,
      project,
    } => {
      println!("Importing {} files", files.len());
      if recursive {
        println!("Importing recursively");
      }
      if let Some(project) = project {
        println!("Into project: {}", project);
      }
    }

    Commands::Apply {
      effect,
      files,
      intensity,
      output,
    } => {
      println!("Applying effect '{}' to {} files", effect, files.len());
      if let Some(intensity) = intensity {
        println!("With intensity: {}", intensity);
      }
      if let Some(output) = output {
        println!("Output to: {}", output);
      }
    }

    Commands::Run {
      pipeline,
      input,
      output,
      parallel,
    } => {
      println!("Running pipeline: {}", pipeline);
      if !input.is_empty() {
        println!("With {} input files", input.len());
      }
      if let Some(output) = output {
        println!("Output to: {}", output);
      }
      if parallel {
        println!("Running in parallel");
      }
    }

    Commands::Export {
      files,
      format,
      quality,
      output,
    } => {
      println!("Exporting {} files", files.len());
      if let Some(format) = format {
        println!("As format: {}", format);
      }
      if let Some(quality) = quality {
        println!("With quality: {}", quality);
      }
      if let Some(output) = output {
        println!("Output to: {}", output);
      }
    }

    Commands::Version => {
      println!("Ellastic v{}", env!("CARGO_PKG_VERSION"));
      println!("LICENSE: GPLv3");
      println!("Repository: https://github.com/playfairs/ellastic");
    }

    Commands::Help { command } => {
      if let Some(command) = command {
        println!("Help for command: {}", command);
      } else {
        println!("Ellastic");
        println!();
        println!("Usage: elastic [OPTIONS] [COMMAND]");
        println!();
        println!("Options:");
        println!("  -h, --help       Print help information");
        println!("  -V, --version    Print version information");
        println!("  -g, --gui        Run in GUI mode");
        println!("  -c, --cli        Run in CLI mode");
        println!("  -s, --server     Run in server mode");
        println!("      --headless   Run in headless mode");
        println!("  -C, --config     Configuration file");
        println!("  -l, --log-level   Log level");
        println!("  -p, --plugin-dir Plugin directory");
        println!("      --debug       Enable debug mode");
        println!("      --verbose     Enable verbose logging");
        println!("      --experimental Enable experimental features");
        println!();
        println!("Commands:");
        println!("  new      Create a new project");
        println!("  open     Open an existing project");
        println!("  import   Import media files");
        println!("  apply    Apply effects to media");
        println!("  run      Run a pipeline");
        println!("  export   Export media");
        println!("  version  Show version information");
        println!("  help     Show help information");
      }
    }
  }

  Ok(())
}
