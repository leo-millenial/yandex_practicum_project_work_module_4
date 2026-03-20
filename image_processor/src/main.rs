use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use image_processor::error::AppError;
use image_processor::plugin_loader::{call_plugin_process, PluginLoader};
use image_processor::Image;

#[derive(Parser, Debug)]
#[command(
    name = "image_processor",
    about = "Image processing tool with dynamic plugin support",
    long_about = None
)]
struct Args {
    #[arg(help = "Path to input PNG image")]
    input: String,

    #[arg(help = "Path to output PNG image")]
    output: String,

    #[arg(help = "Path to plugin .so file")]
    plugin: String,

    #[arg(help = "Parameters string to pass to plugin", default_value = "")]
    params: String,
}

fn main() -> Result<(), AppError> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(
                "image_processor=debug".parse().map_err(|e| {
                    AppError::InvalidArgs(format!("Invalid tracing directive: {}", e))
                })?,
            ),
        )
        .init();

    let args = Args::parse();
    tracing::info!("Starting image processor");
    tracing::info!("Input: {}", args.input);
    tracing::info!("Output: {}", args.output);
    tracing::info!("Plugin: {}", args.plugin);
    tracing::info!("Params: {:?}", args.params);

    tracing::info!("Loading image from: {}", args.input);
    let mut image = Image::from_file(&args.input).map_err(AppError::Image)?;
    tracing::info!("Image loaded: {}x{}", image.width, image.height);

    tracing::info!("Loading plugin from: {}", args.plugin);
    let loader = PluginLoader::new(&args.plugin).map_err(AppError::Plugin)?;
    let process_fn = loader.get_process_image_fn().map_err(AppError::Plugin)?;
    tracing::info!("Plugin loaded successfully");

    tracing::info!("Processing image with plugin");
    call_plugin_process(
        process_fn,
        image.width,
        image.height,
        image.rgba_slice_mut(),
        &args.params,
    )
    .map_err(AppError::Plugin)?;
    tracing::info!("Plugin processing complete");

    tracing::info!("Saving result to: {}", args.output);
    image.save(&args.output).map_err(AppError::Image)?;
    tracing::info!("Result saved successfully");

    println!("Image processed successfully!");
    println!("Output saved to: {}", args.output);

    Ok(())
}
