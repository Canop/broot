use {
    super::{Command, CommandParts},
    crate::{app::AppContext, keys, selection_type::SelectionType, verb::Internal},
    termimad::{Event, InputField},
};

/// consume the event to
/// - maybe change the input
/// - build a command
pub fn to_command(
    event: Event,
    input_field: &mut InputField,
    con: &AppContext,
    selection_type: SelectionType,
) -> Command {
    match event {
        Event::Click(x, y, ..) => {
            return if input_field.apply_event(&event) {
                Command::empty()
            } else {
                Command::Click(x, y)
            };
        }
        Event::DoubleClick(x, y) => {
            return Command::DoubleClick(x, y);
        }
        Event::Key(key) => {
            // value of raw and parts before any key related change
            let raw = input_field.get_content();
            let parts = CommandParts::from(&raw);

            // we first handle the cases that MUST absolutely
            // not be overriden by configuration
            if key == keys::ENTER && parts.verb_invocation.is_some() {
                return Command::from_parts(&parts, true);
            }

            if key == keys::ESC {
                // Esc it's also a reserved key so order doesn't matter
                input_field.set_content("");
                let internal = Internal::back;
                return Command::Internal {
                    internal,
                    input_invocation: parts.verb_invocation,
                };
            }

            if key == keys::QUESTION && (raw.is_empty() || parts.verb_invocation.is_some()) {
                // a '?' opens the help when it's the first char
                // or when it's part of the verb invocation
                return Command::Internal {
                    internal: Internal::help,
                    input_invocation: parts.verb_invocation,
                };
            }

            // we now check if the key is the trigger key of one of the verbs
            if let Some(index) = con.verb_store.index_of_key(key) {
                if selection_type.respects(con.verb_store.verbs[index].selection_condition) {
                    return Command::VerbTrigger {
                        index,
                        input_invocation: parts.verb_invocation,
                    };
                } else {
                    debug!(
                        "verb {} not allowed on current selection",
                        &con.verb_store.verbs[index].name
                    );
                }
            }

            if key == keys::LEFT && raw.is_empty() {
                let internal = Internal::back;
                return Command::Internal {
                    internal,
                    input_invocation: parts.verb_invocation,
                };
            }

            if key == keys::RIGHT && raw.is_empty() {
                return Command::Internal {
                    internal: Internal::open_stay,
                    input_invocation: None,
                };
            }

            // input field management
            if input_field.apply_event(&event) {
                let raw = input_field.get_content();
                let parts = CommandParts::from(&raw);
                return Command::from_parts(&parts, false);
            }
        }
        Event::Wheel(lines_count) => {
            let internal = if lines_count > 0 {
                Internal::line_down
            } else {
                Internal::line_up
            };
            return Command::Internal {
                internal,
                input_invocation: None,
            };
        }
        _ => {}
    }
    Command::None
}
