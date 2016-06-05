
extern crate rand;
extern crate ansi_term;

mod map;


#[macro_use]extern crate conrod;
extern crate piston;
extern crate piston_window;

use conrod::{
    color,
    Canvas,
    Color,
    Colorable,
    Frameable,
    Labelable,
    Positionable,
    Sizeable,
    Text,
    Theme,
    Toggle,
    Widget,
    WidgetMatrix,
};
use piston_window::{EventLoop, Glyphs, OpenGL, PistonWindow, UpdateEvent, WindowSettings};
use std::sync::mpsc;

mod cell;


/// Conrod is backend agnostic. Here, we define the `piston_window` backend to use for our `Ui`.
type Backend = (<piston_window::G2d<'static> as conrod::Graphics>::Texture, Glyphs);
type Ui = conrod::Ui<Backend>;
type UiCell<'a> = conrod::UiCell<'a, Backend>;

enum AppStatus {
    Playing,
    Failed,
    Finished,
}

/// This struct holds all of the variables used to demonstrate application data being passed
/// through the widgets. If some of these seem strange, that's because they are! Most of these
/// simply represent the aesthetic state of different parts of the GUI to offer visual feedback
/// during interaction with the widgets.
struct DemoApp {
    map: map::Map,

    /// Background color (for demonstration of button and sliders).
    bg_color: Color,
    /// Should the button be shown (for demonstration of button).
    /// and the title.
    title_pad: f64,
    /// The height of the vertical sliders (we will play with this
    /// using a number_dialer).
    /// The widget frame width (we'll use this to demo Framing
    /// and number_dialer).
    frame_width: f64,
    /// A vector of strings for drop_down_list demonstration.
    /// A channel for sending results from the `WidgetMatrix`.
    elem_sender: mpsc::Sender<(usize, usize, cell::Interaction)>,
    elem_receiver: mpsc::Receiver<(usize, usize, cell::Interaction)>,

    status: AppStatus,
    title: String,
}

impl DemoApp {

    /// Constructor for the Demonstration Application model.
    fn new() -> DemoApp {
        let (elem_sender, elem_receiver) = mpsc::channel();

        use map::Map;

        let mut map : Map = Default::default();
        map.populate(10, 10, 20);
        println!("{}", &map);

        DemoApp {
            bg_color: color::rgb(0.2, 0.35, 0.45),
            title_pad: 350.0,
            frame_width: 1.0,
            elem_sender: elem_sender,
            elem_receiver: elem_receiver,
            map: map,
            title: "Minesweeper".to_owned(),
            status: AppStatus::Playing,
        }
    }

}


fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("MineSweeper", [1100, 560])
            .opengl(opengl).exit_on_esc(true).vsync(true).build().unwrap();

    // construct our `Ui`.
    let mut ui = {
        let font_path = "assets/fonts/NotoSans/NotoSans-Regular.ttf";
        let theme = Theme::default();
        let glyph_cache = Glyphs::new(&font_path, window.factory.clone());
        Ui::new(glyph_cache.unwrap(), theme)
    };

    // Our dmonstration app that we'll control with our GUI.
    let mut app = DemoApp::new();

    window.set_ups(60);

    // Poll events from the window.
    while let Some(event) = window.next() {
        ui.handle_event(&event);

        // We'll set all our widgets in a single function called `set_widgets`.
        // At the moment conrod requires that we set our widgets in the Render loop,
        // however soon we'll add support so that you can set your Widgets at any arbitrary
        // update rate.
        event.update(|_| ui.set_widgets(|mut ui| set_widgets(&mut ui, &mut app)));

        // Draw our Ui!
        //
        // The `draw_if_changed` method only re-draws the GUI if some `Widget`'s `Element`
        // representation has changed. Normally, a `Widget`'s `Element` should only change
        // if a Widget was interacted with in some way, however this is up to the `Widget`
        // designer's discretion.
        //
        // If instead you need to re-draw your conrod GUI every frame, use `Ui::draw`.
        window.draw_2d(&event, |c, g| ui.draw_if_changed(c, g));
    }
}

