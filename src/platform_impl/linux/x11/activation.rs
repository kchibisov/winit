// SPDX-License-Identifier: Apache-2.0

//! X11 activation handling.
//!
//! X11 has a "startup notification" specification similar to Wayland's, see this URL:
//! https://specifications.freedesktop.org/startup-notification-spec/startup-notification-latest.txt

use super::{atoms::*, X11Error, XConnection};

use std::sync::atomic::{AtomicU32, Ordering};

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, ConnectionExt as _};

impl XConnection {
    /// "Request" a new activation token from the server.
    pub(crate) fn request_activation_token(&self) -> Result<String, X11Error> {
        // The specification recommends the format "hostname+pid+"_TIME"+current time"
        let uname = rustix::system::uname();
        let pid = rustix::process::getpid();
        let time = self.x11_timestamp()?;

        let activation_token = format!(
            "{}{}_TIME{}",
            uname.nodename().to_str().unwrap_or("winit"),
            pid.as_raw_nonzero(),
            time
        );

        Ok(activation_token)
    }

    /// Finish launching a window with the given startup ID.
    pub(crate) fn remove_activation_token(
        &self,
        window: xproto::Window,
        startup_id: &str,
    ) -> Result<(), X11Error> {
        let atoms = self.atoms();

        // Set the _NET_STARTUP_ID property on the window.
        self.xcb_connection()
            .change_property(
                xproto::PropMode::REPLACE,
                window,
                atoms[_NET_STARTUP_ID],
                xproto::AtomEnum::STRING,
                8,
                startup_id.len().try_into().unwrap(),
                startup_id.as_bytes(),
            )?
            .check()?;

        // Send the message indicating that the startup is over.
        let message = {
            const MESSAGE_ROOT: &str = "remove: ID=";

            let mut buffer =
                String::with_capacity(MESSAGE_ROOT.len().checked_add(startup_id.len()).unwrap());

            buffer.push_str(MESSAGE_ROOT);
            buffer.push_str(startup_id);
            buffer.push('\0');

            buffer.into_bytes()
        };

        self.send_message(&message)
    }

    /// Get the current X11 timestamp.
    fn x11_timestamp(&self) -> Result<xproto::Timestamp, X11Error> {
        // TODO: Figure out if the value returned here actually matters.
        static SEED: AtomicU32 = AtomicU32::new(0xDEADBEEF);
        let seed = SEED.load(Ordering::Relaxed);

        // Pseudorandom number generator from the "Xorshift RNGs" paper by George Marsaglia.
        let mut r = seed;
        r ^= r << 13;
        r ^= r >> 17;
        r ^= r << 5;
        SEED.store(r, Ordering::Relaxed);
        Ok(seed)
    }

    /// Send a startup notification message to the window manager.
    fn send_message(&self, message: &[u8]) -> Result<(), X11Error> {
        let atoms = self.atoms();

        // Create a new window to send the message over.
        let screen = self.default_root();
        let window = self.xcb_connection().generate_id()?;
        self.xcb_connection()
            .create_window(
                screen.root_depth,
                screen.root,
                window,
                -100,
                -100,
                1,
                1,
                0,
                xproto::WindowClass::INPUT_OUTPUT,
                screen.root_visual,
                &xproto::CreateWindowAux::new()
                    .override_redirect(1)
                    .event_mask(
                        xproto::EventMask::STRUCTURE_NOTIFY | xproto::EventMask::PROPERTY_CHANGE,
                    ),
            )?
            .ignore_error();

        let _drop_window = CallOnDrop(|| {
            if let Ok(token) = self.xcb_connection().destroy_window(window) {
                token.ignore_error();
            }
        });

        // Serialize the messages in 20-byte chunks.
        let mut message_type = atoms[_NET_STARTUP_INFO_BEGIN];
        message
            .chunks(20)
            .map(|chunk| {
                let mut buffer = [0u8; 20];
                buffer[..chunk.len()].copy_from_slice(chunk);
                let event = xproto::ClientMessageEvent::new(8, window, message_type, buffer);

                // Set the message type to the continuation atom for the next chunk.
                message_type = atoms[_NET_STARTUP_INFO];

                event
            })
            .try_for_each(|event| {
                // Send each event in order.
                self.xcb_connection()
                    .send_event(
                        false,
                        screen.root,
                        xproto::EventMask::PROPERTY_CHANGE,
                        event,
                    )
                    .map(|c| c.ignore_error())
            })?;

        Ok(())
    }
}

struct CallOnDrop<F: FnMut()>(F);

impl<F: FnMut()> Drop for CallOnDrop<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}
