use std::convert::TryInto;

use super::Button;


#[derive(Debug, Eq, PartialEq, Hash, Clone)]
/// A Launchpad MK2 input message
pub enum Message {
	/// A button was pressed
	Press { button: Button },
	/// A button was released
	Release { button: Button },
	/// Emitted after a text scroll was initiated
	TextEndedOrLooped,
	/// The response to a [device inquiry request](super::Output::request_device_inquiry)
	DeviceInquiry { device_id: u8, firmware_revision: u32 },
	/// The response to a [version inquiry request](super::Output::request_version_inquiry)
	VersionInquiry { bootloader_version: u32, firmware_version: u32 },
	/// Emitted when a fader was changed by the user, in [fader mode](super::Output::enter_fader_mode)
	FaderChange { index: u8, value: u8 },
}

/// The Launchpad MK2 input connection creator.
pub struct Input;

fn decode_short_message(data: &[u8]) -> Message {
	assert_eq!(data.len(), 3); // if this function was called, it should be

	// first byte of a launchpad midi message is the message type
	match data[0] {
		0x90 => { // Note on
			let button = decode_grid_button(data[1]);
			
			let velocity = data[2];
			match velocity {
				0 => return Message::Release { button },
				127 => return Message::Press { button },
				other => panic!("Unexpected grid note-on velocity {}", other),
			}
		},
		0xB0 => { // Controller change
			match data[1] {
				104..=111 => {
					let button = Button::ControlButton { index: data[1] - 104 };
	
					let velocity = data[2];
					match velocity {
						0 => return Message::Release { button },
						127 => return Message::Press { button },
						other => panic!("Unexpected control note-on velocity {}", other),
					}
				},
				21..=28 => {
					return Message::FaderChange { index: data[1] - 21, value: data[2] };
				},
				_ => panic!("Unexpected data byte 1. {:?}", data),
			}
		},
		// This is the note off code BUT it's not used by the launchpad. It sends zero-velocity
		// note-on messages instead
		0x80 => panic!("Unexpected note-on message: {:?}", data),
		_other => panic!("First byte of midi short messages was unexpected. {:?}", data),
	}
}

fn decode_sysex_message(data: &[u8]) -> Message {
	return match data {
		&[240, 0, 32, 41, 2, 24, 21, 247] => Message::TextEndedOrLooped,
		&[240, 126, device_id, 6, 2, 0, 32, 41, 105, 0, 0, 0, fr1, fr2, fr3, fr4, 247] => {
			let firmware_revision = u32::from_be_bytes([fr1, fr2, fr3, fr4]);
			Message::DeviceInquiry { device_id, firmware_revision }
		}
		&[240, 0, 32, 41, 0, 112, ref data @ .., 247] => {
			let data: [u8; 12] = data.try_into()
					.expect("Invalid version inquiry response length");
			
			let bootloader_version =
					data[0] as u32 * 10000 +
					data[1] as u32 * 1000 +
					data[2] as u32 * 100 +
					data[3] as u32 * 10 +
					data[4] as u32;
			
			let firmware_version =
					data[5] as u32 * 10000 +
					data[6] as u32 * 1000 +
					data[7] as u32 * 100 +
					data[8] as u32 * 10 +
					data[9] as u32;
			
			// Last two bytes are [13, 1] in my case, but the actual meaning of it is unknown.
			// Let's just ignore them here

			Message::VersionInquiry { bootloader_version, firmware_version }
		},
		other => panic!("Unexpected sysex message: {:?}", other),
	}
}

fn decode_grid_button(btn: u8) -> Button {
	println!("{:#02X}", &btn);

	return Button::GridButton { x: 0, y: 0 };
}

impl crate::InputDevice for Input {
	const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad Mini MK3 MIDI 2";
	const MIDI_CONNECTION_NAME: &'static str = "Launchy Mk3 Input";
	type Message = Message;

	fn decode_message(_timestamp: u64, data: &[u8]) -> Message {
		if data.len() == 3 {
			return decode_short_message(data);
		} else {
			return decode_sysex_message(data);
		}
	}
}