/// Set all `Widget`s within the User Interface.
///
/// The first time this gets called, each `Widget`'s `State` will be initialised and cached within
/// the `Ui` at their given indices. Every other time this get called, the `Widget`s will avoid any
/// allocations by updating the pre-existing cached state. A new graphical `Element` is only
/// retrieved from a `Widget` in the case that it's `State` has changed in some way.
fn set_widgets(ui: &mut UiCell, app: &mut DemoApp) {

    // We can use this `Canvas` as a parent Widget upon which we can place other widgets.
    Canvas::new()
        .frame(app.frame_width)
        .pad(30.0)
        .color(app.bg_color)
        .scroll_kids()
        .set(CANVAS, ui);

    // Text example.
    Text::new(app.title.as_str())
        .top_left_with_margins_on(CANVAS, 0.0, app.title_pad)
        .font_size(32)
        .color(app.bg_color.plain_contrast())
        .set(TITLE, ui);

    // A demonstration using widget_matrix to easily draw
    // a matrix of any kind of widget.
    WidgetMatrix::new(app.map.width, app.map.height)
        .down(20.0)
        .w_h(260.0, 260.0) // matrix width and height.
        .each_widget(|_n, col: usize, row: usize| { // called for every matrix elem.

            // Color effect for fun.
            // let (r, g, b, a) = (
            //     0.5 + (col as f32 / cols as f32) / 2.0,
            //     0.75,
            //     1.0 - (row as f32 / rows as f32) / 2.0,
            //     1.0
            // );

            // Now return the widget we want to set in each element position.
            // You can return any type that implements `Widget`.
            // The returned widget will automatically be positioned and sized to the matrix
            // element's rectangle.
            let elem = app.map.grid[row * app.map.width + col].mine;
            let elem_sender = app.elem_sender.clone();

            let ref cell_data = app.map.grid[row * app.map.width + col];
            let label = match cell_data.interaction {
                map::Interaction::Opened => match cell_data.nearby {
                    0 => " ",
                    1 => "1",
                    2 => "2",
                    3 => "3",
                    4 => "4",
                    5 => "5",
                    6 => "6",
                    7 => "7",
                    8 => "8",
                    _ => "#",
                },
                map::Interaction::Undiscovered => " ",
                map::Interaction::Flagged => "!",
            };

            let (r, g, b) = match cell_data.interaction {
                map::Interaction::Opened => (0.8, 0.8, 0.8),
                map::Interaction::Undiscovered => (0.5, 0.5, 0.5),
                map::Interaction::Flagged => (0.5, 0.5, 0.5),
            };
            let cell = cell::Cell::new()
                .w_h(200.0, 50.0)
                .down_from(TITLE, 45.0)
                .rgb(r, g, b)
                .label(label)
                .react(move |btn: cell::Interaction| elem_sender.send((col, row, btn)).unwrap());
            cell
        })
        .set(TOGGLE_MATRIX, ui);

    // Receive updates to the matrix from the `WidgetMatrix`.
    while let Ok((col, row, btn)) = app.elem_receiver.try_recv() {
        match btn {
            cell::Interaction::LeftClicked => {
                if app.map.grid[row * app.map.width + col].mine {
                    app.title = "EXPLODE!".to_owned();
                    app.status = AppStatus::Failed;
                } else {
                    app.map.reveal(col, row);
                }
            },
            cell::Interaction::RightClicked => {
                match app.map.grid[row * app.map.width + col].interaction {
                    map::Interaction::Flagged => app.map.grid[row * app.map.width + col].interaction = map::Interaction::Undiscovered,
                    map::Interaction::Undiscovered => app.map.grid[row * app.map.width + col].interaction = map::Interaction::Flagged,
                    _ => (),
                };
                println!("Right Clicked");
            },
            cell::Interaction::BothClicked => {
                println!("Both Clicked");
            }
            _ => (),
        }

    }

}


// In conrod, each widget must have its own unique identifier so that the `Ui` can keep track of
// its state between updates.
// To make this easier, conrod provides the `widget_ids` macro, which generates a unique `WidgetId`
// for each identifier given in the list.
// The `with n` syntax reserves `n` number of WidgetIds for that identifier, rather than just one.
// This is often useful when you need to use an identifier in some kind of loop (i.e. like within
// the use of `WidgetMatrix` as above).
widget_ids! {
    CANVAS,
    TITLE,
    FRAME_WIDTH,
    TOGGLE_MATRIX,
}
