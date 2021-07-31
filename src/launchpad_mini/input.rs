use super::Button;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
/// A Launchpad Mini input message
pub enum Message {
    /// A button was pressed
    Press { button: Button },
    /// A button was released
    Release { button: Button },
}

/// The Launchpad Mini input connection creator.
pub struct Input;

fn decode_grid_button(btn: u8) -> Button {
    let x = btn % 16;
    let y = btn / 16;
    return Button::GridButton { x, y };
}

impl crate::InputDevice for Input {
    const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad Mini";
    const MIDI_CONNECTION_NAME: &'static str = "Launchy Mini Input";
    type Message = Message;

    fn decode_message(_timestamp: u64, data: &[u8]) -> Message {
        // first byte of a launchpad midi message is the message type
        return match data {
            // Note on
            &[0x90, button, velocity] => {
                let button = decode_grid_button(button);

                match velocity {
                    0 => Message::Release { button },
                    127 => Message::Press { button },
                    other => panic!("Unexpected grid note-on velocity {}", other),
                }
            }
            // Controller change
            &[0xB0, number @ 104..=111, velocity] => {
                let button = Button::ControlButton {
                    index: number - 104,
                };

                match velocity {
                    0 => Message::Release { button },
                    127 => Message::Press { button },
                    other => panic!("Unexpected control note-on velocity {}", other),
                }
            }
            // YES we have no note off message handler here because it's not used by the launchpad.
            // It sends zero-velocity note-on messages instead.
            other => panic!("Unexpected midi message: {:?}", other),
        };
    }
}
