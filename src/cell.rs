//! A demonstration of designing a custom, third-party widget.
//!
//! In this case, we'll design a simple circular button.
//!
//! All of the custom widget design will occur within the `circular_button` module.
//!
//! We'll *use* our fancy circular button in the `main` function (below the circular_button module).
//!
//! Note that in this case, we use `piston_window` to draw our widget, however in practise you may
//! use any backend you wish.
//!
//! For more information, please see the `Widget` trait documentation.

/// The module in which we'll implement our own custom circular button.
// mod cell {
    use conrod::{
        Backend,
        Color,
        Colorable,
        CommonBuilder,
        Dimensions,
        FontSize,
        IndexSlot,
        Labelable,
        Mouse,
        Point,
        Positionable,
        Rectangle,
        Scalar,
        Text,
        Theme,
        UpdateArgs,
        Widget,
        WidgetKind,
    };


    /// The type upon which we'll implement the `Widget` trait.
    pub struct Cell<'a, F> {
        /// An object that handles some of the dirty work of rendering a GUI. We don't
        /// really have to worry about it.
        common: CommonBuilder,
        /// Optional label string for the button.
        maybe_label: Option<&'a str>,
        /// Optional callback for when the button is pressed. If you want the button to
        /// do anything, this callback must exist.
        maybe_react: Option<F>,
        /// See the Style struct below.
        style: Style,
        /// Whether the button is currently enabled, i.e. whether it responds to
        /// user input.
        enabled: bool
    }

    /// Represents the unique styling for our Cell widget.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Style {
        /// Color of the button.
        pub maybe_color: Option<Color>,
        /// Radius of the button.
        pub maybe_radius: Option<Scalar>,
        /// Color of the button's label.
        pub maybe_label_color: Option<Color>,
        /// Font size of the button's label.
        pub maybe_label_font_size: Option<u32>,
    }

    /// Represents the unique, cached state for our Cell widget.
    #[derive(Clone, Debug, PartialEq)]
    pub struct State {
        /// The current interaction state. See the Interaction enum below. See also
        /// get_new_interaction below, where we define all the logic for transitioning between
        /// interaction states.
        interaction: Interaction,
        /// An index to use for our **Circle** primitive graphics widget.
        circle_idx: IndexSlot,
        /// An index to use for our **Text** primitive graphics widget (for the label).
        text_idx: IndexSlot,
    }

    /// A `&'static str` that can be used to uniquely identify our widget type.
    pub const KIND: WidgetKind = "Cell";

    /// A type to keep track of interaction between updates.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum Interaction {
        Normal,
        Highlighted,
        LeftClicked,
        RightClicked,
        BothClicked,
    }

    impl Interaction {
        /// Alter the widget color depending on the current interaction.
        fn color(&self, color: Color) -> Color {
            match *self {
                /// The base color as defined in the Style struct, or a default provided
                /// by the current Theme if the Style has no color.
                Interaction::Normal => color,
                /// The Color object (from Elmesque) can calculate a highlighted version
                /// of itself. We don't have to use it, though. We could specify any color
                /// we want.
                Interaction::Highlighted => color.highlighted(),
                /// Ditto for clicked.
                Interaction::LeftClicked => color.clicked(),
                Interaction::RightClicked => color.clicked(),
                Interaction::BothClicked => color.clicked(),
            }
        }
    }

    /// Check the current interaction with the button. Takes into account whether the mouse is
    /// over the button and the previous interaction state.
    fn get_new_interaction(is_over: bool, prev: Interaction, mouse: Mouse) -> Interaction {
        use conrod::MouseButtonPosition::{Down, Up};
        use self::Interaction::{Normal, Highlighted, LeftClicked, RightClicked, BothClicked};
        // println!("{:?}, {:?}", mouse.left.position, mouse.right.position);
        match (is_over, prev, mouse.left.position, mouse.right.position) {
            // LMB is down over the button. But the button wasn't Highlighted last
            // update. This means the user clicked somewhere outside the button and
            // moved over the button holding LMB down. We do nothing in this case.
            (true,  Normal,  Down, _) => Normal,
            (true,  Normal,  _, Down) => Normal,

            // LMB is down over the button. The button was either Highlighted or Clicked
            // last update. If it was highlighted before, that means the user clicked
            // just now, and we transition to the Clicked state. If it was clicked
            // before, that means the user is still holding LMB down from a previous
            // click, in which case the state remains Clicked.
            (true, _, Down, Down) => BothClicked,

            (true,  _,       Down, _   ) => LeftClicked,
            (true,  _,       _   , Down) => RightClicked,

            // LMB is up. The mouse is hovering over the button. Regardless of what the
            // state was last update, the state should definitely be Highlighted now.
            (true,  _,       Up, Up)   => Highlighted,

            // LMB is down, the mouse is not over the button, but the previous state was
            // Clicked. That means the user clicked the button and then moved the mouse
            // outside the button while holding LMB down. The button stays Clicked.
            (false, LeftClicked, Down, _) => LeftClicked,

            (false, RightClicked, _, Down) => RightClicked,


            // If none of the above applies, then nothing interesting is happening with
            // this button.
            _                      => Normal,
        }
    }

    /// Return whether or not a given point is over a circle at a given point on a
    /// Cartesian plane. We use this to determine whether the mouse is over the button.
    pub fn is_over_circ(circ_center: Point, mouse_point: Point, dim: Dimensions) -> bool {
        true
    }

    impl<'a, F> Cell<'a, F> {
        /// Create a button context to be built upon.
        pub fn new() -> Cell<'a, F> {
            Cell {
                common: CommonBuilder::new(),
                maybe_react: None,
                maybe_label: None,
                style: Style::new(),
                enabled: true,
            }
        }

        /// Set the reaction for the Button. The reaction will be triggered upon release
        /// of the button. Like other Conrod configs, this returns self for chainability.
        pub fn react(mut self, reaction: F) -> Self {
            self.maybe_react = Some(reaction);
            self
        }

        /// If true, will allow user inputs.  If false, will disallow user inputs.  Like
        /// other Conrod configs, this returns self for chainability. Allow dead code
        /// because we never call this in the example.
        #[allow(dead_code)]
        pub fn enabled(mut self, flag: bool) -> Self {
            self.enabled = flag;
            self
        }
    }

    /// A custom Conrod widget must implement the Widget trait. See the **Widget** trait
    /// documentation for more details.
    impl<'a, F> Widget for Cell<'a, F>
        where F: FnMut(Interaction)
    {
        /// The State struct that we defined above.
        type State = State;

        /// The Style struct that we defined above.
        type Style = Style;

        fn common(&self) -> &CommonBuilder {
            &self.common
        }

        fn common_mut(&mut self) -> &mut CommonBuilder {
            &mut self.common
        }

        fn unique_kind(&self) -> &'static str {
            KIND
        }

        fn init_state(&self) -> State {
            State {
                interaction: Interaction::Normal,
                circle_idx: IndexSlot::new(),
                text_idx: IndexSlot::new(),
            }
        }

        fn style(&self) -> Style {
            self.style.clone()
        }

        /// Update the state of the button. The state may or may not have changed since
        /// the last update. (E.g. it may have changed because the user moused over the
        /// button.) If the state has changed, return the new state. Else, return None.
        fn update<B: Backend>(mut self, args: UpdateArgs<Self, B>) {
            let UpdateArgs { idx, state, rect, mut ui, style, .. } = args;
            let (xy, dim) = rect.xy_dim();
            let maybe_mouse = ui.input(idx).maybe_mouse.map(|mouse| mouse.relative_to(xy));

            // Check whether or not a new interaction has occurred.
            let new_interaction = match (self.enabled, maybe_mouse) {
                (false, _) | (true, None) => Interaction::Normal,
                (true, Some(mouse)) => {
                    // Conrod does us a favor by transforming mouse.xy into this widget's
                    // local coordinate system. Because mouse.xy is in local coords,
                    // we must also pass the circle center in local coords. Thus we pass
                    // [0.0, 0.0] as the center.
                    //
                    // See above where we define is_over_circ.
                    let is_over = is_over_circ([0.0, 0.0], mouse.xy, dim);

                    // See above where we define get_new_interaction.
                    get_new_interaction(is_over, state.view().interaction, mouse)
                },
            };

            if let (Interaction::BothClicked, Interaction::Highlighted) =
                (state.view().interaction, new_interaction)
            {
                // Recall that our Cell struct includes maybe_react, which
                // stores either a reaction function or None. If maybe_react is Some, call
                // the function.
                if let Some(ref mut react) = self.maybe_react {
                    react(Interaction::BothClicked);
                }
            }


            // If the mouse was released over the button, react. state.interaction is the
            // button's state as of a moment ago. new_interaction is the updated state as
            // of right now. So this if statement is saying: If the button was clicked a
            // moment ago, and it's now highlighted, then the button has been activated.
            if let (Interaction::LeftClicked, Interaction::Highlighted) =
                (state.view().interaction, new_interaction)
            {
                // Recall that our Cell struct includes maybe_react, which
                // stores either a reaction function or None. If maybe_react is Some, call
                // the function.
                if let Some(ref mut react) = self.maybe_react {
                    react(Interaction::LeftClicked);
                }
            }

            if let (Interaction::RightClicked, Interaction::Highlighted) =
                (state.view().interaction, new_interaction)
            {
                // Recall that our Cell struct includes maybe_react, which
                // stores either a reaction function or None. If maybe_react is Some, call
                // the function.
                if let Some(ref mut react) = self.maybe_react {
                    react(Interaction::RightClicked);
                }
            }

            // Here we check to see whether or not our button should capture the mouse.
            //
            // Widgets can "capture" user input. If the button captures the mouse, then mouse
            // events will only be seen by the button. Other widgets will not see mouse events
            // until the button uncaptures the mouse.
            match (state.view().interaction, new_interaction) {
                // If the user has pressed the button we capture the mouse.
                (Interaction::Highlighted, Interaction::LeftClicked) |
                (Interaction::Highlighted, Interaction::RightClicked) => {
                    ui.capture_mouse(idx);
                },
                // If the user releases the button, we uncapture the mouse.
                (Interaction::LeftClicked, Interaction::Highlighted) |
                (Interaction::LeftClicked, Interaction::Normal) |
                (Interaction::RightClicked, Interaction::Highlighted) |
                (Interaction::RightClicked, Interaction::Normal) => {
                    ui.uncapture_mouse(idx);
                },
                _ => (),
            }

            // Whenever we call `state.update` (as below), a flag is set within our `State`
            // indicating that there has been some mutation and that our widget requires a
            // re-draw. Thus, we only want to call `state.update` if there has been some change in
            // order to only re-draw when absolutely required.
            //
            // You can see how we do this below - we check if the state has changed before calling
            // `state.update`.

            // If the interaction has changed, set the new interaction.
            if state.view().interaction != new_interaction {
                state.update(|state| state.interaction = new_interaction);
            }

            // Finally, we'll describe how we want our widget drawn by simply instantiating the
            // necessary primitive graphics widgets.
            //
            // Conrod will automatically determine whether or not any changes have occurred and
            // whether or not any widgets need to be re-drawn.
            //
            // The primitive graphics widgets are special in that their unique state is used within
            // conrod's backend to do the actual drawing. This allows us to build up more complex
            // widgets by using these simple primitives with our familiar layout, coloring, etc
            // methods.
            //
            // If you notice that conrod is missing some sort of primitive graphics that you
            // require, please file an issue or open a PR so we can add it! :)

            // First, we'll draw the **Circle** with a radius that is half our given width.
            let radius = rect.w() / 2.0;
            let color = new_interaction.color(style.color(ui.theme()));
            let circle_idx = state.view().circle_idx.get(&mut ui);
            Rectangle::fill([radius, radius])
                .middle_of(idx)
                .graphics_for(idx)
                .color(color)
                .set(circle_idx, &mut ui);

            // Now we'll instantiate our label using the **Text** widget.
            let label_color = style.label_color(ui.theme());
            let font_size = style.label_font_size(ui.theme());
            let text_idx = state.view().text_idx.get(&mut ui);
            if let Some(ref label) = self.maybe_label {
                Text::new(label)
                    .middle_of(idx)
                    .font_size(font_size)
                    .graphics_for(idx)
                    .color(label_color)
                    .set(text_idx, &mut ui);
            }
        }

    }

    impl Style {
        /// Construct the default Style.
        pub fn new() -> Style {
            Style {
                maybe_color: None,
                maybe_radius: None,
                maybe_label_color: None,
                maybe_label_font_size: None,
            }
        }

        /// Get the Color for an Element.
        pub fn color(&self, theme: &Theme) -> Color {
            self.maybe_color.or(theme.widget_style::<Self>(KIND).map(|default| {
                default.style.maybe_color.unwrap_or(theme.shape_color)
            })).unwrap_or(theme.shape_color)
        }

        /// Get the label Color for an Element.
        pub fn label_color(&self, theme: &Theme) -> Color {
            self.maybe_label_color.or(theme.widget_style::<Self>(KIND).map(|default| {
                default.style.maybe_label_color.unwrap_or(theme.label_color)
            })).unwrap_or(theme.label_color)
        }

        /// Get the label font size for an Element.
        pub fn label_font_size(&self, theme: &Theme) -> FontSize {
            self.maybe_label_font_size.or(theme.widget_style::<Self>(KIND).map(|default| {
                default.style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
            })).unwrap_or(theme.font_size_medium)
        }
    }

    /// Provide the chainable color() configuration method.
    impl<'a, F> Colorable for Cell<'a, F> {
        fn color(mut self, color: Color) -> Self {
            self.style.maybe_color = Some(color);
            self
        }
    }

    /// Provide the chainable label(), label_color(), and label_font_size()
    /// configuration methods.
    impl<'a, F> Labelable<'a> for Cell<'a, F> {
        fn label(mut self, text: &'a str) -> Self {
            self.maybe_label = Some(text);
            self
        }
        fn label_color(mut self, color: Color) -> Self {
            self.style.maybe_label_color = Some(color);
            self
        }
        fn label_font_size(mut self, size: FontSize) -> Self {
            self.style.maybe_label_font_size = Some(size);
            self
        }
    }
// }
