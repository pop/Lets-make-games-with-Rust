use amethyst::{
    assets::{AssetStorage, Loader},
    core::{
        timing::Time,
        transform::{Transform, TransformBundle},
    },
    derive::SystemDesc,
    ecs::{Component, DenseVecStorage, Join, Read, ReadStorage, System, SystemData, WriteStorage},
    input::{InputBundle, InputHandler, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        Camera, ImageFormat, RenderingBundle, SpriteRender, SpriteSheet, SpriteSheetFormat,
        Texture,
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

    // The Transform bundle allows us to use the Transform component.
    // This is used to tag entities with positions in space and move then around.
    let transform = TransformBundle::new();

    // Our Rendering bundle draws a simple orange backdrop and sets some defaults for a 2D game.
    let renderer = RenderingBundle::<DefaultBackend>::new()
        .with_plugin(
            RenderToWindow::from_config_path(display_config_path)?
                .with_clear([1.00, 0.33, 0.00, 1.0]),
        )
        .with_plugin(RenderFlat2D::default());

    // The InputBundle maps user input to game actions
    // Later on we can monitor user input and search for things like "horizontal" instead of the
    // literal key inputs.
    let bindings_path = app_root.join("config").join("bindings.ron");
    let inputs = InputBundle::<StringBindings>::new().with_bindings_from_file(bindings_path)?;

    // Create our game object with our bundles.
    // we also include our own systems now that we have some of our own to run!
    let game_data = GameDataBuilder::default()
        .with_bundle(renderer)?
        .with_bundle(transform)?
        .with_bundle(inputs)?
        .with(MoveSystem, "move_system", &["input_system"]);

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

impl SimpleState for SeaglState {
    // Here we've implemented one of the `SimpleState` methods: `on_start`.
    // When the State is first booted up Amethyst will run through this code _once_.
    fn on_start(&mut self, data: StateData<GameData>) {
        // We don't have any Systems that use Seal yet, so we need to manually register this
        // component with the world.
        data.world.register::<Seagl>();

        // Create our Camera
        // Our Camera won't move around, we still need to initialize it in space, so we add the
        // transform component to our Camera entity.
        let mut transform = Transform::default();

        // We set the Z transform to `1` so we can later move sprites to different Z levels.
        // If everything was at the same Z level sprites would overlap at random -- CHAOS!
        transform.set_translation_xyz(50.0, 50.0, 1.0);

        // Create an entity with the Camera component.
        // Now we can place entities in front of or behind the camera.
        data.world
            .create_entity()
            .with(Camera::standard_2d(100.0, 100.0))
            .with(transform)
            .build();

        // Next we're going to load up our sprite sheet so we have something to represent our Seagl entity.
        let sprite_sheet_handle = {
            // Create a loader object which we will use at the end to load our final sprite sheet.
            let loader = data.world.read_resource::<Loader>();

            // Create a storage object
            let texture_storage = data.world.read_resource::<AssetStorage<Texture>>();

            // First we have to load a texture which represents the entire PNG image.
            let texture_handle = loader.load(
                "texture/spritesheet.png",
                ImageFormat::default(),
                (),
                &texture_storage,
            );

            // Then we need to create a sprite sheet storage object
            let sprite_sheet_store = data.world.read_resource::<AssetStorage<SpriteSheet>>();

            // Convert our massive image into a sprite sheet based on the mapping in
            // `spritesheet.ron`.
            //
            // A Sprite Sheet is like a Texture, but you can index it like an array.
            // in our `spritesheet.ron` file we map areas of the texture to specific sprites.
            // This means we can have sprites overlap and pack tightly.
            // Later we will get the first element from this sheet, the Seagl.
            //
            // One last thing: This is the last statement in a let foo = { ... }; block, so it is
            // returned the value to be assigned to `sprite_sheet_handle`.
            loader.load(
                "texture/spritesheet.ron",
                SpriteSheetFormat(texture_handle),
                (),
                &sprite_sheet_store,
            )
        };

        // Next we create a Seagl.
        // It definitely lives in space, and will move around, so we give it a transform component.
        let mut transform = Transform::default();

        // Our arena is 100x100, so we draw (see our Camera component above)
        // So to draw our Seagl at the center we put it at 50x50
        transform.set_translation_xyz(50.0, 50.0, 0.0);

        // Using our sprite sheet, get the first sprite specified by `spritesheet.ron`
        let sprite = SpriteRender::new(sprite_sheet_handle.clone(), 0);

        // Create a Seagl struct, which happens to just be a marker on this entity.
        let seagl = Seagl::default();

        // Build our main player entity
        data.world
            .create_entity()
            .with(seagl)
            .with(sprite)
            .with(transform)
            .build();
    }
}

// Our Seagl component.
// We use a zero-size struct as a marker on an entity later in the exercise.
// We can filter on, for example, all entities with Seagl and Transform components
// and do some transformation on all of them.
#[derive(Default)]
pub struct Seagl;

// This is boilerplate Amethyst.
// We could optimize the Storage type if we wanted, but DenseVecStorage is a pretty good default.
impl Component for Seagl {
    type Storage = DenseVecStorage<Self>;
}

// Movement system
// Systems can store state, but ours does not.
// It's just a data type to implement System onto.
#[derive(SystemDesc)]
pub struct MoveSystem;

impl<'s> System<'s> for MoveSystem {
    // We need four pices of data to query on:
    // * Transform (everything with coordinates) as WriteStorage because we will modify it.
    // * Seagl as ReadStorage, we won't modify this component, but we will use it to filter for
    //   specific entities.
    // * Time is used to figure out how much time has passed since the last frame. This is then
    //   used to determine how far the Seagl should move this frame.
    // * InputHandler gives us all of the user input. We monitor for specific input and throw away
    //   what we don't need.
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Seagl>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,
    );

    // We define a run method which takes a mutable reference to the system itself and the
    // systemdata described above.
    // I still think it's so cool that we can tell from the function signature what this function
    // does: modify transforms based on the state of seagls + time + user input.
    fn run(&mut self, (mut transforms, seagls, time, input): Self::SystemData) {
        // This is fun to modify. A speed of 50 felt pretty good.
        let speed: f32 = 50.0;
        // Iterate over all Seagls. I know there is only one, but we could add more!
        for (_seagl, transform) in (&seagls, &mut transforms).join() {
            // Horizontal movement can be positive or negative
            if let Some(horizontal) = input.axis_value("horizontal") {
                transform.prepend_translation_x(
                    // move the horizontal amount (some positive or negative value)
                    // Multiplied by the time difference since the last frame
                    // And scaled by the speed we want to move
                    horizontal * time.delta_seconds() * speed as f32,
                );
                // If the user input is non-zero we change direction
                // We're rotating around the Y-axis by PI because we are rotating by radians.
                // It would be more intuitive maybe to mirror our sprite or something, but this
                // achieves the same result.
                if horizontal > 0.0 {
                    transform.set_rotation_y_axis(std::f32::consts::PI);
                }
                // Note we specifically do nothing if the horizontal value is exactly 0 because
                // that means they're not touching the keyboard, and we want our Seagl to keep
                // looking in whatever direction they were looking at last.
                if horizontal < 0.0 {
                    transform.set_rotation_y_axis(0.0);
                }
            };
            // Same stuff but for up and down.
            if let Some(vertical) = input.axis_value("vertical") {
                transform.prepend_translation_y(vertical * time.delta_seconds() * speed as f32);
            };
        }
    }
}
