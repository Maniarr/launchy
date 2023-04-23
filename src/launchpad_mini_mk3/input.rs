use crate::protocols::query::*;

use super::Button;

/// A Launchpad Mini MK3 input message
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Message {
    /// A button was pressed
    Press { button: Button },
    /// A button was released
    Release { button: Button },
    /// Emitted after a text scroll ends or loops
    TextEndedOrLooped,
    /// The response to a [device inquiry request](super::Output::request_device_inquiry)
    DeviceInquiry(DeviceInquiry),
    /// The response to a [version inquiry request](super::Output::request_version_inquiry)
    VersionInquiry(VersionInquiry),
    Unsupported,
}

fn decode_grid_button(btn: u8) -> Button {
    let x = (btn % 10) - 1;
    let y = (btn / 10) - 1;
    Button::GridButton { x, y }
}

/// The Launchpad Mini input connection creator.
pub struct Input;

impl crate::InputDevice for Input {
    const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad Mini MK3 MIDI 2";
    const MIDI_CONNECTION_NAME: &'static str = "Launchy Mini Mk3 Input";
    type Message = Message;

    fn decode_message(_timestamp: u64, data: &[u8]) -> Message {
        if let Some(device_inquiry) = parse_device_query(data) {
            return Message::DeviceInquiry(device_inquiry);
        }

        if let Some(version_inquiry) = parse_version_query(data) {
            return Message::VersionInquiry(version_inquiry);
        }

        // first byte of a launchpad midi message is the message type
        match data {
            // Note on
            &[0x90, button, velocity] => {
                let button = decode_grid_button(button);

                match velocity {
                    0 => Message::Release { button },
                    127 => Message::Press { button },
                    _ => Message::Unsupported,
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
                    _ => Message::Unsupported,
                }
            }
            &[0xB0, 0, 3] => Message::TextEndedOrLooped,
            _ => Message::Unsupported,
        }
    }
}
