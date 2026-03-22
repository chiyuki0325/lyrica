use std::collections::HashMap;
use zbus::proxy;
use zbus::zvariant::OwnedValue;

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
