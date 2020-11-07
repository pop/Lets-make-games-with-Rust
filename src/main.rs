use amethyst::{
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
};

fn main() -> amethyst::Result<()> {
    // Although not required, the logger provides useful information at runtime.
    amethyst::start_logger(Default::default());

    // Define our application root using this helper method.
    // This is our current directory.
    // For a more robust game we might make the root configurable, or more hard-coded depending on
    // how the game is installed.
    let app_root = application_root_dir()?;

    // Our assets directory is the root where Amethyst is going to look for config files.
    // Things like Sprite Sheets and other modding content.
    let assets_dir = app_root.join("assets");

    // Our display is configurable.
    // For now it's a simple 500x500 pixel window, but you can make it whatever you want!
    let display_config_path = app_root.join("config").join("display.ron");

    // Our Rendering bundle draws a simple orange backdrop and sets some defaults for a 2D game.
    let renderer = RenderingBundle::<DefaultBackend>::new()
        .with_plugin(
            RenderToWindow::from_config_path(display_config_path)?
                .with_clear([1.00, 0.33, 0.00, 1.0]),
        )
        .with_plugin(RenderFlat2D::default());

    // Create our game object with our bundles.
    let game_data = GameDataBuilder::default().with_bundle(renderer)?;

    // Create our application with our assets, our starting state, and our systems.
    let mut game = Application::new(assets_dir, SeaglState, game_data)?;

    // Vroom vroom!
    game.run();

    Ok(())
}

// In Amethyst, States can be transitioned to and from.
// In our simple game we only have one state, but you could imagine having a LoadingScreen state, a
// MenuState, a PlayState, etc.
// https://book.amethyst.rs/stable/concepts/state.html
struct SeaglState;

// In the next step we will implement custom State logic, but for now we can use the out of the box
// implementation provided by Amethyst to get our game to compile.
impl SimpleState for SeaglState {}
