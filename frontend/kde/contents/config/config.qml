import QtQuick 2.0

import org.kde.plasma.configuration 2.0

ConfigModel {
    id: configModel

    ConfigCategory {
        name: i18n("Display")
        icon: "preferences-desktop-color"
        source: "configFrontend.qml"
    }

    ConfigCategory {
        name: i18n("Behavior")
        icon: "configure"
        source: "configBackend.qml"
    }
}
