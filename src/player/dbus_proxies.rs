use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zbus::proxy;
use zbus::zvariant::OwnedValue;
use zbus::zvariant::Type;

use crate::player::mpris_metadata::Metadata;

#[proxy(
    interface = "org.freedesktop.DBus",
    default_service = "org.freedesktop.DBus",
    default_path = "/org/freedesktop/DBus"
)]
pub(crate) trait DBus {
    #[zbus(signal)]
    fn name_owner_changed(&self, name: &str, old_owner: &str, new_owner: &str) -> zbus::Result<()>;

    fn list_names(&self) -> zbus::Result<Vec<String>>;
}

#[proxy(
    interface = "org.freedesktop.DBus.Properties",
    default_path = "/org/mpris/MediaPlayer2"
)]
pub(crate) trait PlayerProperties {
    #[zbus(signal)]
    fn properties_changed(
        &self,
        interface_name: &str,
        changed_properties: HashMap<String, OwnedValue>,
        invalidated_properties: Vec<String>,
    ) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_path = "/org/mpris/MediaPlayer2"
)]
pub(crate) trait Player {
    #[zbus(property)]
    fn metadata(&self) -> zbus::Result<Metadata>;

    #[zbus(property)]
    fn playback_status(&self) -> zbus::Result<PlaybackStatus>;

    #[zbus(property)]
    fn position(&self) -> zbus::Result<i64>;

    #[zbus(property)]
    fn rate(&self) -> zbus::Result<f64>;

    #[zbus(signal)]
    fn seeked(&self, position: i64) -> zbus::Result<()>;
}

#[derive(Serialize, Deserialize, Type, Debug, PartialEq)]
pub(crate) enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

impl TryFrom<OwnedValue> for PlaybackStatus {
    type Error = zbus::Error;

    fn try_from(value: OwnedValue) -> Result<Self, Self::Error> {
        if let Some(s) = value.downcast_ref::<String>().ok() {
            match s.as_str() {
                "Playing" => Ok(PlaybackStatus::Playing),
                "Paused" => Ok(PlaybackStatus::Paused),
                "Stopped" => Ok(PlaybackStatus::Stopped),
                _ => Err(zbus::Error::InvalidField),
            }
        } else {
            Err(zbus::Error::InvalidField)
        }
    }
}
