import QtQuick
import QtQuick.Controls
import QtQuick.Layouts 1.12
import org.kde.kirigami as Kirigami

Kirigami.FormLayout {
    id: page

    property alias cfg_characterLimit: characterLimit.text
    property alias cfg_shouldUseDefaultThemeFontSize: shouldUseDefaultThemeFontSize.checked
    property alias cfg_configuredFontSize: configuredFontSize.text
    property alias cfg_layoutHeight: layoutHeight.text
    property alias cfg_showReconnectingText: showReconnectingText.checked

    TextField {
        id: characterLimit
        Kirigami.FormData.label: i18n("Character Limit:")
        placeholderText: i18n("")
        validator: IntValidator {bottom: 0; top: 9999}
    }

    CheckBox {
        id: shouldUseDefaultThemeFontSize
        text: i18n("Ignore the setting below and use theme default size")
    }

    TextField {
        id: configuredFontSize
        Kirigami.FormData.label: i18n("Custom font size:")
        placeholderText: i18n("")
        validator: IntValidator {bottom: 0; top: 9999}
    }

    TextField {
        id: layoutHeight
        Kirigami.FormData.label: i18n("Layout Height:")
        placeholderText: i18n("")
        validator: IntValidator {bottom: 0; top: 9999}
    }

    CheckBox {
        id: showReconnectingText
        text: i18n("Show [Reconnecting...] text when connection lost")
    }

}
